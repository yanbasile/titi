use std::sync::Arc;
use std::time::{Duration, Instant};
use titi::{renderer::Renderer, ui::PaneManager, Config};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, ModifiersState, NamedKey},
    window::{Window, WindowId},
};

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    pane_manager: PaneManager,
    config: Config,
    modifiers: ModifiersState,
    last_frame: Instant,
}

impl App {
    fn new(config: Config) -> Self {
        Self {
            window: None,
            renderer: None,
            pane_manager: PaneManager::new(),
            config,
            modifiers: ModifiersState::default(),
            last_frame: Instant::now(),
        }
    }

    async fn initialize_renderer(&mut self) -> anyhow::Result<()> {
        if let Some(window) = &self.window {
            let renderer = Renderer::new(window.clone(), &self.config).await?;
            self.renderer = Some(renderer);
        }
        Ok(())
    }

    fn handle_key(&mut self, event: KeyEvent) {
        if event.state != ElementState::Pressed {
            return;
        }

        // Handle keyboard shortcuts
        match &event.logical_key {
            Key::Named(NamedKey::Enter) if self.modifiers.control_key() => {
                // Ctrl+Enter: Create new pane
                if let Some(renderer) = &self.renderer {
                    let (cell_width, cell_height) = renderer.cell_dimensions();
                    let window_size = self.window.as_ref().unwrap().inner_size();
                    let cols = (window_size.width as f32 / cell_width) as u16;
                    let rows = (window_size.height as f32 / cell_height) as u16;

                    if let Err(e) = self.pane_manager.create_pane(cols, rows) {
                        log::error!("Failed to create pane: {}", e);
                    }
                }
            }
            Key::Character(c) if c == "t" && self.modifiers.control_key() => {
                // Ctrl+T: New terminal (same as Ctrl+Enter)
                if let Some(renderer) = &self.renderer {
                    let (cell_width, cell_height) = renderer.cell_dimensions();
                    let window_size = self.window.as_ref().unwrap().inner_size();
                    let cols = (window_size.width as f32 / cell_width) as u16;
                    let rows = (window_size.height as f32 / cell_height) as u16;

                    if let Err(e) = self.pane_manager.create_pane(cols, rows) {
                        log::error!("Failed to create pane: {}", e);
                    }
                }
            }
            Key::Character(c) if c == "h" && self.modifiers.control_key() => {
                // Ctrl+H: Split horizontal
                if let Some(pane_id) = self.pane_manager.active_pane() {
                    if let Some(renderer) = &self.renderer {
                        let (cell_width, cell_height) = renderer.cell_dimensions();
                        let window_size = self.window.as_ref().unwrap().inner_size();
                        let cols = (window_size.width as f32 / cell_width / 2.0) as u16;
                        let rows = (window_size.height as f32 / cell_height) as u16;

                        if let Err(e) = self.pane_manager.split_pane(
                            pane_id,
                            titi::ui::SplitDirection::Horizontal,
                            cols.max(20),
                            rows.max(5),
                        ) {
                            log::error!("Failed to split pane: {}", e);
                        }
                    }
                }
            }
            Key::Character(c) if c == "v" && self.modifiers.control_key() => {
                // Ctrl+V: Split vertical
                if let Some(pane_id) = self.pane_manager.active_pane() {
                    if let Some(renderer) = &self.renderer {
                        let (cell_width, cell_height) = renderer.cell_dimensions();
                        let window_size = self.window.as_ref().unwrap().inner_size();
                        let cols = (window_size.width as f32 / cell_width) as u16;
                        let rows = (window_size.height as f32 / cell_height / 2.0) as u16;

                        if let Err(e) = self.pane_manager.split_pane(
                            pane_id,
                            titi::ui::SplitDirection::Vertical,
                            cols.max(20),
                            rows.max(5),
                        ) {
                            log::error!("Failed to split pane: {}", e);
                        }
                    }
                }
            }
            Key::Character(c) if c == "w" && self.modifiers.control_key() => {
                // Ctrl+W: Close active pane
                if let Some(pane_id) = self.pane_manager.active_pane() {
                    self.pane_manager.close_pane(pane_id);
                }
            }
            Key::Named(NamedKey::ArrowUp) if self.modifiers.control_key() => {
                // Ctrl+ArrowUp: Navigate to pane above
                self.pane_manager.navigate_up();
            }
            Key::Named(NamedKey::ArrowDown) if self.modifiers.control_key() => {
                // Ctrl+ArrowDown: Navigate to pane below
                self.pane_manager.navigate_down();
            }
            Key::Named(NamedKey::ArrowLeft) if self.modifiers.control_key() => {
                // Ctrl+ArrowLeft: Navigate to pane on left
                self.pane_manager.navigate_left();
            }
            Key::Named(NamedKey::ArrowRight) if self.modifiers.control_key() => {
                // Ctrl+ArrowRight: Navigate to pane on right
                self.pane_manager.navigate_right();
            }
            Key::Named(NamedKey::PageUp) => {
                // PageUp: Scroll back in history
                if let Some(pane_id) = self.pane_manager.active_pane() {
                    if let Some(pane) = self.pane_manager.get_pane_mut(pane_id) {
                        // Scroll back by half a screen
                        let (_, rows) = {
                            let grid = pane.terminal.grid();
                            let g = grid.lock().unwrap();
                            g.size()
                        };
                        pane.terminal.scroll_back_up(rows / 2);
                    }
                }
            }
            Key::Named(NamedKey::PageDown) => {
                // PageDown: Scroll forward in history
                if let Some(pane_id) = self.pane_manager.active_pane() {
                    if let Some(pane) = self.pane_manager.get_pane_mut(pane_id) {
                        // Scroll forward by half a screen
                        let (_, rows) = {
                            let grid = pane.terminal.grid();
                            let g = grid.lock().unwrap();
                            g.size()
                        };
                        pane.terminal.scroll_back_down(rows / 2);
                    }
                }
            }
            Key::Named(NamedKey::Home) if self.modifiers.shift_key() => {
                // Shift+Home: Scroll to top of history
                if let Some(pane_id) = self.pane_manager.active_pane() {
                    if let Some(pane) = self.pane_manager.get_pane_mut(pane_id) {
                        let scrollback_len = {
                            let grid = pane.terminal.grid();
                            let g = grid.lock().unwrap();
                            g.scrollback_len()
                        };
                        pane.terminal.scroll_back_up(scrollback_len);
                    }
                }
            }
            Key::Named(NamedKey::End) if self.modifiers.shift_key() => {
                // Shift+End: Scroll to bottom (current)
                if let Some(pane_id) = self.pane_manager.active_pane() {
                    if let Some(pane) = self.pane_manager.get_pane_mut(pane_id) {
                        pane.terminal.scroll_to_bottom();
                    }
                }
            }
            _ => {
                // Send input to active pane
                if let Some(text) = self.key_to_bytes(&event) {
                    if let Some(pane_id) = self.pane_manager.active_pane() {
                        if let Some(pane) = self.pane_manager.get_pane_mut(pane_id) {
                            // Scroll to bottom on any input
                            pane.terminal.scroll_to_bottom();

                            if let Err(e) = pane.terminal.write(&text) {
                                log::error!("Failed to write to terminal: {}", e);
                            }
                        }
                    }
                }
            }
        }
    }

