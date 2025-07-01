/// Implements Sixel rendering for images
/// Convert an RGB image to a Sixel string for terminal display
pub fn encode_sixel(img: &image::RgbImage) -> String {
        // Start with DCS sequence + sixel + raster attributes with image dimensions
        let mut result = String::from("\x1BPq");

        // Add raster attributes (crucial for Rio compatibility)
        result.push_str(&format!("\"{};{}", img.width(), img.height()));

        // Define a basic 16-color palette (8-bit RGB values)
        // Using VT-340 compatible palette for better Rio compatibility
        result.push_str("#0;2;0;0;0"); // 0: Black
        result.push_str("#1;2;20;20;80"); // 1: Dark Blue
        result.push_str("#2;2;20;80;20"); // 2: Dark Green
        result.push_str("#3;2;20;80;80"); // 3: Dark Cyan
        result.push_str("#4;2;80;20;20"); // 4: Dark Red
        result.push_str("#5;2;80;20;80"); // 5: Dark Magenta
        result.push_str("#6;2;80;80;20"); // 6: Brown
        result.push_str("#7;2;80;80;80"); // 7: Light Gray
        result.push_str("#8;2;40;40;40"); // 8: Dark Gray
        result.push_str("#9;2;40;40;100"); // 9: Light Blue
        result.push_str("#10;2;40;100;40"); // 10: Light Green
        result.push_str("#11;2;40;100;100"); // 11: Light Cyan
        result.push_str("#12;2;100;40;40"); // 12: Light Red
        result.push_str("#13;2;100;40;100"); // 13: Light Magenta
        result.push_str("#14;2;100;100;40"); // 14: Yellow
        result.push_str("#15;2;100;100;100"); // 15: White

        // Function to find the closest color in our palette
        let find_closest_color = |r: u8, g: u8, b: u8| -> u16 {
            // Very simple distance calculation - in production you'd want a better algorithm
            let colors = [
                (0, 0, 0),       // Black
                (20, 20, 80),    // Dark Blue
                (20, 80, 20),    // Dark Green
                (20, 80, 80),    // Dark Cyan
                (80, 20, 20),    // Dark Red
                (80, 20, 80),    // Dark Magenta
                (80, 80, 20),    // Brown
                (80, 80, 80),    // Light Gray
                (40, 40, 40),    // Dark Gray
                (40, 40, 100),   // Light Blue
                (40, 100, 40),   // Light Green
                (40, 100, 100),  // Light Cyan
                (100, 40, 40),   // Light Red
                (100, 40, 100),  // Light Magenta
                (100, 100, 40),  // Yellow
                (100, 100, 100), // White
            ];

            let mut min_dist = u32::MAX;
            let mut closest = 0;

            for (i, &(cr, cg, cb)) in colors.iter().enumerate() {
                // Simple Euclidean distance
                let dist = ((r as i32 - cr).pow(2)
                    + (g as i32 - cg).pow(2)
                    + (b as i32 - cb).pow(2)) as u32;
                if dist < min_dist {
                    min_dist = dist;
                    closest = i;
                }
            }

            closest as u16
        };

        // Process the image in sixel format (6 vertical pixels at a time)
        for y in (0..img.height()).step_by(6) {
            // Initialize with color 0
            result.push_str("#0");

            let mut current_color = 0;
            let mut run_length = 0;
            let mut last_sixel_value = 0;

            for x in 0..img.width() {
                // Get the color for this column and select the dominant one
                let mut column_colors = [0u16; 6];

                for i in 0..6 {
                    if y + i < img.height() {
                        let pixel = img.get_pixel(x, y + i);
                        column_colors[i as usize] =
                            find_closest_color(pixel[0], pixel[1], pixel[2]);
                    }
                }

                // Use most common color in this column
                let dominant_color = *column_colors
                    .iter()
                    .max_by_key(|&&c| column_colors.iter().filter(|&&x| x == c).count())
                    .unwrap_or(&0);

                // Switch color if needed
                if dominant_color != current_color {
                    // Output any pending run-length
                    if run_length > 0 {
                        if run_length > 1 {
                            result.push_str(&format!("!{}", run_length));
                        }
                        result.push(char::from_u32(63 + last_sixel_value).unwrap());
                        run_length = 0;
                    }

                    result.push_str(&format!("#{}", dominant_color));
                    current_color = dominant_color;
                }

                // Calculate sixel value for this column
                let mut sixel_value = 0;
                for i in 0..6 {
                    if y + i < img.height() {
                        // Set bit i if pixel is closer to foreground than background
                        let pixel = img.get_pixel(x, y + i);
                        let color = find_closest_color(pixel[0], pixel[1], pixel[2]);

                        // If color matches dominant, set the bit
                        if color == current_color {
                            sixel_value |= 1 << i;
                        }
                    }
                }

                // Check if we can use run-length encoding
                if sixel_value == last_sixel_value && run_length > 0 {
                    run_length += 1;
                } else {
                    // Output any pending run-length
                    if run_length > 0 {
                        if run_length > 1 {
                            result.push_str(&format!("!{}", run_length));
                        }
                        result.push(char::from_u32(63 + last_sixel_value).unwrap());
                    }

                    last_sixel_value = sixel_value;
                    run_length = 1;
                }
            }

            // Output the last run
            if run_length > 0 {
                if run_length > 1 {
                    result.push_str(&format!("!{}", run_length));
                }
                result.push(char::from_u32(63 + last_sixel_value).unwrap());
            }

            // End of line - use "-" for Rio compatibility instead of "$\n"
            result.push('-');
        }

        // End sixel sequence
        result.push_str("\x1B\\");

        result
    }
