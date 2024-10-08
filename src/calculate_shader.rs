use pollster::block_on;
use wgpu::util::DeviceExt;
use std::error::Error;
use std::fs;
use bytemuck::Pod;
use bytemuck::Zeroable;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Parameters {
    pub(crate) left: f32,
    pub(crate) bottom: f32,
    pub(crate) size: f32
}


pub async fn run_shader(matrix: &mut [f32], params: Parameters) -> Result<(), Box<dyn Error>> {
    let t1 = std::time::Instant::now();

    // Load shader code
    let shader_code = fs::read_to_string("shader.wgsl")?;

    // Create instance
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
        flags: wgpu::InstanceFlags::empty(),
        gles_minor_version: wgpu::Gles3MinorVersion::Version0,
    });

    // Request an adapter and device
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        force_fallback_adapter: false,
        compatible_surface: None,
    }).await.unwrap();

    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None).await.unwrap();

    // Create matrix buffer for GPU processing
    let matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Matrix Buffer"),
        contents: bytemuck::cast_slice(matrix),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    });

    // Output buffer for reading results back
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output Buffer"),
        size: (matrix.len() * std::mem::size_of::<f32>()) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    // Parameters
    let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Parameters Buffer"),
        contents: bytemuck::cast_slice(&[params]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_SRC,
    });

    // Load and compile the shader
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(shader_code.into()),
    });

    // Create compute pipeline
    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute Pipeline"),
        layout: None,
        module: &shader,
        entry_point: "main",
        cache: None,
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    });

    // Bind group setup
    let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bind Group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: matrix_buffer.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
            binding: 1,
            resource: params_buffer.as_entire_binding(),
        }],
    });

    let workgroups_x: u32 = (1000 + 3) / 4; // Adding 3 to handle any remainder when dividing by 4
    let workgroups_y: u32 = (1000 + 3) / 4;

    // Command encoder and dispatch
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Command Encoder") });
    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute Pass"),
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(&compute_pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
    }

    // Copy results to output buffer
    encoder.copy_buffer_to_buffer(&matrix_buffer, 0, &output_buffer, 0, (matrix.len() * std::mem::size_of::<f32>()) as u64);
    queue.submit(Some(encoder.finish()));

    // Read back the results
    let buffer_slice = output_buffer.slice(..);
    buffer_slice.map_async(wgpu::MapMode::Read, |result| {
        if let Err(e) = result {
            eprintln!("Failed to map buffer: {}", e);
        }
    });
    device.poll(wgpu::Maintain::Wait);

    let data = buffer_slice.get_mapped_range();
    matrix.copy_from_slice(bytemuck::cast_slice(&data));
    drop(data);
    output_buffer.unmap();

    let t2 = std::time::Instant::now();
    println!("Shader took: {:?}", t2 - t1);

    Ok(())
}