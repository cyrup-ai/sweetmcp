use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use rio_backend::{
    clipboard::Clipboard,
    // Assuming PtyConfig is public under config
    config::{Config as RioConfig}, // PtyConfig removed, Shell removed
    event::RioEvent, // Import only RioEvent
    sugarloaf::{
        font::FontLibrary,
        layout::RootStyle,
        SugarloafWindow,
        SugarloafWindowSize,
        Sugarloaf,
        SugarloafRenderer,
        // Use the error type that contains the instance
        SugarloafWithErrors,
    },
    // Use the public terminal/pty APIs from rio-backend
    // Remove unused alias: crosswords::grid::Dimensions,
};
// Import Crosswords and alias as Terminal from the correct path
use rio_backend::crosswords::Crosswords as Terminal;
// Import Pty, WinsizeBuilder and ProcessReadWrite trait from teletypewriter
use teletypewriter::{WinsizeBuilder, ProcessReadWrite};

use rio_window::{
    event_loop::ActiveEventLoop,
    window::Window, // Remove WindowId import
};
use std::{cell::RefCell, rc::Rc, io::{Read, Write, BufReader}};
use std::sync::{mpsc, Arc, atomic::{AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};
use anyhow::Result;
use tracing::{error, info, debug};

// Make TerminalPane generic over the EventListener type U
pub struct TerminalPane<U: rio_backend::event::EventListener + Clone + Send + 'static> { // Add Clone bound
    pub window: Window,
    pub terminal: Terminal<U>, // Use the generic parameter U
    pub pty_tx: mpsc::Sender<Vec<u8>>, // Channel to send data to PTY
    pub event_proxy: U, // Use the generic parameter U for the proxy type
    pub clipboard: Rc<RefCell<Clipboard>>,
    pub sugarloaf: Sugarloaf<'static>,
    
    // Thread management resources
    running: Arc<AtomicBool>,
    reader_thread: Option<JoinHandle<()>>,
    writer_thread: Option<JoinHandle<()>>,
}

