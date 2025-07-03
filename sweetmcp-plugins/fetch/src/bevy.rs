use async_trait::async_trait;
use base64::Engine;
use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages},
    window::PrimaryWindow,
    ui::{Style, Val, UiRect, node_bundles::{NodeBundle, TextBundle}},
    text::TextStyle,
};
use image::{DynamicImage, ImageBuffer, Rgba};
use std::error::Error as StdError;
use std::fmt;
use std::sync::{Arc, Mutex};

use crate::chromiumoxide::{ContentFetcher, FetchResult};

#[derive(Debug)]
pub enum BevyRenderError {
    Setup(String),
    Render(String),
    Screenshot(String),
    Content(String),
}

impl fmt::Display for BevyRenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BevyRenderError::Setup(e) => write!(f, "Setup error: {}", e),
            BevyRenderError::Render(e) => write!(f, "Render error: {}", e),
            BevyRenderError::Screenshot(e) => write!(f, "Screenshot error: {}", e),
            BevyRenderError::Content(e) => write!(f, "Content error: {}", e),
        }
    }
}

impl StdError for BevyRenderError {}

// Resource to store the HTML content
#[derive(Resource)]
struct HtmlContent(String);

// Resource to store the screenshot
#[derive(Resource)]
struct Screenshot(Option<DynamicImage>);

// Flag to indicate when rendering is complete
#[derive(Resource)]
struct RenderComplete(bool);

// Mutable state to be shared between Bevy and the main thread
struct SharedState {
    content: Option<String>,
    screenshot_base64: Option<String>,
}

pub struct BevyRenderer;

impl BevyRenderer {
    fn setup_bevy_app(html_content: String) -> Result<String, BevyRenderError> {
        let shared_state = Arc::new(Mutex::new(SharedState {
            content: None,
            screenshot_base64: None,
        }));
        
        let shared_state_clone = shared_state.clone();

        // Create and run the Bevy app
        let mut app = App::new();
        
        app.insert_resource(HtmlContent(html_content))
            .insert_resource(Screenshot(None))
            .insert_resource(RenderComplete(false))
            .add_plugins(DefaultPlugins.set(
                WindowPlugin {
                    primary_window: Some(Window {
                        title: "Web Content Renderer".to_string(),
                        resolution: (1280., 800.).into(),
                        visible: false, // Headless rendering
                        ..default()
                    }),
                    ..default()
                }
            ))
            .add_systems(Startup, setup_renderer_system)
            .add_systems(Update, (render_system, screenshot_system, check_completion_system));

        // Run the app with a custom runner that will terminate when rendering is complete
        app.add_systems(Update, move |render_complete: Res<RenderComplete>, 
                                    screenshot: Res<Screenshot>,
                                    html_content: Res<HtmlContent>| {
            if render_complete.0 {
                if let Some(image) = &screenshot.0 {
                    // Convert image to base64
                    let mut buffer = std::io::Cursor::new(Vec::new());
                    if let Err(e) = image.write_to(&mut buffer, image::ImageOutputFormat::Png) {
                        eprintln!("Failed to encode screenshot: {}", e);
                    } else {
                        let base64_image = base64::engine::general_purpose::STANDARD.encode(buffer.into_inner());
                        match shared_state_clone.lock() {
                            Ok(mut state) => {
                                state.content = Some(html_content.0.clone());
                                state.screenshot_base64 = Some(base64_image);
                            }
                            Err(e) => {
                                eprintln!("Failed to lock shared state: {}", e);
                            }
                        }
                }
                std::process::exit(0);
            }
        });

        app.run();

        // Extract the results from the shared state
        let state = match shared_state.lock() {
            Ok(state) => state,
            Err(e) => return Err(format!("Failed to lock shared state: {}", e)),
        };
        
        if let Some(screenshot_base64) = &state.screenshot_base64 {
            Ok(screenshot_base64.clone())
        } else {
            Err(BevyRenderError::Screenshot("Failed to capture screenshot".to_string()))
        }
    }
}

// System to set up the renderer
fn setup_renderer_system(mut commands: Commands) {
    // Add a 2D camera
    commands.spawn(Camera2dBundle::default());
}

// System to render the HTML content
fn render_system(html_content: Res<HtmlContent>, mut commands: Commands, asset_server: Res<AssetServer>) {
    // In a real implementation, this would parse and render the HTML
    // For this example, we'll just create a simple UI element
    
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        background_color: Color::WHITE.into(),
        ..default()
    }).with_children(|parent| {
        // Add some text to represent the rendered content
        parent.spawn(TextBundle::from_section(
            format!("Rendered content (sample): {}", &html_content.0[0..min(50, html_content.0.len())]),
            TextStyle {
                font_size: 20.0,
                color: Color::BLACK,
                ..default()
            },
        ));
    });
}

