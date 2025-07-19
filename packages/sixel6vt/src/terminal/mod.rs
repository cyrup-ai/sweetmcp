/// TerminalUtils provides OSC sequence support for terminal integration
pub struct TerminalUtils;

impl TerminalUtils {
    /// Generate OSC sequence for setting the working directory
    pub fn osc_set_working_dir(hostname: &str, path: &str) -> String {
        // Import the type locally where needed
        use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

        let url = format!(
            "file://{}/{}",
            hostname,
            utf8_percent_encode(path, NON_ALPHANUMERIC)
        );
        format!("\x1b]7;{}\x07", url)
    }

    /// Generate OSC sequence for setting the document
    pub fn osc_set_document(hostname: &str, path: &str) -> String {
        // Import the type locally where needed
        use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

        let url = format!(
            "file://{}/{}",
            hostname,
            utf8_percent_encode(path, NON_ALPHANUMERIC)
        );
        format!("\x1b]6;{}\x07", url)
    }

    /// Generate OSC sequence for creating a hyperlink
    pub fn osc_set_hyperlink(url: &str, text: &str) -> String {
        format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, text)
    }
}
