@group(0) @binding(0)
var<storage, read_write> field: array<f32, 1000000>;

struct Parameters {
    left: f32,
    bottom: f32,
    size: f32,
}

@group(0) @binding(1)
var<uniform> params: Parameters;

@compute @workgroup_size(4, 4, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;

    if (x < 1000u && y < 1000u) {
        let index = y * 1000 + x;

        // Use params.size to zoom, but fix the complexity to maintain constant rendering time
        let scale_factor = params.size / 1000.0;

        let a0: f32 = f32(x) * scale_factor + params.left;
        let b0: f32 = f32(y) * scale_factor + params.bottom;

        var a: f32 = 0.0;
        var b: f32 = 0.0;

        // Set a fixed iteration count, e.g., 100. This ensures rendering time is approximately constant
        let max_iterations: i32 = i32(40 * (8.0 / sqrt(params.size)));

        for (var i = 0; i < max_iterations; i = i + 1) {
            let new_a = a * a - b * b + a0;
            b = 2.0 * a * b + b0;
            a = new_a;

            if (a * a + b * b > 4.0) {
                // Apply smooth coloring based on escape time
                field[index] = f32(i) / f32(40);
                return;
            }
        }

        // If the point doesn't escape, mark it as part of the Mandelbrot set
        field[index] = 0.0;
    }
}