// System to take a screenshot
fn screenshot_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
    mut screenshot: ResMut<Screenshot>,
    render_complete: Res<RenderComplete>,
) {
    if !render_complete.0 {
        return;
    }

    if let Ok(window) = windows.get_single() {
        // Create a new image
        let size = Extent3d {
            width: window.width() as u32,
            height: window.height() as u32,
            depth_or_array_layers: 1,
        };

        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            },
            ..default()
        };

        // Fill with a simple gradient (in a real implementation, this would be the rendered content)
        let mut buffer: Vec<u8> = Vec::with_capacity((size.width * size.height * 4) as usize);
        for y in 0..size.height {
            for x in 0..size.width {
                let r = (x as f32 / size.width as f32 * 255.0) as u8;
                let g = (y as f32 / size.height as f32 * 255.0) as u8;
                let b = 128;
                let a = 255;
                buffer.extend_from_slice(&[r, g, b, a]);
            }
        }
        image.data = buffer;

        // Convert to DynamicImage
        let img_buffer = match ImageBuffer::<Rgba<u8>, _>::from_raw(
            size.width,
            size.height,
            image.data.clone(),
        ) {
            Some(buffer) => buffer,
            None => {
                eprintln!("Failed to create image buffer from screenshot data");
                return;
            }
        };
        
        let dynamic_image = DynamicImage::ImageRgba8(img_buffer);
        screenshot.0 = Some(dynamic_image);
    }
}

// System to check when rendering is complete
fn check_completion_system(mut render_complete: ResMut<RenderComplete>) {
    // In a real implementation, this would check if the content is fully rendered
    // For this example, we'll just set it to true after one frame
    render_complete.0 = true;
}

fn min(a: usize, b: usize) -> usize {
    if a < b { a } else { b }
}

#[async_trait]
impl ContentFetcher for BevyRenderer {
    async fn fetch_content(&self, url: &str) -> Result<FetchResult, Box<dyn StdError + Send + Sync>> {
        // First, fetch the content using Hyper
        let html_content = crate::hyper::HyperFetcher::fetch(url)
            .await
            .map_err(|e| BevyRenderError::Content(format!("Failed to fetch content: {}", e)))?;
        
        // Clean the HTML (remove scripts and styles)
        let cleaned_html = crate::hyper::HyperFetcher::clean_html(&html_content);
        let cleaned_html_clone = cleaned_html.clone();
        
        // Render the content and get a screenshot
        let screenshot_base64 = tokio::task::spawn_blocking(move || {
            BevyRenderer::setup_bevy_app(cleaned_html_clone)
        })
        .await
        .map_err(|e| BevyRenderError::Setup(format!("Failed to spawn rendering task: {}", e)))?
        .map_err(|e| BevyRenderError::Render(format!("Failed to render content: {}", e)))?;
        
        Ok(FetchResult {
            content: cleaned_html,
            screenshot_base64,
            content_type: "text/html".to_string(),
        })
    }
}