    fn key_to_bytes(&self, event: &KeyEvent) -> Option<Vec<u8>> {
        match &event.logical_key {
            Key::Named(NamedKey::Enter) => Some(b"\r".to_vec()),
            Key::Named(NamedKey::Tab) => Some(b"\t".to_vec()),
            Key::Named(NamedKey::Backspace) => Some(b"\x7f".to_vec()),
            Key::Named(NamedKey::ArrowUp) => Some(b"\x1b[A".to_vec()),
            Key::Named(NamedKey::ArrowDown) => Some(b"\x1b[B".to_vec()),
            Key::Named(NamedKey::ArrowRight) => Some(b"\x1b[C".to_vec()),
            Key::Named(NamedKey::ArrowLeft) => Some(b"\x1b[D".to_vec()),
            Key::Named(NamedKey::Home) => Some(b"\x1b[H".to_vec()),
            Key::Named(NamedKey::End) => Some(b"\x1b[F".to_vec()),
            Key::Named(NamedKey::PageUp) => Some(b"\x1b[5~".to_vec()),
            Key::Named(NamedKey::PageDown) => Some(b"\x1b[6~".to_vec()),
            Key::Named(NamedKey::Delete) => Some(b"\x1b[3~".to_vec()),
            Key::Named(NamedKey::Escape) => Some(b"\x1b".to_vec()),
            Key::Character(c) => {
                let mut bytes = c.to_string().into_bytes();

                // Handle Ctrl+ combinations
                if self.modifiers.control_key() && bytes.len() == 1 {
                    let byte = bytes[0];
                    if byte >= b'a' && byte <= b'z' {
                        // Ctrl+a = 1, Ctrl+b = 2, etc.
                        bytes[0] = byte - b'a' + 1;
                    } else if byte >= b'A' && byte <= b'Z' {
                        bytes[0] = byte - b'A' + 1;
                    }
                }

                Some(bytes)
            }
            _ => None,
        }
    }

