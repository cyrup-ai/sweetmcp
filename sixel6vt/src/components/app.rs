// Use RioConfig alias
use rio_backend::config::Config as RioConfig;
// Use rio-backend's EventProxy
use rio_backend::event::EventProxy;
use rio_window::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    // Use KeyCode from rio_window::keyboard
    keyboard::KeyCode,
    event_loop::{ActiveEventLoop, ControlFlow},
    window::WindowId,
};
// Import EventPayload for EventLoop and run closure
use rio_backend::event::{EventPayload, RioEvent, RioEventType};
use std::sync::Arc;
use tracing::{error, info};
// Add StartCause for ApplicationHandler
use rio_window::event::StartCause;

use crate::components::browser::BrowserPane; // Import to use BrowserPane directly
use crate::components::terminal::TerminalPane;
use tokio; // Keep tokio import

// Reintroduce Application struct
pub struct Application {
    config: RioConfig,
    terminal_pane: Option<TerminalPane<EventProxy>>, // Make generic
    browser_pane: Arc<BrowserPane>, // Holds the browser interaction logic
    event_proxy: EventProxy, // Proxy for sending events to rio-backend
    // Track window dimensions for layout calculations
    window_width: u32,
    window_height: u32,
    // Whether we're in split mode (true) or fullscreen terminal mode (false)
    split_mode: bool,
}

impl Application {
    // Accept dependencies in constructor
    pub fn new(
        config: RioConfig,
        browser_pane: Arc<BrowserPane>,
        event_proxy: EventProxy,
    ) -> Self {
        Self {
            config,
            terminal_pane: None, // Initialize terminal later
            browser_pane,
            event_proxy,
            window_width: 1200, // Default initial size
            window_height: 800, // Default initial size
            split_mode: true,   // Start in split layout mode
        }
    }
    
    // Calculate the dimensions for the current layout
    // Use the calculate_layout_dimensions function with the split mode parameter
    
    // Toggle between split and fullscreen terminal modes and return the new state
    // This is actively used in the KeyF handler
    fn toggle_split_mode(&mut self) -> bool {
        self.split_mode = !self.split_mode;
        self.split_mode
    }
    
    // Update the layout when the split mode or window size changes
    fn calculate_layout_dimensions(&self, is_split: bool) -> (u32, u32, u32, u32) {
        if is_split {
            // 50/50 split: left side for browser, right side for terminal
            let browser_width = self.window_width / 2;
            let terminal_width = self.window_width - browser_width; // Account for odd numbers
            
            (browser_width, self.window_height, terminal_width, self.window_height)
        } else {
            // Fullscreen terminal mode: use the entire window
            (0, 0, self.window_width, self.window_height)
        }
    }
    
    // This is used when window configuration changes
    fn update_layout(&mut self) {
        // Calculate layout dimensions first with a separate method to avoid borrow issues
        let (_browser_width, _browser_height, terminal_width, terminal_height) = self.calculate_layout_dimensions(self.split_mode);
        
        if let Some(terminal_pane) = self.terminal_pane.as_mut() {
            
            // Resize terminal
            terminal_pane.resize(terminal_width, terminal_height);
            
            // Reposition the terminal according to the mode
            if self.split_mode {
                // Position on the right side in split mode
                terminal_pane.window.set_outer_position(rio_window::dpi::PhysicalPosition::new(
                    self.window_width / 2, 0
                ));
            } else {
                // Position at the left edge in fullscreen mode
                terminal_pane.window.set_outer_position(rio_window::dpi::PhysicalPosition::new(
                    0, 0
                ));
            }
            
            terminal_pane.window.request_redraw();
        }
    }
}