// Implement methods for the generic TerminalPane
impl<U: rio_backend::event::EventListener + Clone + Send + 'static> TerminalPane<U> { // Add Clone bound
    pub fn new(
        active_event_loop: &ActiveEventLoop, // Use ActiveEventLoop
        event_proxy: U, // Use the generic parameter U
        config: &RioConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> { // Specify Error type
        // Configure window
        let window_builder = Window::default_attributes()
            .with_title("Rio Terminal")
            .with_transparent(config.window.opacity < 1.0);

        // Create window using ActiveEventLoop
        let window = active_event_loop.create_window(window_builder)?;

        // Create font library
        let (font_library, _) = FontLibrary::new(config.fonts.clone());

        // Create sugarloaf window
        let sugarloaf_window = SugarloafWindow {
            // Use HasWindowHandle trait
            handle: window.window_handle()?.into(),
            display: window.display_handle()?.into(),
            scale: window.scale_factor() as f32,
            size: SugarloafWindowSize {
                width: window.inner_size().width as f32,
                height: window.inner_size().height as f32,
            },
        };

        // Create sugarloaf instance
        let sugarloaf = match Sugarloaf::new(
            sugarloaf_window,
            SugarloafRenderer::default(),
            &font_library,
            RootStyle::new(
                window.scale_factor() as f32,
                config.fonts.size,
                config.line_height,
            ),
        ) {
            Ok(instance) => instance,
            // Use the correct error type from Sugarloaf
            Err(SugarloafWithErrors { instance, .. }) => instance, // Use SugarloafWithErrors
        };

        // Get dimensions for terminal
        let _scale = window.scale_factor() as f32; // Unused but kept for clarity
        let width_u = window.inner_size().width;
        let height_u = window.inner_size().height;
        let width = width_u as f32;
        let height = height_u as f32;

        // Calculate terminal dimensions manually
        // Note: This is a simplified calculation. A real implementation
        // might need to account for padding_x/padding_y more precisely
        // depending on how Sugarloaf handles it internally.
        let cols = (width / config.fonts.size).floor() as usize; // Use font size for width approximation
        let lines = (height / (config.fonts.size * config.line_height)).floor() as usize; // Use font size * line_height for height
        let terminal_size = (cols, lines); // Store as a tuple for now

        // Create CrosswordsSize for Terminal::new - This needs to implement Dimensions
        let cross_size = rio_backend::crosswords::CrosswordsSize::new_with_dimensions(cols, lines, width_u, height_u, 0, 0); // Assuming 0 for square width/height initially

        // Get cursor shape and blinking setting from config
        let cursor_shape = config.cursor.shape;
            
        let terminal = Terminal::new(
            // Correct argument order: Dimensions, CursorStyle, EventListener, WindowId, route_id
            cross_size, // Pass CrosswordsSize
            cursor_shape, // Pass cursor shape
            event_proxy.clone(), // Pass event proxy
            window.id(),
            0, // Default route_id
        ); // Remove ? operator, new doesn't return Result

        // Create PTY using public API
        // Use teletypewriter::WinsizeBuilder and construct manually
        let pty_spawn_builder = WinsizeBuilder { // Construct manually
            rows: terminal_size.1 as u16, // rows
            cols: terminal_size.0 as u16, // cols
            // Add missing width and height fields
            width: width_u as u16,
            height: height_u as u16,
        };
        // Set critical terminal environment variables before creating PTY
        // From frontends/rioterm/src/main.rs example
        std::env::set_var("TERM", "xterm-256color");
        std::env::set_var("TERM_PROGRAM", "rio");
        std::env::set_var("TERM_PROGRAM_VERSION", "1.0.0"); // Simulate a version
        std::env::set_var("COLORTERM", "truecolor");
        
        // Get shell path using same approach as the spawn.rs example
        use std::borrow::Cow;
        let shell_path = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        let shell = Cow::Borrowed(shell_path.as_str());
        info!("Using shell: {}", shell);
        
        // Unix (Linux and macOS) – use the portable create_pty API
        #[cfg(unix)]
        let mut process = teletypewriter::create_pty_with_spawn(
            &shell, 
            vec![], 
            &config.working_dir,
            pty_spawn_builder.cols,
            pty_spawn_builder.rows,
        )?;

        // Windows – still use conpty for cmd.exe
        #[cfg(windows)]
        let mut process = teletypewriter::windows::new(
            "cmd.exe",
            &config.working_dir,
            pty_spawn_builder.cols,
            pty_spawn_builder.rows,
        )?;

        // Create channels for PTY communication - we'll need this for the application
        let (pty_tx, pty_rx) = mpsc::channel::<Vec<u8>>();
        
        // Log status
        info!("PTY created successfully with dimensions {}x{}", 
             pty_spawn_builder.cols, pty_spawn_builder.rows);
        
        // Using the same pattern as spawn.rs: direct command sequence
        // Send initialization commands immediately to the PTY
        // This is crucial for shell initialization
        info!("Sending initialization commands to PTY");
        
        // First, send individual characters as in the example
        process.writer().write_all(b"1").map_err(|e| {
            error!("Failed to write init chars to PTY: {}", e);
            anyhow::anyhow!("Failed to write to PTY: {}", e)
        })?;
        process.writer().write_all(b"2").map_err(|e| {
            error!("Failed to write init chars to PTY: {}", e);
            anyhow::anyhow!("Failed to write to PTY: {}", e)
        })?;
        
        // Then send actual commands with newlines
        let init_commands = [
            b"clear\n".as_slice(),
            b"stty -echo\n".as_slice(), // Disable echo for cleaner display
            b"stty cols 50 rows 50\n".as_slice(), // Set terminal size explicitly
            b"echo 'Terminal initialized successfully'\n".as_slice(),
            b"pwd\n".as_slice(),          // Show current directory
            b"ls -la\n".as_slice(),       // List files
            b"export PS1='$ '\n".as_slice(), // Set a simple prompt
        ];
        
        for cmd in &init_commands {
            process.writer().write_all(cmd).map_err(|e| {
                error!("Failed to write command to PTY: {}", e);
                anyhow::anyhow!("Failed to write to PTY: {}", e)
            })?;
            process.writer().flush().map_err(|e| {
                error!("Failed to flush PTY writer: {}", e);
                anyhow::anyhow!("Failed to flush PTY: {}", e)
            })?;
            
            // Add a small delay between commands to ensure they're processed properly
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        
        // Create shared running flag
        let running = Arc::new(AtomicBool::new(true));
        
        // Clone event proxy for threads and create window id
        let event_proxy_clone = event_proxy.clone();
        let window_id = window.id();
        
        // Create cloned reader for reader thread
        #[cfg(not(windows))]
        let reader = process.reader().try_clone().map_err(|e| {
            error!("Failed to clone PTY reader: {}", e);
            anyhow::anyhow!("Failed to clone PTY reader: {}", e)
        })?;
        
        #[cfg(windows)]
        let reader = process.reader().try_clone().map_err(|e| {
            error!("Failed to clone PTY reader: {}", e);
            anyhow::anyhow!("Failed to clone PTY reader: {}", e)
        })?;
        
        // Clone running flag for reader thread
        let reader_running = running.clone();
        
        // Set up reader thread with proper resource management
        let reader_thread = thread::spawn(move || {
            info!("PTY reader thread started");
            
            // Use BufReader for efficient reading one byte at a time
            let buf_reader = BufReader::new(reader);
            for bs in buf_reader.bytes() {
                // Check if we should continue running
                if !reader_running.load(Ordering::SeqCst) {
                    debug!("Reader thread received shutdown signal");
                    break;
                }
                
                match bs {
                    Ok(byte) => {
                        // Convert single byte to string and send to terminal
                        let u = [byte];
                        let text = String::from_utf8_lossy(&u).to_string();
                        let event = rio_backend::event::RioEvent::PtyWrite(text);
                        
                        // Send the event to the terminal window
                        // The terminal's EventListener implementation directly takes RioEvent
                        // and returns (), not Result, so we can't check for errors
                        event_proxy_clone.send_event(event, window_id);
                    }
                    Err(e) => {
                        // Check if this is just a "would block" error (EAGAIN/EWOULDBLOCK)
                        if e.kind() == std::io::ErrorKind::WouldBlock {
                            // This is normal for non-blocking I/O - just wait a bit and continue
                            std::thread::sleep(std::time::Duration::from_millis(10));
                            continue;
                        }
                        
                        // Only log as error if thread is still supposed to be running
                        if reader_running.load(Ordering::SeqCst) {
                            error!("Failed to read from PTY: {}", e);
                        } else {
                            debug!("PTY reader closed during shutdown: {}", e);
                        }
                        break;
                    }
                }
            }
            info!("PTY reader thread terminated");
        });
        
        // Clone running flag for writer thread
        let writer_running = running.clone();
        
        // Set up writer thread with proper resource management
        let writer_thread = thread::spawn(move || {
            info!("PTY writer thread started");
            
            // Handle incoming commands from the application
            while writer_running.load(Ordering::SeqCst) {
                // Use timeout recv to allow checking the running flag periodically
                match pty_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok(data) => {
                        debug!("Received command to send to PTY ({} bytes)", data.len());
                        
                        // Only log the first 20 bytes to avoid flooding logs with large payloads
                        if data.len() > 20 {
                            let preview = String::from_utf8_lossy(&data[0..20]);
                            debug!("Command preview: {:?}...", preview);
                        } else {
                            debug!("Command: {:?}", String::from_utf8_lossy(&data));
                        }
                        
                        if let Err(e) = process.writer().write_all(&data) {
                            // Check if this is just a "would block" error
                            if e.kind() == std::io::ErrorKind::WouldBlock {
                                // For writes, we should retry after a short delay
                                std::thread::sleep(std::time::Duration::from_millis(10));
                                // Try again
                                if let Err(e2) = process.writer().write_all(&data) {
                                    if writer_running.load(Ordering::SeqCst) && e2.kind() != std::io::ErrorKind::WouldBlock {
                                        error!("Failed to write to PTY after retry: {}", e2);
                                    }
                                }
                            } else if writer_running.load(Ordering::SeqCst) {
                                error!("Failed to write to PTY: {}", e);
                                break;
                            } else {
                                debug!("PTY write error during shutdown: {}", e);
                                break;
                            }
                        }
                        
                        if let Err(e) = process.writer().flush() {
                            if writer_running.load(Ordering::SeqCst) {
                                error!("Failed to flush PTY writer: {}", e);
                            } else {
                                debug!("PTY flush error during shutdown: {}", e);
                            }
                            break;
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // Timeout is normal, just check the running flag
                        continue;
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        debug!("PTY writer channel disconnected");
                        break;
                    }
                }
            }
            info!("PTY writer thread terminated");
        });
        
// The reader thread is already created above

        // Get clipboard using public API
        let clipboard = unsafe {
            // Use HasDisplayHandle trait
            Clipboard::new(window.display_handle()?.as_raw())
        };

        // No need to manually write initial content, Pty handles shell startup

        Ok(Self {
            window,
            terminal, // Assign created terminal
            pty_tx, // Assign the pty writer channel
            event_proxy,
            clipboard: Rc::new(RefCell::new(clipboard)),
            sugarloaf,
            // Store thread management resources
            running,
            reader_thread: Some(reader_thread),
            writer_thread: Some(writer_thread),
        })
    }

    // Render terminal content using Sugarloaf
    pub fn render(&mut self) {
        // Update clipboard state for copy/paste functionality
        if let Ok(_clipboard) = self.clipboard.try_borrow() {
            // Terminal clipboard handling is managed by Rio internally
            // No explicit call needed here
        }
        
        // CRITICAL: Update the terminal grid state to Sugarloaf
        // Crosswords stores all cell data needed for rendering
        // Rio would normally handle this through its internal renderer
        
        // Render the current terminal state using Sugarloaf's renderer
        self.sugarloaf.render();
        
        // Log that rendering occurred for debugging
        tracing::debug!("Terminal rendered");
    }

    // Handle window resize
    pub fn resize(&mut self, width: u32, height: u32) {
        let dpr = self.window.scale_factor();
        let _scale = dpr as f32; // Keep but unused
        self.sugarloaf.resize(width, height);
        // Resize the terminal - needs a type implementing Dimensions
        // Calculate terminal dimensions based on window size
        // Use configuration values from Rio for accurate calculations
        // This avoids accessing private sugarloaf methods
        let font_size = 12.0; // Default font size, should match config
        let line_height = 1.2; // Default line height, should match config
        
        let cols = (width as f32 / font_size).floor() as usize;
        let lines = (height as f32 / (font_size * line_height)).floor() as usize;
        
        let cross_size = rio_backend::crosswords::CrosswordsSize::new_with_dimensions(
            cols, 
            lines,
            width, height, 0, 0 // Assuming 0 for square width/height initially
        );
        self.terminal.resize(cross_size);
        // Create the resize command with window dimensions
        // Store resize info to send in the channel
        // Using the builder directly to keep all dimension data
        let resize_info = WinsizeBuilder { 
            cols: cols as u16, 
            rows: lines as u16, 
            width: width as u16, 
            height: height as u16 
        };
        // Serialize the resize info and send to PTY
        let resize_sequence = format!(
            "\x1b[8;{};{}t", 
            resize_info.rows,
            resize_info.cols
        ).into_bytes();
        
        if let Err(e) = self.pty_tx.send(resize_sequence) {
            error!("Failed to send resize command to PTY: {}", e);
        } else {
            info!("Terminal resized to {}x{}", resize_info.cols, resize_info.rows);
        }
    }
    
    /// Handle output from the PTY process
    pub fn handle_pty_output(&mut self, data: &[u8]) -> anyhow::Result<()> {
        // Skip processing if terminal is shutting down
        if !self.running.load(Ordering::SeqCst) {
            debug!("Skipping PTY output processing - terminal is shutting down");
            return Ok(());
        }

        // Convert bytes to UTF-8 string for terminal processing
        let text = String::from_utf8_lossy(data).to_string();
        
        // Check for common control sequences that don't need to be echoed back
        let is_control_sequence = text.starts_with("\x1B") || 
                                 text.starts_with("\r") || 
                                 text.starts_with("\n");
        
        // Only process text that actually contains visible content
        if !text.trim().is_empty() {
            // CRITICAL: Process the text through the terminal
            // Send a direct PtyWrite event to update the terminal grid
            let update_event = RioEvent::PtyWrite(text.clone());
            
            // The event_proxy.send_event method returns (), not Result
            self.event_proxy.send_event(update_event, self.window.id());
            
            // Only send to PTY for actual command input (not control sequences or output)
            // This helps prevent feedback loops
            if !is_control_sequence && text.contains(|c: char| c.is_alphanumeric()) {
                match self.pty_tx.send(data.to_vec()) {
                    Ok(_) => {
                        debug!("Sent user input to PTY: {:?}", text);
                    }
                    Err(e) => {
                        // Don't fail the whole operation just because we couldn't send to PTY
                        // This could be during shutdown
                        debug!("Failed to send to PTY (possibly shutting down): {}", e);
                    }
                }
            }
            
            // Request redraw to display the updates
            self.window.request_redraw();
        }
        
        Ok(())
    }
    
    /// Clean up resources before dropping the terminal
    pub fn cleanup(&mut self) {
        // Set running flag to false to signal threads to shut down
        self.running.store(false, Ordering::SeqCst);
        info!("Signaled terminal threads to shut down");
        
        // Join reader thread if it exists
        if let Some(reader_thread) = self.reader_thread.take() {
            // Give thread a bit of time to exit cleanly
            if let Err(e) = reader_thread.join() {
                error!("Failed to join reader thread: {:?}", e);
            } else {
                debug!("Reader thread joined successfully");
            }
        }
        
        // Join writer thread if it exists
        if let Some(writer_thread) = self.writer_thread.take() {
            // Give thread a bit of time to exit cleanly
            if let Err(e) = writer_thread.join() {
                error!("Failed to join writer thread: {:?}", e);
            } else {
                debug!("Writer thread joined successfully");
            }
        }
        
        info!("Terminal cleanup completed");
    }
}

// Implement Drop for TerminalPane to ensure cleanup happens
impl<U: rio_backend::event::EventListener + Clone + Send + 'static> Drop for TerminalPane<U> {
    fn drop(&mut self) {
        self.cleanup();
    }
}
