use pollster::block_on;
use crate::calculate_shader::{run_shader, Parameters};
use minifb::{Key, Window, WindowOptions, MouseButton, MouseMode};

mod calculate_shader;

fn main() {
    // Initialize parameters
    let mut params = Parameters {
        left: -2.0,
        bottom: -1.5,
        size: 3.0,
    };

    // Create window
    let mut window = match Window::new("Shader Visualization", 1000, 1000, WindowOptions::default()) {
        Ok(win) => win,
        Err(e) => {
            eprintln!("Unable to create window: {}", e);
            return;
        }
    };

    // Prepare the screen buffer for visualization
    let mut screen_matrix: [u32; 1_000_000] = [0; 1_000_000];
    let mut matrix = vec![1.0f32; 1_000_000];

    // Run the shader and update the screen_matrix
    let mut needs_update = true;
    let mut mouse_down = false;
    let mut x0 : u32 = 0;
    let mut y0 : u32 = 0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if needs_update {
            // Run the shader
            if let Err(e) = block_on(run_shader(&mut matrix, params)) {
                eprintln!("Error running shader: {}", e);
                return;
            }

            // Update the screen buffer with smooth coloring
            for i in 0..1_000_000 {
                let intensity = (matrix[i] * 255.0) as u32; // Map escape time to [0, 255]

                // Create a grayscale color (or adjust to a color gradient if preferred)
                let color = if intensity > 0 {
                    (intensity << 16) | (intensity << 8) | intensity // Grayscale color
                } else {
                    0x000000 // Black for points within the Mandelbrot set
                };

                screen_matrix[i] = color;
            }


            needs_update = false;
        }

        // Handle mouse input
        if let Some((mouse_x, mouse_y)) = window.get_mouse_pos(MouseMode::Discard) {
            if window.get_mouse_down(MouseButton::Left) && !mouse_down {
                mouse_down = true;
                x0 = mouse_x as u32;
                y0 = mouse_y as u32;
            } else if !window.get_mouse_down(MouseButton::Left) && mouse_down {
                mouse_down = false;
                let x1 = mouse_x as u32;
                let y1 = mouse_y as u32;

                // Calculate the new parameters
                let (dx, dy) = (x1 as f32 - x0 as f32, y1 as f32 - y0 as f32);
                let (x0, y0) = (x1, y1);

                params.left -= dx * params.size / 1000.0;
                params.bottom -= dy * params.size / 1000.0;
                needs_update = true;
            }
        }

        // Handle zooming with keys
        if window.is_key_down(Key::E) {
            // Zoom in
            params.size *= 0.9;
            needs_update = true;
        }

        if window.is_key_down(Key::Q) {
            // Zoom out
            params.size /= 0.9;
            needs_update = true;
        }

        if window.is_key_down(Key::W) {
            // Move up
            params.bottom -= params.size * 0.1;
            needs_update = true;
        }

        if window.is_key_down(Key::S) {
            // Move down
            params.bottom += params.size * 0.1;
            needs_update = true;
        }

        if window.is_key_down(Key::A) {
            // Move left
            params.left -= params.size * 0.1;
            needs_update = true;
        }

        if window.is_key_down(Key::D) {
            // Move right
            params.left += params.size * 0.1;
            needs_update = true;
        }

        // Update the window
        if window
            .update_with_buffer(&screen_matrix, 1000, 1000)
            .is_err()
        {
            eprintln!("Failed to update window buffer.");
            break;
        }

        // Add a short sleep to limit CPU usage
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