// Implement ApplicationHandler
impl ApplicationHandler<EventPayload> for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create the TerminalPane (which includes the window) on first resume
        if self.terminal_pane.is_none() {
            info!("Creating Terminal Pane with 50/50 split layout...");
            
            // Calculate dimensions for the split layout
            let (_browser_width, _browser_height, terminal_width, terminal_height) = 
                self.calculate_layout_dimensions(self.split_mode);
            
            // Ensure event_proxy was stored during run_app
            let proxy = self.event_proxy.clone();
            
            // Handle the Result from TerminalPane::new and store Option
            match TerminalPane::new(event_loop, proxy, &self.config) {
                Ok(mut pane) => {
                    let window_id = pane.window.id();
                    info!("Terminal Pane created successfully (Window ID: {:?})", window_id);
                    
                    // Resize to fill only the right half of the screen
                    pane.resize(terminal_width, terminal_height);
                    
                    // Position the window on the right side
                    pane.window.set_outer_position(rio_window::dpi::PhysicalPosition::new(
                        self.window_width / 2, 0
                    ));
                    
                    // Send some test content to verify terminal is working
                    // (this will be displayed immediately if rendering is working)
                    let test_msg = "\r\n\x1b[32mTerminal initialized\x1b[0m - 50/50 split with browser\r\n";
                    let test_payload = RioEvent::PtyWrite(test_msg.to_string());
                    self.event_proxy.send_event(RioEventType::Rio(test_payload), pane.window.id());
                    
                    // Force a redraw of the terminal with a Render event
                    let render_event = RioEvent::Render;
                    self.event_proxy.send_event(RioEventType::Rio(render_event), pane.window.id());
                    
                    pane.window.request_redraw(); // Request initial draw
                    self.terminal_pane = Some(pane); // Store the pane
                    
                    // Initialize browser and display it on the left side
                    let browser = self.browser_pane.clone();
                    let proxy = self.event_proxy.clone();
                    
                    // Send browser content to term
                    tokio::spawn(async move {
                        match browser.update_and_send_sixel(proxy, window_id).await {
                            Ok(_) => info!("Initial browser Sixel render completed"),
                            Err(e) => error!("Failed to render initial browser content: {}", e)
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to create Terminal Pane: {:?}", e); // Use {:?} for error
                    // Consider exiting or handling the error appropriately
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // Ensure the event is for the terminal pane's window
        if let Some(terminal_pane) = self.terminal_pane.as_mut() {
            if terminal_pane.window.id() == window_id {
                match event {
                    WindowEvent::CloseRequested => {
                        info!("Terminal window {:?} close requested. Exiting.", window_id);
                        event_loop.exit();
                    }
                    WindowEvent::Resized(physical_size) => {
                        // Update overall window dimensions
                        self.window_width = physical_size.width;
                        self.window_height = physical_size.height;
                        
                        // Calculate dimensions directly to avoid borrow issues
                        let (_browser_width, _browser_height, terminal_width, terminal_height) = 
                            if self.split_mode {
                                let bw = self.window_width / 2;
                                let tw = self.window_width - bw;
                                (bw, self.window_height, tw, self.window_height)
                            } else {
                                (0, 0, self.window_width, self.window_height)
                            };
                        
                        // Resize terminal to only take up the right half
                        terminal_pane.resize(terminal_width, terminal_height);
                        
                        // Reposition the terminal based on mode
                        if self.split_mode {
                            terminal_pane.window.set_outer_position(rio_window::dpi::PhysicalPosition::new(
                                self.window_width / 2, 0
                            ));
                        } else {
                            terminal_pane.window.set_outer_position(rio_window::dpi::PhysicalPosition::new(
                                0, 0
                            ));
                        }
                        
                        terminal_pane.window.request_redraw();
                        
                        // Re-render browser view when window resizes
                        let browser = self.browser_pane.clone();
                        let proxy = self.event_proxy.clone();
                        let term_window_id = terminal_pane.window.id();
                        
                        tokio::spawn(async move {
                            match browser.update_and_send_sixel(proxy, term_window_id).await {
                                Ok(_) => info!("Browser Sixel updated after resize"),
                                Err(e) => error!("Failed to update browser Sixel after resize: {}", e)
                            }
                        });
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        // Keyboard shortcuts for browser interaction
                        if event.state == ElementState::Pressed {
                            match event.physical_key {
                                // Refresh browser content with B key
                                rio_window::keyboard::PhysicalKey::Code(KeyCode::KeyB) => {
                                    info!("B pressed: Triggering browser Sixel update...");
                                    let browser = self.browser_pane.clone();
                                    let proxy = self.event_proxy.clone();
                                    let term_window_id = terminal_pane.window.id();
                                    
                                    // Spawn an async task to avoid blocking the event loop
                                    tokio::spawn(async move {
                                        match browser
                                            .update_and_send_sixel(proxy, term_window_id)
                                            .await
                                        {
                                            Ok(_) => info!("Sixel update sent successfully."),
                                            Err(e) => {
                                                error!("Failed to update and send Sixel: {}", e)
                                            }
                                        }
                                    });
                                },
                                
                                // Toggle fullscreen/split mode with F key
                                rio_window::keyboard::PhysicalKey::Code(KeyCode::KeyF) => {
                                    info!("F pressed: Toggling fullscreen/split mode...");
                                    // Toggle the split mode and store the new value
                                    self.split_mode = !self.split_mode;
                                    let is_split = self.split_mode;
                                    
                                    // Calculate dimensions directly to avoid borrow issues
                                    let (_browser_width, _browser_height, terminal_width, terminal_height) = 
                                        if is_split {
                                            let bw = self.window_width / 2;
                                            let tw = self.window_width - bw;
                                            (bw, self.window_height, tw, self.window_height)
                                        } else {
                                            (0, 0, self.window_width, self.window_height)
                                        };
                                    
                                    // Update term position/size
                                    terminal_pane.resize(terminal_width, terminal_height);
                                    
                                    if is_split {
                                        terminal_pane.window.set_outer_position(rio_window::dpi::PhysicalPosition::new(
                                            self.window_width / 2, 0
                                        ));
                                    } else {
                                        terminal_pane.window.set_outer_position(rio_window::dpi::PhysicalPosition::new(
                                            0, 0
                                        ));
                                    }
                                    
                                    // Notify the user about the mode change
                                    let mode_msg = if is_split {
                                        "\r\nSwitched to split mode (50/50 layout)\r\n"
                                    } else {
                                        "\r\nSwitched to fullscreen terminal mode\r\n"
                                    };
                                    
                                    let mode_payload = RioEvent::PtyWrite(mode_msg.to_string());
                                    self.event_proxy.send_event(RioEventType::Rio(mode_payload), window_id);
                                    
                                    // If we're in split mode, trigger a browser update
                                    if is_split {
                                        let browser = self.browser_pane.clone();
                                        let proxy = self.event_proxy.clone();
                                        let term_window_id = terminal_pane.window.id();
                                        
                                        tokio::spawn(async move {
                                            match browser.update_and_send_sixel(proxy, term_window_id).await {
                                                Ok(_) => info!("Updated Sixel after layout change"),
                                                Err(e) => error!("Failed to update Sixel after layout change: {}", e)
                                            }
                                        });
                                    }
                                },
                                
                                // Navigate to a new URL with N key (would normally use input dialog)
                                rio_window::keyboard::PhysicalKey::Code(KeyCode::KeyN) => {
                                    info!("N pressed: Navigating to a new URL...");
                                    // Implement URL input through terminal prompt
                                    // Show input prompt for URL navigation
                                    let prompt = "\r\n[URL Input Mode] Enter URL: ";
                                    let prompt_payload = RioEvent::PtyWrite(prompt.to_string());
                                    self.event_proxy.send_event(
                                        RioEventType::Rio(prompt_payload),
                                        window_id
                                    );
                                    
                                    // Navigate to GitHub trending
                                    let new_url = "https://github.com/trending";
                                    let browser = self.browser_pane.clone();
                                    let proxy = self.event_proxy.clone();
                                    let term_window_id = terminal_pane.window.id();
                                    
                                    // Write message to terminal about navigation
                                    let info_msg = format!("\r\nNavigating to: {}\r\n", new_url);
                                    let info_payload = RioEvent::PtyWrite(info_msg);
                                    self.event_proxy.send_event(RioEventType::Rio(info_payload), window_id);
                                    
                                    tokio::spawn(async move {
                                        // Use BrowserPane.navigate method
                                        match browser.navigate(new_url).await {
                                            Ok((title, url)) => {
                                                info!("Navigated to {}", new_url);
                                                
                                                // Update browser state with the new title and URL
                                                // Update window title for display
                                                let window_title = format!("Rio Browser - {}: {}", title, url);
                                                
                                                // Update the title in the terminal for user visibility
                                                let title_msg = format!("\r\nPage loaded: {} - {}\r\n", title, url);
                                                let title_payload = RioEvent::PtyWrite(title_msg);
                                                proxy.send_event(RioEventType::Rio(title_payload), term_window_id);
                                                
                                                // Update window title via terminal notification
                                                // Send the title info as a special terminal sequence that can be
                                                // intercepted by the terminal for window title updates
                                                let title_seq = format!("\x1b]0;{}\x07", window_title);
                                                let title_event = RioEvent::PtyWrite(title_seq);
                                                proxy.send_event(RioEventType::Rio(title_event), term_window_id);
                                                
                                                // After navigation, update the Sixel
                                                match browser.update_and_send_sixel(proxy, term_window_id).await {
                                                    Ok(_) => info!("Updated Sixel after navigation"),
                                                    Err(e) => error!("Failed to update Sixel after navigation: {}", e)
                                                }
                                            },
                                            Err(e) => error!("Navigation failed: {}", e)
                                        }
                                    });
                                },
                                _ => {}
                            }
                        } else {
                            // Handle key release events if needed
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        // CRITICAL: This is where terminal rendering actually happens
                        info!("Rendering terminal due to RedrawRequested");
                        terminal_pane.render();
                        
                        // Trigger a render event for the terminal
                        // This ensures the terminal grid state is properly updated
                        let render_event = RioEvent::Render;
                        self.event_proxy.send_event(RioEventType::Rio(render_event), window_id);
                    }
                    _ => {}
                }
            } else {
                // Event for a window we don't manage? Should not happen in single-window setup.
                info!("Received event for unknown window: {:?}", window_id);
            }
        } else {
            // Terminal pane not initialized yet, ignore event.
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Default behavior: Wait for next event
        event_loop.set_control_flow(ControlFlow::Wait);
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        info!("Event loop exiting - cleaning up resources");
        
        // Clean up terminal resources
        if let Some(terminal_pane) = self.terminal_pane.as_mut() {
            info!("Cleaning up terminal resources");
            terminal_pane.cleanup();
        }
        
        info!("Application cleanup completed");
    }

    // Implement other ApplicationHandler methods as needed (user_event, new_events, etc.)
    fn new_events(&mut self, event_loop: &ActiveEventLoop, _cause: StartCause) {
        // Default: Wait
        event_loop.set_control_flow(ControlFlow::Wait);
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: EventPayload) {
        // Process Rio events
        match event.payload {
            RioEventType::Rio(RioEvent::PtyWrite(text)) => {
                if let Some(terminal_pane) = self.terminal_pane.as_mut() {
                    // Log for debugging
                    tracing::trace!("PtyWrite event received: {} bytes", text.len());
                    
                    // Process the bytes received from the PTY reader thread
                    // This updates the terminal grid via the Crosswords component
                    if let Err(e) = terminal_pane.handle_pty_output(text.as_bytes()) {
                        error!("Failed to process PTY output: {}", e);
                    }
                    
                    // Important: Explicitly request a redraw after processing
                    // This ensures the terminal content updates are rendered
                    terminal_pane.window.request_redraw();
                }
            }
            RioEventType::Rio(RioEvent::Render) => {
                if let Some(terminal_pane) = self.terminal_pane.as_mut() {
                    terminal_pane.render(); // Call render directly
                    terminal_pane.window.request_redraw();
                }
            }
            RioEventType::Rio(event) => {
                // Process other Rio events that need to be handled
                tracing::debug!("Other Rio event received: {:?}", event);
                
                if let Some(terminal_pane) = self.terminal_pane.as_mut() {
                    // Let the terminal pane handle other events if needed
                    // Request redraw to ensure changes are visible
                    terminal_pane.window.request_redraw();
                }
            }
            _ => {} // Handle other event types if necessary
        }
    }
}
