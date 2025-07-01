// bring in Tokio and error‐handling
use anyhow::Result;
use tokio::main;

// bring in Rio's config & event APIs
use rio_backend::config::Config as RioConfig;
use rio_backend::event::EventProxy;

// bring in the windowing entry‐point
use rio_window::event_loop::EventLoop;

// your application and its pieces
use crate::components::app::Application;
use crate::components::browser::BrowserPane;
use std::sync::Arc;

// your modules
mod browser;
mod components;
mod renderer;
mod sixel;
mod sixel_region;
mod terminal;

#[main]
async fn main() -> Result<()> {
    // initialize tracing/logging
    tracing_subscriber::fmt::init();

    // load Rio config (or default)
    // Config::load returns a Config, not a Result
    let config = RioConfig::load();

    // prime the browser pane (headless navigation)
    // pick your URL from RIO_START_URL or fall back to a default you control
    let start_url = std::env::var("RIO_START_URL")
        .unwrap_or_else(|_| "https://github.com/trendng/rust".to_string());
    let browser_pane = Arc::new(BrowserPane::new(&start_url).await?);

    // create winit EventLoop + its proxy, then wrap
    // EventLoop::with_user_event() creates an EventLoopBuilder that we need to build()
    let event_loop = EventLoop::with_user_event().build()?;
    let proxy = event_loop.create_proxy();
    let event_proxy = EventProxy::new(proxy);

    // instantiate your Application
    let mut app = Application::new(config, browser_pane.clone(), event_proxy.clone());

    // hand control (and the EventLoop) to Rio
    event_loop.run_app(&mut app)?;
    Ok(())
}
