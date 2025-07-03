pub mod claude_desktop;
pub mod windsurf;
pub mod cursor;
pub mod zed;
pub mod roo_code;

use crate::ClientConfigPlugin;
use std::sync::Arc;

/// Get all available client plugins
pub fn all_clients() -> Vec<Arc<dyn ClientConfigPlugin>> {
    vec![
        Arc::new(claude_desktop::ClaudeDesktopPlugin),
        Arc::new(windsurf::WindsurfPlugin),
        Arc::new(cursor::CursorPlugin),
        Arc::new(zed::ZedPlugin),
        Arc::new(roo_code::RooCodePlugin),
    ]
}