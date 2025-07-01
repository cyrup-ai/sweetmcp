use serde::{Deserialize, Serialize};

/// Browser command types that can be executed by the host
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BrowserCommand {
    Navigate(NavigateCommand),
    Screenshot(ScreenshotCommand),
    Click(ClickCommand),
    TypeText(TypeTextCommand),
    ExtractText(ExtractTextCommand),
    Scroll(ScrollCommand),
    Wait(WaitCommand),
    RunAutomation(RunAutomationCommand),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigateCommand {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotCommand {
    pub element_selector: Option<String>,
    pub format: ScreenshotFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ScreenshotFormat {
    #[default]
    Base64,
    Png,
    Jpeg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickCommand {
    pub selector: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeTextCommand {
    pub selector: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractTextCommand {
    pub selector: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollCommand {
    pub direction: ScrollDirection,
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ScrollDirection {
    Up,
    #[default]
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitCommand {
    pub duration: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunAutomationCommand {
    pub task: String,
    pub use_vision: bool,
    pub additional_info: String,
}

/// Command execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}
