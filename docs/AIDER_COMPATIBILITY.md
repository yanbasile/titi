# Aider.chat Compatibility with Titi Terminal

This document explains how to use [Aider](https://aider.chat/), an AI pair programming tool, with the Titi terminal emulator.

## Table of Contents

1. [What is Aider?](#what-is-aider)
2. [Compatibility Analysis](#compatibility-analysis)
3. [Installation](#installation)
4. [Running Aider in Titi](#running-aider-in-titi)
5. [Orchestrating Multiple Aider Sessions](#orchestrating-multiple-aider-sessions)
6. [Use Cases](#use-cases)
7. [Troubleshooting](#troubleshooting)

---

## What is Aider?

**Aider** is an AI pair programming tool that runs in your terminal, enabling chat-driven code edits, new files, and refactors directly from the command line.

### Key Features

- **AI Pair Programming**: Chat with AI to edit code in your local git repo
- **Multiple LLM Support**: Works with Claude 3.7 Sonnet, GPT-4o, DeepSeek, o1, and local models
- **Git Integration**: Automatically commits changes with descriptive messages
- **Multi-Language**: Supports Python, JavaScript, Rust, Go, C++, and dozens more
- **Terminal-Based**: Runs entirely in your terminal with rich ANSI formatting

### Links

- **Official Website**: [aider.chat](https://aider.chat/)
- **GitHub**: [github.com/Aider-AI/aider](https://github.com/Aider-AI/aider)
- **Documentation**: [aider.chat/docs/](https://aider.chat/docs/)
- **Installation Guide**: [aider.chat/docs/install.html](https://aider.chat/docs/install.html)

---

## Compatibility Analysis

### ✅ Titi Terminal Capabilities

Titi is **fully compatible** with Aider. Here's why:

**Terminal Emulation:**
- ✅ **VTE Parser**: Industry-standard terminal parser (same as used by Alacritty, GNOME Terminal)
- ✅ **ANSI/VT100 Support**: Complete escape sequence support
- ✅ **PTY (Pseudo-Terminal)**: Full pseudo-terminal support via `portable-pty`
- ✅ **Interactive I/O**: Proper stdin/stdout/stderr handling

**Text Rendering:**
- ✅ **256 Colors**: Full 256-color palette support
- ✅ **RGB Colors**: True color (24-bit) support
- ✅ **Text Styling**: Bold, italic, underline, strikethrough, inverse
- ✅ **Unicode**: Full UTF-8 support

**Cursor & Screen Control:**
- ✅ **Cursor Movement**: Up, down, left, right, absolute positioning
- ✅ **Cursor Save/Restore**: Save and restore cursor position
- ✅ **Screen Clearing**: Clear screen, clear line
- ✅ **Scroll Regions**: Scrolling region support

**Advanced Features:**
- ✅ **Line Editing**: Readline support (via PTY)
- ✅ **Control Characters**: Proper handling of Ctrl+C, Ctrl+D, etc.
- ✅ **Backspace/Delete**: Full editing support

### Aider Requirements

Aider requires:
- ✅ Terminal with ANSI escape sequence support → **Titi has this**
- ✅ Python 3.9-3.12 → **System requirement, not terminal**
- ✅ Git repository → **System requirement, not terminal**
- ✅ LLM API key → **Application requirement, not terminal**

**Verdict**: Aider will run **perfectly** in Titi terminal.

---

## Installation

### Prerequisites

1. **Python 3.9-3.12** (or 3.8-3.13 with uv)
2. **Git** (for repository integration)
3. **LLM API Key** (Claude, OpenAI, DeepSeek, etc.)

### Installation Methods

**Method 1: Using pipx (Recommended)**

```bash
# Install pipx if you don't have it
python3 -m pip install --user pipx
python3 -m pipx ensurepath

# Install aider
pipx install aider-chat

# Verify installation
aider --version
```

**Method 2: Using uv (Most Flexible)**

```bash
# Install uv
curl -LsSf https://astral.sh/uv/install.sh | sh

# Install aider (uv will handle Python version automatically)
uv tool install aider-chat

# Verify installation
aider --version
```

**Method 3: Using pip**

```bash
# Install aider
python3 -m pip install --user aider-chat

# Add to PATH if needed
export PATH="$HOME/.local/bin:$PATH"

# Verify installation
aider --version
```

### Setting Up API Keys

Aider needs an LLM API key. Set it as an environment variable:

**For Claude (Recommended):**

```bash
export ANTHROPIC_API_KEY=your-api-key-here

# Make it permanent
echo 'export ANTHROPIC_API_KEY=your-api-key-here' >> ~/.bashrc
source ~/.bashrc
```

**For OpenAI:**

```bash
export OPENAI_API_KEY=your-api-key-here
echo 'export OPENAI_API_KEY=your-api-key-here' >> ~/.bashrc
source ~/.bashrc
```

**For DeepSeek:**

```bash
export DEEPSEEK_API_KEY=your-api-key-here
echo 'export DEEPSEEK_API_KEY=your-api-key-here' >> ~/.bashrc
source ~/.bashrc
```

---

## Running Aider in Titi

### Basic Usage

1. **Start Titi Terminal:**

```bash
# Build and run Titi
cd /path/to/titi
cargo build --release
./target/release/titi
```

2. **Navigate to Your Project:**

```bash
cd /path/to/your/project
git status  # Ensure it's a git repo
```

3. **Start Aider:**

```bash
aider
```

You'll see Aider's colorful interface with a prompt:

```
Aider v0.x.x
Main model: claude-3-5-sonnet-20241022
Weak model: claude-3-5-haiku-20241022
Git repo: /path/to/your/project
Repo-map: universal-ctags using 1024 tokens

>
```

### Example Session

```bash
# Start aider with specific files
aider src/main.rs src/lib.rs

# Chat with AI
> Add a function to calculate fibonacci numbers

# Aider will:
# 1. Analyze your code
# 2. Suggest changes
# 3. Apply changes if you approve
# 4. Commit with descriptive message

# Add more files to the chat
/add src/utils.rs

# Run shell commands
/run cargo test

# Ask questions
> Explain how the grid system works

# Exit
/exit
```

### Aider Commands

While in Aider, you can use these commands:

- `/add <file>` - Add files to the chat
- `/drop <file>` - Remove files from chat
- `/run <cmd>` - Run shell command
- `/undo` - Undo last git commit
- `/diff` - Show pending changes
- `/commit` - Commit pending changes
- `/tokens` - Show token usage
- `/help` - Show all commands
- `/exit` - Exit aider

---

## Orchestrating Multiple Aider Sessions

This is where Titi + Redititi really shine! You can run multiple Aider sessions in parallel, each working on different parts of your project.

### Use Case: Parallel Development

**Setup:**

```python
from titipy import TitiClient
import asyncio

async def run_aider_task(client, task_name, files, prompt):
    """Run Aider in a dedicated terminal session"""

    # Create session for this Aider instance
    session = await client.create_session_async(f"aider-{task_name}")
    pane = await session.create_pane_async()

    # Start Aider with specific files
    file_args = " ".join(files)
    await pane.inject_async(f"cd /path/to/project && aider {file_args}")

    # Wait for Aider to be ready
    await pane.wait_for_async(">", timeout=10)

    # Send the task prompt
    await pane.inject_async(prompt)

    # Wait for completion (look for git commit message)
    output = await pane.capture_until_async("Commit", timeout=300)

    return task_name, output

async def parallel_aider_development():
    """Run 4 Aider sessions in parallel"""
    client = TitiClient()

    tasks = [
        ("frontend",
         ["src/ui/dashboard.rs", "src/ui/components.rs"],
         "Create a beautiful dashboard UI with metrics widgets"),

        ("backend",
         ["src/api/server.rs", "src/api/handlers.rs"],
         "Implement REST API endpoints for user authentication"),

        ("database",
         ["src/db/schema.rs", "src/db/queries.rs"],
         "Add database migrations for user profiles"),

        ("tests",
         ["tests/integration_tests.rs"],
         "Write comprehensive integration tests for the API"),
    ]

    # Run all Aider sessions in parallel
    results = await asyncio.gather(*[
        run_aider_task(client, name, files, prompt)
        for name, files, prompt in tasks
    ])

    for task_name, output in results:
        print(f"\n=== {task_name} completed ===")
        print(output)

# Run it
asyncio.run(parallel_aider_development())
```

**Architecture:**

```
┌────────────────────────────────────────┐
│    Orchestration Script (Python)       │
│         (titipy client)                │
└──────┬──────┬──────┬──────┬────────────┘
       │      │      │      │
       │      │      │      │
   ┌───▼──┐ ┌─▼───┐ ┌▼────┐ ┌▼────┐
   │Titi#1│ │Titi2│ │Titi3│ │Titi4│
   │head- │ │head-│ │head-│ │head-│
   │less  │ │less │ │less │ │less │
   └───┬──┘ └─┬───┘ └┬────┘ └┬────┘
       │      │      │      │
   ┌───▼──┐ ┌─▼───┐ ┌▼────┐ ┌▼────┐
   │Aider │ │Aider│ │Aider│ │Aider│
   │Front │ │Back │ │ DB  │ │Test │
   │end   │ │end  │ │     │ │     │
   └──────┘ └─────┘ └─────┘ └─────┘
```

### Use Case: Review Pipeline

Run Aider sessions in a pipeline where each reviews the previous work:

```python
async def aider_review_pipeline(project_path):
    """Sequential review with multiple Aider instances"""
    client = TitiClient()

    # Step 1: Implementer writes code
    implementer = await client.create_session_async("implementer")
    await implementer.inject_async(f"cd {project_path} && aider")
    await implementer.inject_async("Implement user authentication system")
    code = await implementer.capture_until_async("Commit")

    # Step 2: Security reviewer checks for vulnerabilities
    security = await client.create_session_async("security")
    await security.inject_async(f"cd {project_path} && aider")
    await security.inject_async(
        "Review the authentication code for security vulnerabilities. "
        "Focus on SQL injection, XSS, and authentication bypasses."
    )
    security_report = await security.capture_until_async("Commit")

    # Step 3: Performance reviewer optimizes
    performance = await client.create_session_async("performance")
    await performance.inject_async(f"cd {project_path} && aider")
    await performance.inject_async(
        "Optimize the authentication code for performance. "
        "Add caching where appropriate and reduce database queries."
    )
    perf_report = await performance.capture_until_async("Commit")

    # Step 4: Documentation specialist adds docs
    docs = await client.create_session_async("docs")
    await docs.inject_async(f"cd {project_path} && aider")
    await docs.inject_async(
        "Add comprehensive documentation and examples for the "
        "authentication system."
    )
    docs_report = await docs.capture_until_async("Commit")

    return {
        "implementation": code,
        "security": security_report,
        "performance": perf_report,
        "documentation": docs_report
    }
```

---

## Use Cases

### 1. Multi-Repository Development

Work on multiple repositories simultaneously:

```python
repos = [
    "/path/to/frontend",
    "/path/to/backend",
    "/path/to/mobile",
    "/path/to/infrastructure"
]

for repo in repos:
    session = await client.create_session_async(f"aider-{repo.split('/')[-1]}")
    await session.inject_async(f"cd {repo} && aider")
```

### 2. Specialized AI Agents

Create specialized Aider instances with different focuses:

```python
specialists = {
    "rust-expert": "You are a Rust expert. Focus on memory safety and performance.",
    "security-audit": "You are a security auditor. Look for vulnerabilities.",
    "refactoring": "You are a refactoring specialist. Improve code quality.",
    "testing": "You are a testing expert. Write comprehensive tests."
}

for role, context in specialists.items():
    session = await client.create_session_async(f"aider-{role}")
    await session.inject_async(f"aider --message '{context}'")
```

### 3. CI/CD Integration

Run Aider in headless mode for automated code reviews:

```bash
#!/bin/bash
# ci-aider-review.sh

# Start Titi in headless mode
titi --headless --session ci-review &

# Wait for terminal to be ready
sleep 2

# Connect to redititi and run Aider
echo "AUTH $(cat ~/.titi/token)" | nc localhost 6379
echo "PUBLISH session-ci-review/pane-1/input 'cd /project && aider --yes-always'" | nc localhost 6379
echo "PUBLISH session-ci-review/pane-1/input 'Review this PR for bugs and suggest improvements'" | nc localhost 6379

# Capture output
echo "SUBSCRIBE session-ci-review/pane-1/output" | nc localhost 6379
echo "RPOP session-ci-review/pane-1/output" | nc localhost 6379
```

### 4. Interactive Development with Multiple Models

Run different LLMs side-by-side:

```python
models = [
    ("claude", "claude-3-5-sonnet-20241022"),
    ("gpt4", "gpt-4o"),
    ("deepseek", "deepseek-chat")
]

for name, model in models:
    session = await client.create_session_async(f"aider-{name}")
    await session.inject_async(f"aider --model {model}")
    await session.inject_async("Implement a binary search tree in Rust")

# Compare implementations
```

---

## Troubleshooting

### Issue: Aider Not Found

**Problem**: `aider: command not found`

**Solution**:
```bash
# Make sure aider is in PATH
export PATH="$HOME/.local/bin:$PATH"

# Or use full path
~/.local/bin/aider
```

### Issue: API Key Not Set

**Problem**: `No API key found`

**Solution**:
```bash
# Set the appropriate API key
export ANTHROPIC_API_KEY=your-key
# or
export OPENAI_API_KEY=your-key

# Make it permanent
echo 'export ANTHROPIC_API_KEY=your-key' >> ~/.bashrc
```

### Issue: Colors Not Displaying

**Problem**: Colors look wrong or missing

**Solution**:
- Titi supports full color! This shouldn't happen.
- Make sure you're running the release build: `cargo run --release`
- Check GPU drivers if rendering looks broken

### Issue: Git Repository Required

**Problem**: `Aider requires a git repository`

**Solution**:
```bash
# Initialize git if needed
git init
git add .
git commit -m "Initial commit"

# Then run aider
aider
```

### Issue: Readline/Input Issues

**Problem**: Keyboard input feels slow or unresponsive

**Solution**:
- Use release build (debug builds are slower)
- Check if other processes are using high CPU
- Titi's PTY implementation should handle this well

### Issue: Unicode Characters Broken

**Problem**: Special characters or emojis don't render

**Solution**:
- Titi supports UTF-8 fully
- Make sure your font supports the characters
- Try a different font in `~/.config/titi/config.toml`:

```toml
[font]
family = "JetBrains Mono"  # or "Fira Code", "Source Code Pro"
size = 14.0
```

---

## Performance Tips

### 1. Use Release Build

Always use the release build for better performance:

```bash
cargo build --release
./target/release/titi
```

### 2. Headless Mode for Automation

When running multiple Aider instances, use headless mode:

```bash
titi --headless --session aider-1
titi --headless --session aider-2
titi --headless --session aider-3
```

This saves GPU resources and improves performance.

### 3. Limit Context Window

For faster responses, limit the files Aider analyzes:

```bash
# Only include relevant files
aider src/specific/file.rs

# Instead of
aider src/**/*.rs
```

### 4. Use Weak Models for Simple Tasks

Aider uses two models - main and weak. The weak model is faster:

```bash
aider --weak-model claude-3-5-haiku-20241022
```

---

## Summary

**Aider works perfectly with Titi!**

✅ **Fully Compatible**: All terminal features supported
✅ **Rich Formatting**: Colors, styles, unicode all work
✅ **Multi-Session**: Run multiple Aider instances in parallel
✅ **Automation**: Use redititi for orchestration
✅ **Headless Mode**: Perfect for CI/CD (Phase 3)

**Getting Started:**

1. Install Aider: `pipx install aider-chat`
2. Set API key: `export ANTHROPIC_API_KEY=your-key`
3. Run Titi: `cargo run --release`
4. Start Aider: `aider`
5. Start coding with AI!

**For Multi-Agent Orchestration:**

See [ARCHITECTURE.md](../ARCHITECTURE.md) for details on using Titi + Redititi to orchestrate multiple Aider sessions in parallel.

---

## Additional Resources

- **Aider Documentation**: https://aider.chat/docs/
- **Aider GitHub**: https://github.com/Aider-AI/aider
- **Titi Architecture**: [ARCHITECTURE.md](../ARCHITECTURE.md)
- **Getting Started**: [GETTING_STARTED.md](../GETTING_STARTED.md)
- **Titi Repository**: https://github.com/yourusername/titi

---

**Happy AI pair programming! 🚀**
