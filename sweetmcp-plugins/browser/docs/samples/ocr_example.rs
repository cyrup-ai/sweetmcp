use kalosm::vision::*;

#[tokio::main]
async fn main() {
    // Initialize OCR model
    let mut model = Ocr::builder().build().await.unwrap();
    
    // Load an image
    let image = image::open("examples/ocr.png").unwrap();
    
    // Perform OCR
    let text = model
        .recognize_text(OcrInferenceSettings::new(image))
        .unwrap();
    
    // Print the result
    println!("{}", text);
}

// Source: https://github.com/floneum/floneum/blob/main/interfaces/kalosm/examples/ocr.rs
