use kalosm::vision::*;

fn main() {
    // Initialize segmentation model
    let model = SegmentAnything::builder().build().unwrap();
    
    // Load an image
    let image = image::open("examples/landscape.jpg").unwrap();
    
    // Set points to segment around
    let x = image.width() / 2;
    let y = image.height() / 4;
    
    // Perform segmentation
    let images = model
        .segment_from_points(SegmentAnythingInferenceSettings::new(image).add_goal_point(x, y))
        .unwrap();

    // Save result
    images.save("out.png").unwrap();
}

// Source: https://github.com/floneum/floneum/blob/main/interfaces/kalosm/examples/segment-image.rs
