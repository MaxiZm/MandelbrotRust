@group(0) @binding(0)
var<storage, read_write> field: array<f32, 1000000>;

struct Parameters {
    left: f32,
    bottom: f32,
    size: f32,
}

struct Complex {
    real: f32,
    imag: f32,

}

fn add_complex(a: Complex, b: Complex) -> Complex {
    return Complex(a.real + b.real, a.imag + b.imag);
}

fn multiple_complex(a: Complex, b: Complex) -> Complex {
    return Complex(a.real * b.real - a.imag * b.imag, a.real * b.imag + a.imag * b.real);
}

fn multiple_real(a: Complex, b: f32) -> Complex {
    return Complex(a.real * b, a.imag * b);
}

fn pow_complex(a: Complex, n: i32) -> Complex {
    var result: Complex = Complex(1.0, 0.0);
    for (var i = 0; i < n; i = i + 1) {
        result = multiple_complex(result, a);
    }
    return result;
}

fn sub_complex(a: Complex, b: Complex) -> Complex {
    return Complex(a.real - b.real, a.imag - b.imag);
}

fn magnitude(z: Complex) -> f32 {
    return sqrt(z.real * z.real + z.imag * z.imag);
}


fn fn_z(z_: Complex, c: Complex) -> Complex {
    return add_complex(pow_complex(z_, 4), c);
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

        var c: Complex = Complex(a0, b0);
        var z: Complex = Complex(0.0, 0.0);

        // Set a fixed iteration count, e.g., 100. This ensures rendering time is approximately constant
        let max_iterations: i32 = i32(40 * (8.0 / sqrt(params.size)));

        for (var i = 0; i < max_iterations; i = i + 1) {
            z = fn_z(z, c);
            let dist = magnitude(sub_complex(z, c));

            if (dist > 2.0) {
                // Apply smooth coloring based on escape time
                field[index] = f32(i) / f32(40);
                return;
            }
        }

        // If the point doesn't escape, mark it as part of the Mandelbrot set
        field[index] = 0.0;
    }
}
