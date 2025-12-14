# Titi Architecture: Multi-Agent Terminal Orchestration

This document explains the architecture of the Titi terminal emulator and Redititi automation server, designed specifically for orchestrating multiple Claude Code terminal agents.

## Table of Contents

1. [System Overview](#system-overview)
2. [Core Components](#core-components)
3. [Communication Architecture](#communication-architecture)
4. [Command Injection](#command-injection)
5. [Screen Capture](#screen-capture)
6. [Headless Mode](#headless-mode)
7. [Claude Code Orchestration](#claude-code-orchestration)
8. [Multi-Agent Patterns](#multi-agent-patterns)
9. [API Reference](#api-reference)

---

## System Overview

Titi consists of two main applications that work together:

```
┌─────────────────────────────────────────────────────────────────┐
│                      Orchestration Layer                         │
│                   (Python/titipy or any client)                  │
└────────────┬───────────────────────────────┬────────────────────┘
             │                               │
             │ TCP Connection                │ TCP Connection
             │ (port 6379)                   │ (port 6379)
             │                               │
    ┌────────▼──────────┐         ┌─────────▼─────────┐
    │   Redititi Server │◄────────►│  Redititi Server  │
    │   (Standalone)    │  Pub/Sub │   (Standalone)    │
    └────────┬──────────┘  Channels└─────────┬─────────┘
             │                               │
             │ Injects Commands              │ Injects Commands
             │ Captures Output               │ Captures Output
             │                               │
    ┌────────▼──────────┐         ┌─────────▼─────────┐
    │  Titi Terminal #1 │         │  Titi Terminal #2 │
    │   (with GPU)      │         │   (--headless)    │
    │                   │         │                   │
    │  ┌──────────────┐ │         │  ┌──────────────┐ │
    │  │ Claude Code  │ │         │  │ Claude Code  │ │
    │  │   Agent #1   │ │         │  │   Agent #2   │ │
    │  └──────────────┘ │         │  └──────────────┘ │
    └───────────────────┘         └───────────────────┘
```

### Components

1. **Titi Terminal** (`titi`): GPU-accelerated terminal emulator
   - Runs Claude Code agents interactively
   - Supports headless mode (no GPU) for CI/automation
   - Publishes output to redititi channels
   - Receives commands from redititi channels

2. **Redititi Server** (`redititi`): Redis-like automation server
   - Standalone TCP server (port 6379)
   - Token-based authentication
   - Session/pane registry
   - Pub/sub channel system
   - Command injection API
   - Screen capture API

3. **Client Libraries** (`titipy`): Python automation library (planned)
   - High-level API for terminal control
   - Session/pane management
   - Command execution and output capture
   - Pytest fixtures for testing

---

## Core Components

### 1. Redititi Server

The redititi server is a standalone Redis-like TCP server that acts as a central hub for terminal automation.

**Key Features:**

- **TCP Server**: Listens on `127.0.0.1:6379` (localhost-only for security)
- **Token Authentication**: Single global token (~/.titi/token)
- **Session Registry**: Manages multiple terminal sessions
- **Pane Registry**: Manages multiple panes within sessions
- **Pub/Sub Channels**: Real-time message broadcasting
- **FIFO Queues**: Message consumption (read = consumed)

**Architecture:**

```
Redititi Server
├── TCP Listener (tokio)
│   └── Connection Handler (per-connection authentication)
├── Authentication System
│   ├── Token Generation (64-char random)
│   └── Token Validation (max 3 attempts)
├── Registry
│   ├── Sessions (HashMap<SessionId, SessionInfo>)
│   └── Panes (HashMap<(SessionId, PaneId), PaneInfo>)
├── Channel Manager
│   ├── Channels (HashMap<ChannelName, Channel>)
│   └── FIFO Message Queues (VecDeque<Message>)
└── Command Handler
    ├── Session Commands (CREATE, LIST, CLOSE)
    ├── Channel Commands (SUBSCRIBE, PUBLISH, RPOP)
    └── Terminal Commands (INJECT, CAPTURE)
```

### 2. Titi Terminal

The titi terminal is a GPU-accelerated terminal emulator that integrates with redititi.

**Key Features:**

- **GPU Rendering**: wgpu-based text rendering
- **PTY Management**: Full pseudo-terminal support
- **ANSI/VT100**: Complete escape sequence support
- **Headless Mode**: Run without GPU for automation
- **Server Integration**: Connects to redititi on startup
- **Output Publishing**: Real-time output to channels
- **Input Subscription**: Receives commands from channels

**Architecture:**

```
Titi Terminal
├── Window Manager (winit) [disabled in headless]
│   └── Event Loop (keyboard, mouse, resize)
├── GPU Renderer (wgpu) [disabled in headless]
│   ├── Glyph Atlas (font caching)
│   └── Text Renderer (shader-based)
├── Terminal Backend
│   ├── PTY (portable-pty)
│   ├── Grid (terminal buffer)
│   └── Parser (VTE escape sequences)
├── Server Client [PLANNED - Phase 3]
│   ├── Connection (TCP to redititi)
│   ├── Authentication (token from ~/.titi/token)
│   ├── Output Publisher (grid → channel)
│   └── Input Subscriber (channel → PTY)
└── UI System
    ├── Pane Manager (splits, focus)
    └── Layout Engine (hierarchical)
```

---

## Communication Architecture

### Channel Naming Convention

Channels use a hierarchical naming scheme:

```
session-{session_id}/pane-{pane_id}/input   # Commands to pane
session-{session_id}/pane-{pane_id}/output  # Output from pane
session-{session_id}/input                  # Broadcast to all panes
_registry/sessions                          # Session creation events
_control/create                             # Control commands
```

**Examples:**

```
session-libre-ph1/pane-swift-red5/input     # Input to specific pane
session-libre-ph1/pane-swift-red5/output    # Output from specific pane
session-libre-ph1/input                     # Broadcast to all panes in session
```

### Communication Flow

**1. Command Injection (Client → Terminal):**

```
┌────────────┐     PUBLISH      ┌──────────────┐     Subscribe     ┌──────────┐
│   Client   │─────────────────►│   Redititi   │◄──────────────────│   Titi   │
│  (Python)  │  session-X/      │    Server    │  session-X/       │ Terminal │
│            │  pane-Y/input    │              │  pane-Y/input     │          │
└────────────┘  "ls -la\n"      └──────────────┘                   └─────┬────┘
                                                                          │
                                                                    Executes
                                                                    Command
                                                                          │
                                                                          ▼
                                                                       Shell
```

**2. Screen Capture (Terminal → Client):**

```
┌──────────┐                    ┌──────────────┐                   ┌────────────┐
│   Titi   │     PUBLISH        │   Redititi   │    RPOP/SUBSCRIBE │   Client   │
│ Terminal │───────────────────►│    Server    │◄──────────────────│  (Python)  │
│          │  session-X/        │              │  session-X/       │            │
│          │  pane-Y/output     │              │  pane-Y/output    │            │
└────┬─────┘  "file1.txt\n"    └──────────────┘                   └────────────┘
     │
   Shell
  Output
```

### Message Format

Messages are plain text with simple protocol:

**Commands:**

```
AUTH <token>
LIST SESSIONS
LIST PANES <session_id>
CREATE SESSION [name] [first_pane_name]
CREATE PANE <session_id> [name]
SUBSCRIBE <channel>
PUBLISH <channel> <message>
INJECT <target> <command> [NOWAIT|QUEUE|BATCH]
CAPTURE <target> [FULL|LINES|STREAM]
RPOP <channel>
LLEN <channel>
CLOSE PANE <session_id> <pane_id>
CLOSE SESSION <session_id>
```

**Responses:**

```
+OK                     # Success
+OK session-libre-ph1   # Success with data
-ERR message            # Error
"string"                # String response
["item1", "item2"]      # Array response
{"key": "value"}        # JSON response
```

---

## Command Injection

Command injection allows external clients to send commands to terminal panes programmatically.

### How It Works

1. **Client publishes to input channel:**
   ```
   PUBLISH session-libre-ph1/pane-swift-red5/input "npm test\n"
   ```

2. **Redititi queues the message**

3. **Titi terminal (subscribed) receives the message**

4. **Terminal writes to PTY input:**
   ```rust
   pty.write_all(b"npm test\n")
   ```

5. **Shell executes the command**

### Injection Modes

**NOWAIT (Default):**
- Send command immediately
- Don't wait for completion
- Return immediately

**QUEUE:**
- Queue command after previous commands
- Wait for previous to complete
- Execute in order

**BATCH:**
- Send multiple commands at once
- Execute sequentially
- Return when all complete

### Example Usage

**Python Client (titipy - planned):**

```python
from titipy import TitiClient

# Connect to redititi
client = TitiClient(port=6379)

# Create a session with Claude Code
session = client.create_session("claude-dev")
pane = session.create_pane()

# Start Claude Code
pane.inject("claude --version")

# Wait for output
output = pane.capture_lines(1)
print(output)  # "Claude Code version X.Y.Z"

# Send a task to Claude Code
pane.inject("help me write a function to sort a list")

# Capture response
response = pane.capture_stream(timeout=30)
print(response)
```

**Direct TCP (netcat):**

```bash
# Connect
nc localhost 6379

# Authenticate
AUTH abc123...xyz789

# Create session
CREATE SESSION claude-session-1

# Inject command
PUBLISH session-claude-session-1/pane-1/input "ls -la\n"

# Read output
SUBSCRIBE session-claude-session-1/pane-1/output
RPOP session-claude-session-1/pane-1/output
```

### Security Considerations

- **Localhost-only**: Server binds to 127.0.0.1 (no external access)
- **Token authentication**: 64-character random token required
- **Auto-append newline**: Commands automatically get `\n` appended
- **No command history**: Commands not stored server-side
- **Session isolation**: Panes can't interfere with each other

---

## Screen Capture

Screen capture allows external clients to extract terminal output programmatically.

### How It Works

1. **Terminal publishes output:**
   ```rust
   // Every time terminal grid updates
   let dirty_lines = grid.get_dirty_lines();
   for (line_num, content) in dirty_lines {
       publish(output_channel, format!("L{}: {}", line_num, content));
   }
   ```

2. **Redititi queues output in channel**

3. **Client retrieves output:**
   ```
   RPOP session-libre-ph1/pane-swift-red5/output
   ```

4. **Message consumed (FIFO)**

### Capture Modes

**FULL:**
- Capture entire terminal grid
- Returns all rows
- Includes cursor position
- JSON format

**LINES:**
- Capture specific line range
- Efficient for targeted extraction
- Text format

**STREAM:**
- Real-time streaming mode
- Subscribe to output channel
- Receive updates as they happen
- Auto-scroll support

### Example Usage

**Full Grid Capture:**

```python
# Capture entire terminal state
grid = pane.capture_full()
print(grid['rows'][0])  # First row
print(grid['cursor'])   # Cursor position
print(grid['size'])     # (cols, rows)
```

**Line-by-Line Capture:**

```python
# Wait for specific output
while True:
    line = pane.capture_line()
    if "Error:" in line:
        print(f"Found error: {line}")
        break
    if "Success:" in line:
        print(f"Task completed: {line}")
        break
```

**Streaming Capture:**

```python
# Stream output in real-time
for output in pane.stream_output(timeout=60):
    print(output, end='')
    if "DONE" in output:
        break
```

### Dirty Rectangle Tracking

Titi uses dirty rectangle tracking for efficient screen updates:

- Only changed cells are marked dirty
- Output publishes only dirty lines
- Reduces network traffic
- Improves performance

**Implementation:**

```rust
pub struct Grid {
    cells: Vec<Cell>,
    dirty_rects: Vec<Rect>,
    // ...
}

impl Grid {
    pub fn mark_dirty(&mut self, x: usize, y: usize) {
        self.dirty_rects.push(Rect { x, y, w: 1, h: 1 });
    }

    pub fn get_dirty_lines(&mut self) -> Vec<(usize, String)> {
        let lines = self.extract_dirty_lines();
        self.dirty_rects.clear();
        lines
    }
}
```

---

## Headless Mode

Headless mode allows Titi to run without a GPU or display, perfect for CI/CD and automation.

### Why Headless Mode?

- **Automation**: Run terminals in CI/CD pipelines
- **Docker**: Run in containers without display
- **SSH**: Remote terminal access without X11
- **Testing**: Automated test suites with pytest
- **Scalability**: Spawn hundreds of terminals
- **Cost**: No GPU required

### Architecture

**Normal Mode:**
```
Titi Terminal
├── Window (winit)           ← ACTIVE
├── GPU Renderer (wgpu)      ← ACTIVE
├── Terminal Backend         ← ACTIVE
└── Server Client           ← ACTIVE
```

**Headless Mode:**
```
Titi Terminal --headless
├── Window (winit)           ← DISABLED
├── GPU Renderer (wgpu)      ← DISABLED
├── Terminal Backend         ← ACTIVE
└── Server Client           ← ACTIVE
```

### Implementation (Planned - Phase 3)

```rust
pub struct TerminalConfig {
    pub headless: bool,
    pub server_connect: bool,
    pub session_id: Option<String>,
    pub pane_id: Option<String>,
}

pub fn run(config: TerminalConfig) {
    if config.headless {
        run_headless(config)
    } else {
        run_windowed(config)
    }
}

fn run_headless(config: TerminalConfig) {
    // Create terminal backend only (no GPU)
    let terminal = Terminal::new(80, 24);

    // Connect to redititi
    let client = ServerClient::connect("127.0.0.1:6379").await?;
    client.authenticate(&read_token()?).await?;

    // Create or join session
    let session_id = config.session_id.unwrap_or_else(||
        client.create_session(None).await?
    );

    let pane_id = config.pane_id.unwrap_or_else(||
        client.create_pane(&session_id, None).await?
    );

    // Subscribe to input channel
    let input_channel = format!("{}/pane-{}/input", session_id, pane_id);
    client.subscribe(&input_channel).await?;

    // Publish output to output channel
    let output_channel = format!("{}/pane-{}/output", session_id, pane_id);

    // Run event loop (PTY I/O only, no rendering)
    loop {
        // Read from PTY
        let output = terminal.read_output()?;
        if !output.is_empty() {
            client.publish(&output_channel, &output).await?;
        }

        // Read from input channel
        if let Some(msg) = client.rpop(&input_channel).await? {
            terminal.write_input(&msg.content)?;
        }

        // No rendering needed!
    }
}
```

### Usage

**Command Line:**

```bash
# Start in headless mode
titi --headless

# Start with specific session
titi --headless --session my-session

# Start and connect to redititi
titi --headless --server 127.0.0.1:6379 --token-file ~/.titi/token
```

**Environment Variables:**

```bash
export TITI_HEADLESS=1
export TITI_SESSION=claude-agent-1
export TITI_SERVER=127.0.0.1:6379
titi
```

**Docker:**

```dockerfile
FROM rust:latest

RUN cargo install titi

ENV TITI_HEADLESS=1
ENV TITI_SERVER=host.docker.internal:6379

CMD ["titi"]
```

---

## Claude Code Orchestration

Titi and Redititi are designed specifically for orchestrating multiple Claude Code agents.

### Use Cases

1. **Parallel Development**: Multiple agents working on different features
2. **Testing Automation**: Agents running tests in parallel
3. **Code Review**: Agent reviewing code while another continues development
4. **Multi-Repository**: Agents working across different repositories
5. **CI/CD Integration**: Automated agent workflows in pipelines

### Single Claude Code Agent

**Setup:**

```python
from titipy import TitiClient

client = TitiClient()

# Create session for Claude Code
session = client.create_session("claude-dev")
pane = session.create_pane()

# Start Claude Code
pane.inject("claude")

# Wait for prompt
pane.wait_for("Claude Code")

# Send task
pane.inject("/help")

# Capture response
response = pane.capture_until(">>>")
print(response)
```

**Workflow:**

```
┌──────────┐
│  Python  │
│  Script  │
└────┬─────┘
     │
     │ create_session()
     ▼
┌──────────────┐
│  Redititi    │
│   Server     │
└──────┬───────┘
       │
       │ INJECT "claude"
       ▼
┌──────────────┐
│ Titi Terminal│
│              │
│ ┌──────────┐ │
│ │  Claude  │ │
│ │   Code   │ │
│ └──────────┘ │
└──────────────┘
```

### Multiple Claude Code Agents (Parallel)

**Setup:**

```python
from titipy import TitiClient
import asyncio

async def run_agent(client, task_name, task):
    # Each agent gets own session
    session = await client.create_session_async(f"claude-{task_name}")
    pane = await session.create_pane_async()

    # Start Claude Code
    await pane.inject_async("claude")
    await pane.wait_for_async("Claude Code")

    # Execute task
    await pane.inject_async(task)

    # Capture result
    result = await pane.capture_until_async("TASK_COMPLETE", timeout=300)

    return task_name, result

async def main():
    client = TitiClient()

    tasks = [
        ("frontend", "Build the React dashboard component"),
        ("backend", "Implement the REST API endpoints"),
        ("tests", "Write integration tests for the API"),
        ("docs", "Update the API documentation"),
    ]

    # Run all agents in parallel
    results = await asyncio.gather(*[
        run_agent(client, name, task)
        for name, task in tasks
    ])

    for name, result in results:
        print(f"{name}: {result}")

asyncio.run(main())
```

**Architecture:**

```
┌────────────────────────────────────────────────────────┐
│                 Orchestration Script                    │
│                    (Python/titipy)                      │
└──┬────────────┬────────────┬────────────┬──────────────┘
   │            │            │            │
   │ Task 1     │ Task 2     │ Task 3     │ Task 4
   │            │            │            │
┌──▼──────┐ ┌──▼──────┐ ┌──▼──────┐ ┌──▼──────┐
│Redititi │ │Redititi │ │Redititi │ │Redititi │
│Server #1│ │Server #2│ │Server #3│ │Server #4│
└──┬──────┘ └──┬──────┘ └──┬──────┘ └──┬──────┘
   │            │            │            │
┌──▼──────┐ ┌──▼──────┐ ┌──▼──────┐ ┌──▼──────┐
│ Titi #1 │ │ Titi #2 │ │ Titi #3 │ │ Titi #4 │
│headless │ │headless │ │headless │ │headless │
│         │ │         │ │         │ │         │
│ Claude  │ │ Claude  │ │ Claude  │ │ Claude  │
│ Agent#1 │ │ Agent#2 │ │ Agent#3 │ │ Agent#4 │
└─────────┘ └─────────┘ └─────────┘ └─────────┘
```

### Agent Coordination

**Master-Worker Pattern:**

```python
class ClaudeOrchestrator:
    def __init__(self):
        self.client = TitiClient()
        self.master = None
        self.workers = []

    async def start_master(self):
        """Master agent coordinates workers"""
        session = await self.client.create_session_async("claude-master")
        pane = await session.create_pane_async()
        await pane.inject_async("claude")
        self.master = pane
        return pane

    async def start_worker(self, worker_id):
        """Worker agent executes tasks"""
        session = await self.client.create_session_async(f"claude-worker-{worker_id}")
        pane = await session.create_pane_async()
        await pane.inject_async("claude")
        self.workers.append(pane)
        return pane

    async def coordinate(self, project_task):
        """Master breaks down task and assigns to workers"""
        # Ask master to plan
        await self.master.inject_async(f"Break down this task: {project_task}")
        subtasks = await self.master.capture_json_async()

        # Assign subtasks to workers
        results = await asyncio.gather(*[
            worker.execute_task_async(subtask)
            for worker, subtask in zip(self.workers, subtasks)
        ])

        # Master reviews results
        await self.master.inject_async(f"Review these results: {results}")
        final_result = await self.master.capture_async()

        return final_result

# Usage
orchestrator = ClaudeOrchestrator()
await orchestrator.start_master()
for i in range(4):
    await orchestrator.start_worker(i)

result = await orchestrator.coordinate("Build a full-stack web application")
```

### Pipeline Pattern

**Sequential Agent Pipeline:**

```python
async def agent_pipeline(task):
    """
    Task flows through multiple specialized agents:
    Design → Implementation → Testing → Documentation
    """
    client = TitiClient()

    # Agent 1: Designer
    designer = await client.create_session_async("designer")
    await designer.inject_async(f"Design the architecture for: {task}")
    design = await designer.capture_async()

    # Agent 2: Implementer (receives design)
    implementer = await client.create_session_async("implementer")
    await implementer.inject_async(f"Implement this design: {design}")
    code = await implementer.capture_async()

    # Agent 3: Tester (receives code)
    tester = await client.create_session_async("tester")
    await tester.inject_async(f"Test this code: {code}")
    test_results = await tester.capture_async()

    # Agent 4: Documenter (receives everything)
    documenter = await client.create_session_async("documenter")
    context = f"Design: {design}\nCode: {code}\nTests: {test_results}"
    await documenter.inject_async(f"Document: {context}")
    docs = await documenter.capture_async()

    return {
        "design": design,
        "code": code,
        "tests": test_results,
        "docs": docs
    }
```

---

## Multi-Agent Patterns

### Pattern 1: Fan-Out / Fan-In

Distribute work to multiple agents, then aggregate results.

```python
async def fan_out_fan_in(tasks):
    """
    Distribute tasks to N agents, aggregate results
    """
    client = TitiClient()

    # Fan-out: Create agent for each task
    agents = []
    for i, task in enumerate(tasks):
        session = await client.create_session_async(f"agent-{i}")
        pane = await session.create_pane_async()
        await pane.inject_async("claude")
        agents.append(pane)

    # Execute in parallel
    results = await asyncio.gather(*[
        agent.execute_async(task)
        for agent, task in zip(agents, tasks)
    ])

    # Fan-in: Aggregate results
    aggregator = await client.create_session_async("aggregator")
    await aggregator.inject_async(f"Combine these results: {results}")
    final = await aggregator.capture_async()

    return final
```

### Pattern 2: Agent Pool

Maintain a pool of ready agents for task execution.

```python
class AgentPool:
    def __init__(self, size=4):
        self.client = TitiClient()
        self.pool = asyncio.Queue(maxsize=size)
        self.size = size

    async def start(self):
        """Initialize agent pool"""
        for i in range(self.size):
            session = await self.client.create_session_async(f"pool-agent-{i}")
            pane = await session.create_pane_async()
            await pane.inject_async("claude")
            await pane.wait_for_async("Claude Code")
            await self.pool.put(pane)

    async def execute_task(self, task):
        """Get agent from pool, execute, return to pool"""
        agent = await self.pool.get()
        try:
            result = await agent.execute_async(task)
            return result
        finally:
            await self.pool.put(agent)

    async def execute_many(self, tasks):
        """Execute tasks using pool"""
        return await asyncio.gather(*[
            self.execute_task(task)
            for task in tasks
        ])

# Usage
pool = AgentPool(size=4)
await pool.start()

tasks = ["task1", "task2", "task3", "task4", "task5", "task6"]
results = await pool.execute_many(tasks)
```

### Pattern 3: Specialized Agent Teams

Different agents with different specializations.

```python
class AgentTeam:
    def __init__(self):
        self.client = TitiClient()
        self.agents = {}

    async def add_specialist(self, role, prompt):
        """Add specialized agent"""
        session = await self.client.create_session_async(f"specialist-{role}")
        pane = await session.create_pane_async()
        await pane.inject_async("claude")
        await pane.inject_async(f"You are a {role}. {prompt}")
        self.agents[role] = pane

    async def consult(self, role, question):
        """Consult specific specialist"""
        if role not in self.agents:
            raise ValueError(f"No {role} specialist")

        agent = self.agents[role]
        await agent.inject_async(question)
        return await agent.capture_async()

    async def team_meeting(self, topic):
        """All specialists discuss topic"""
        responses = {}
        for role, agent in self.agents.items():
            await agent.inject_async(f"As a {role}, comment on: {topic}")
            responses[role] = await agent.capture_async()

        # Synthesize responses
        synthesis = await self.consult("coordinator",
            f"Synthesize these perspectives: {responses}")

        return synthesis

# Usage
team = AgentTeam()
await team.add_specialist("architect", "Focus on system design")
await team.add_specialist("security", "Focus on security concerns")
await team.add_specialist("performance", "Focus on optimization")
await team.add_specialist("coordinator", "Synthesize team input")

result = await team.team_meeting("Design a distributed cache")
```

### Pattern 4: Monitoring and Health Checks

Monitor agent health and restart if needed.

```python
class AgentMonitor:
    def __init__(self):
        self.client = TitiClient()
        self.agents = {}
        self.health_check_interval = 30  # seconds

    async def start_agent(self, name):
        """Start agent with monitoring"""
        session = await self.client.create_session_async(name)
        pane = await session.create_pane_async()
        await pane.inject_async("claude")

        self.agents[name] = {
            "pane": pane,
            "session": session,
            "last_response": time.time(),
            "healthy": True
        }

        # Start health check
        asyncio.create_task(self.monitor_agent(name))

    async def monitor_agent(self, name):
        """Monitor agent health"""
        while name in self.agents:
            agent = self.agents[name]

            # Send ping
            await agent["pane"].inject_async("echo PING")

            try:
                # Wait for pong
                response = await asyncio.wait_for(
                    agent["pane"].wait_for_async("PING"),
                    timeout=5.0
                )
                agent["last_response"] = time.time()
                agent["healthy"] = True
            except asyncio.TimeoutError:
                # Agent not responding
                agent["healthy"] = False
                await self.restart_agent(name)

            await asyncio.sleep(self.health_check_interval)

    async def restart_agent(self, name):
        """Restart unhealthy agent"""
        print(f"Restarting agent {name}")

        # Close old session
        old_agent = self.agents[name]
        await old_agent["session"].close_async()

        # Start new session
        await self.start_agent(name)

    async def get_healthy_agents(self):
        """Get list of healthy agents"""
        return [
            name for name, agent in self.agents.items()
            if agent["healthy"]
        ]
```

---

## API Reference

### Redititi Server Commands

**Authentication:**
```
AUTH <token>
  → +OK
  → -ERR Invalid token
```

**Session Management:**
```
CREATE SESSION [name] [first_pane_name]
  → +OK session-libre-ph1

LIST SESSIONS
  → ["session-libre-ph1", "session-swift-red5"]

CLOSE SESSION <session_id>
  → +OK
```

**Pane Management:**
```
CREATE PANE <session_id> [name]
  → +OK pane-swift-red5

LIST PANES <session_id>
  → ["pane-swift-red5", "pane-bold-gold3"]

CLOSE PANE <session_id> <pane_id>
  → +OK
```

**Channel Operations:**
```
SUBSCRIBE <channel>
  → +OK

UNSUBSCRIBE <channel>
  → +OK

PUBLISH <channel> <message>
  → +OK

RPOP <channel>
  → "message content"
  → -ERR Queue empty

LLEN <channel>
  → +OK 42
```

**Terminal Control:**
```
INJECT <target> <command> [NOWAIT|QUEUE|BATCH]
  → +OK

CAPTURE <target> [FULL|LINES|STREAM]
  → {"rows": [...], "cursor": {...}}
```

### Python Client (titipy - Planned)

```python
from titipy import TitiClient, Session, Pane

# Connection
client = TitiClient(host="127.0.0.1", port=6379, token="...")
await client.connect_async()

# Session management
session = await client.create_session_async("my-session")
sessions = await client.list_sessions_async()
await session.close_async()

# Pane management
pane = await session.create_pane_async("my-pane")
panes = await session.list_panes_async()
await pane.close_async()

# Command injection
await pane.inject_async("ls -la\n")
await pane.inject_async("npm test", mode="NOWAIT")

# Screen capture
output = await pane.capture_line_async()
grid = await pane.capture_full_async()
async for line in pane.stream_output_async():
    print(line)

# Utilities
await pane.wait_for_async("DONE", timeout=30)
result = await pane.execute_and_capture_async("pwd", timeout=5)
```

---

## Summary

The Titi + Redititi architecture provides a powerful foundation for orchestrating multiple Claude Code agents:

**Key Features:**
- ✅ Standalone server for centralized control
- ✅ Headless mode for automation and CI/CD
- ✅ Real-time command injection
- ✅ Screen capture with dirty tracking
- ✅ Token-based security
- ✅ Pub/sub for flexible communication
- ✅ Session/pane isolation
- ✅ Scalable to hundreds of agents

**Perfect For:**
- 🤖 Multi-agent development workflows
- 🔄 Parallel task execution
- 🧪 Automated testing with multiple agents
- 📊 Agent coordination and orchestration
- 🐳 Containerized agent deployments
- ⚡ CI/CD integration

**Next Steps:**
1. Implement Phase 3 (terminal integration)
2. Implement headless mode
3. Create Python client library (titipy)
4. Write pytest fixtures for testing
5. Build example orchestration patterns

---

For more information, see:
- [GETTING_STARTED.md](GETTING_STARTED.md) - Installation and setup
- [README.md](README.md) - Project overview
- [TESTING.md](TESTING.md) - Running tests
