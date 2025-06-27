use std::{
    io::{Stdout, stdout},
    time::Duration,
};

use anyhow::Result;
use clap::Subcommand;
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};
use uuid::Uuid;

use crate::{
    resource::cms::cms_dao,
    sampling::{CreateMessageRequest, McpMessage, model::McpMessageContent},
};

#[derive(Subcommand, Debug)]
pub enum HealthCommands {
    /// Check MCP functionality
    #[command(subcommand)]
    Check(CheckCommands),
}

#[derive(Subcommand, Debug)]
pub enum CheckCommands {
    /// Check resources functionality
    Resources,

    /// Check sampling functionality
    Sampling,
}

pub struct HealthCheck {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    results: Vec<CheckResult>,
}

pub struct CheckResult {
    id: String,
    name: String,
    status: CheckStatus,
    message: String,
}

#[derive(Clone, PartialEq)]
pub enum CheckStatus {
    Success,
    Failure,
    Running,
    Pending,
}

impl HealthCheck {
    pub fn new() -> Result<Self> {
        // Setup terminal
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        Ok(Self {
            terminal,
            results: Vec::new(),
        })
    }

    pub fn cleanup(&mut self) -> Result<()> {
        // Restore terminal
        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }

    pub fn add_result(&mut self, name: &str, status: CheckStatus, message: &str) {
        let result = CheckResult {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            status,
            message: message.to_string(),
        };
        self.results.push(result);
    }

    pub fn render(&mut self) -> Result<()> {
        self.terminal.draw(|f| {
            let size = f.area();

            // Title block
            let title_block = Block::default()
                .title("MCP Health Check")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL);

            let title = Paragraph::new("Press 'q' to exit")
                .block(title_block)
                .alignment(Alignment::Center);

            let title_area = Rect::new(0, 0, size.width, 3);
            f.render_widget(title, title_area);

            // Results table
            let header_cells = ["ID", "Check", "Status", "Message"]
                .iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
            let header = Row::new(header_cells).height(1);

            let rows = self.results.iter().map(|result| {
                let status_style = match result.status {
                    CheckStatus::Success => Style::default().fg(Color::Green),
                    CheckStatus::Failure => Style::default().fg(Color::Red),
                    CheckStatus::Running => Style::default().fg(Color::Yellow),
                    CheckStatus::Pending => Style::default().fg(Color::Gray),
                };

                let status_text = match result.status {
                    CheckStatus::Success => "✓ Success",
                    CheckStatus::Failure => "✗ Failure",
                    CheckStatus::Running => "⟳ Running",
                    CheckStatus::Pending => "⋯ Pending",
                };

                let cells = [
                    Cell::from(result.id.clone().split_at(8).0.to_string()),
                    Cell::from(result.name.clone()),
                    Cell::from(status_text).style(status_style),
                    Cell::from(result.message.clone()),
                ];
                Row::new(cells).height(1)
            });

            let table = Table::new(
                rows,
                [
                    Constraint::Percentage(10),
                    Constraint::Percentage(20),
                    Constraint::Percentage(15),
                    Constraint::Percentage(55),
                ],
            )
            .header(header)
            .block(Block::default().title("Test Results").borders(Borders::ALL))
            .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

            let table_area = Rect::new(0, 3, size.width, size.height - 3);
            f.render_widget(table, table_area);
        })?;

        Ok(())
    }

    pub fn handle_events(&mut self) -> Result<bool> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    return Ok(true); // Exit
                }
            }
        }
        Ok(false)
    }
}

use futures::StreamExt;

use crate::sampling::sampling_create_message;

pub fn check_resources() -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;

    // Use block_on to create a resources stream and then consume it
    let mut found = false;
    let run_result: Result<(), anyhow::Error> = rt.block_on(async {
        let stream = cms_dao::resources_list(None);
        tokio::pin!(stream);
        while let Some(result) = stream.next().await {
            // Handle the result inside the async block
            match result {
                Ok(cms_node) => {
                    println!("CMS Node: {} ({})", cms_node.name, cms_node.uri);
                    found = true;
                }
                Err(e) => {
                    eprintln!("Error fetching CMS node: {}", e);
                    // Decide if you want to stop or continue on error
                    // return Err(anyhow::anyhow!("Error fetching CMS node: {}", e)); // Example:
                    // Stop on error
                }
            }
        }
        Ok(()) // Return Ok if the loop completes
    });
    run_result?; // Propagate error from the async block if any
    if !found {
        println!("No CMS nodes found.");
    }
    Ok(())
}

pub fn check_sampling() -> Result<()> {
    let request = CreateMessageRequest {
        messages: vec![McpMessage {
            role: "user".to_string(),
            content: McpMessageContent {
                type_: "text".to_string(),
                text: Some("Hello, world!".to_string()),
                data: None,
                mime_type: None,
            },
        }],
        system_prompt: Some("You are a helpful AI.".to_string()),
        model_preferences: None,
        include_context: None,
        max_tokens: Some(16),
        temperature: Some(0.7),
        stop_sequences: None,
        metadata: None,
        meta: None,
    };
    let fut = sampling_create_message(request);
    let rt = tokio::runtime::Runtime::new()?;
    let result = rt.block_on(fut)?;
    println!("Sampling result: {:?}", result);
    Ok(())
}

// Add this function at the end of src/ui/health/mod.rs
pub fn handle_health_command(cmd: &HealthCommands) -> Result<()> {
    match cmd {
        HealthCommands::Check(check_cmd) => match check_cmd {
            CheckCommands::Resources => {
                check_resources()?;
            }
            CheckCommands::Sampling => {
                check_sampling()?;
            }
        },
    }
    Ok(())
}
