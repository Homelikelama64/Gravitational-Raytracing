use wgpu::{
    include_wgsl, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    ComputePipelineDescriptor, DeviceDescriptor, Instance, InstanceDescriptor,
    PipelineLayoutDescriptor, RequestAdapterOptions, StorageTextureAccess, TextureFormat,
    TextureViewDimension,
};

fn main() {
    let instance = Instance::new(InstanceDescriptor::default());

    let adapter =
        pollster::block_on(instance.request_adapter(&RequestAdapterOptions::default())).unwrap();

    let (device, queue) =
        pollster::block_on(adapter.request_device(&DeviceDescriptor::default(), None)).unwrap();

    let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

    let output_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("output_bind_group_layout"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: BindingType::StorageTexture {
                access: StorageTextureAccess::WriteOnly,
                format: TextureFormat::Rgba8Unorm,
                view_dimension: TextureViewDimension::D2,
            },
            count: None,
        }],
    });

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("pipeline_layout"),
        bind_group_layouts: &[&output_bind_group_layout],
        push_constant_ranges: &[],
    });

    let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("compute_pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: "trace_ray",
        compilation_options: Default::default(),
        cache: Default::default(),
    });
}