    fn poll_terminals(&mut self) {
        let pane_ids: Vec<_> = self.pane_manager.panes().keys().copied().collect();

        for pane_id in pane_ids {
            if let Some(pane) = self.pane_manager.get_pane_mut(pane_id) {
                match pane.terminal.read() {
                    Ok(Some(data)) => {
                        pane.terminal.process_output(&data);
                    }
                    Ok(None) => {}
                    Err(e) => {
                        log::error!("Error reading from terminal: {}", e);
                    }
                }
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attrs = Window::default_attributes()
                .with_title(&self.config.window.title)
                .with_inner_size(winit::dpi::LogicalSize::new(
                    self.config.window.width,
                    self.config.window.height,
                ));

            match event_loop.create_window(window_attrs) {
                Ok(window) => {
                    let window = Arc::new(window);
                    self.window = Some(window.clone());

                    // Initialize renderer asynchronously
                    let future = self.initialize_renderer();
                    pollster::block_on(future).expect("Failed to initialize renderer");

                    // Create initial pane
                    if let Some(renderer) = &self.renderer {
                        let (cell_width, cell_height) = renderer.cell_dimensions();
                        let window_size = window.inner_size();
                        let cols = (window_size.width as f32 / cell_width) as u16;
                        let rows = (window_size.height as f32 / cell_height) as u16;

                        self.pane_manager
                            .create_pane(cols.max(80), rows.max(24))
                            .expect("Failed to create initial pane");
                    }
                }
                Err(e) => {
                    log::error!("Failed to create window: {}", e);
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(physical_size);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_key(event);
            }
            WindowEvent::ModifiersChanged(new_modifiers) => {
                self.modifiers = new_modifiers.state();
            }
            WindowEvent::RedrawRequested => {
                // Poll terminals for output
                self.poll_terminals();

                // Render all panes
                if let Some(renderer) = &mut self.renderer {
                    if let Err(e) = renderer.render_panes(&self.pane_manager) {
                        log::error!("Render error: {}", e);
                    }
                }

                // Request next frame
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Limit frame rate to ~60 FPS
        let now = Instant::now();
        let elapsed = now - self.last_frame;
        if elapsed < Duration::from_millis(16) {
            std::thread::sleep(Duration::from_millis(16) - elapsed);
        }
        self.last_frame = Instant::now();

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Starting Titi Terminal Emulator");

    let config = Config::load().unwrap_or_default();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(config);
    event_loop.run_app(&mut app)?;

    Ok(())
}
