#![allow(non_snake_case)]

use std::sync::Arc;
use vulkano::instance::{Instance, PhysicalDevice, InstanceExtensions};
use vulkano::device::{Device, Features, DeviceExtensions};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBuffer, DynamicState};
use vulkano::sync::GpuFuture;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::image::{StorageImage, Dimensions};
use vulkano::format::Format;
use vulkano::framebuffer::{Framebuffer, Subpass};
use image::{Rgba, ImageBuffer};
use vulkano::pipeline::viewport::Viewport;

fn main() {
	let instance = Instance::new(None, &InstanceExtensions::none(), None).expect("Failed to create instance.");
	let physicalDevice = PhysicalDevice::enumerate(&instance).next().expect("No physical device available.");
	let queueFamily = physicalDevice.queue_families().find(|&q| q.supports_graphics()).expect("Couldn't find a graphical queue family.");
	let (device, mut queues) = { Device::new(
		physicalDevice,
		&Features::none(),
		&DeviceExtensions{khr_storage_buffer_storage_class:true, ..DeviceExtensions::none()},
		[(queueFamily, 0.5)].iter().cloned()
	).expect("Failed to create device.") };
	let queue = queues.next().unwrap();

	#[derive(Default, Copy, Clone)]
	struct Vertex {
		position: [f32; 2],
	}
	vulkano::impl_vertex!(Vertex, position);

	// Vertices correspond to two triangles (one for each half of the screen); both triangles cover entire screen
	let vertices: Vec<Vertex> = [
		Vertex { position: [0.5, 0.5] },
		Vertex { position: [-0.5, 0.5] },
		Vertex { position: [0.0, 0.5 - 3f32.sqrt() / 2f32] }
	].to_vec();

	let imageSize = 1024;
	let image = StorageImage::new(device.clone(), Dimensions::Dim2d { width: imageSize, height: imageSize }, Format::R8G8B8A8Unorm, Some(queue.family())).unwrap();
	let buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, (0 .. imageSize * imageSize * 4).map(|_| 0u8)).expect("Failed to create Buffer.");

	mod VertexShader { vulkano_shaders::shader!{ ty: "vertex", path: "src/shaders/SetPositionVertexShader.glsl" } }
	mod FragmentShader { vulkano_shaders::shader!{ ty: "fragment", path: "src/shaders/FillWhiteFragmentShader.glsl" } }
	let vertexShader = VertexShader::Shader::load(device.clone()).expect("Failed to load vertex shader.");
	let fragmentShader = FragmentShader::Shader::load(device.clone()).expect("Failed to load fragment shader.");
	let vertexBuffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, vertices.into_iter()).unwrap();
	let renderPass = Arc::new(vulkano::single_pass_renderpass!(device.clone(),
	    attachments: {
	        color: {
	            load: Clear,
	            store: Store,
	            format: Format::R8G8B8A8Unorm,
	            samples: 1,
	        }
	    },
	    pass: {
	        color: [color],
	        depth_stencil: {}
	    }
	).unwrap());
	let framebuffer = Arc::new(Framebuffer::start(renderPass.clone()).add(image.clone()).unwrap().build().unwrap());

	let dynamicState = DynamicState {
		viewports: Some(vec![Viewport {
			origin: [0.0, 0.0],
			dimensions: [imageSize as f32, imageSize as f32],
			depth_range: 0.0 .. 1.0,
		}]),
		.. DynamicState::none()
	};

	let pipeline = Arc::new(GraphicsPipeline::start()
		.vertex_input_single_buffer::<Vertex>()
		.vertex_shader(vertexShader.main_entry_point(), ())
		.viewports_dynamic_scissors_irrelevant(1)
		.fragment_shader(fragmentShader.main_entry_point(), ())
		.render_pass(Subpass::from(renderPass.clone(), 0).unwrap())
		.build(device.clone())
		.unwrap());

	let mut builder = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap();
	builder
		.begin_render_pass(framebuffer.clone(), false, vec![[0.0, 0.0, 0.0, 1.0].into()]).unwrap()
		.draw(pipeline.clone(), &dynamicState, vertexBuffer.clone(), (), ()).unwrap()
		.end_render_pass().unwrap()
		.copy_image_to_buffer(image.clone(), buffer.clone()).unwrap();
	builder.build().unwrap().execute(queue.clone()).unwrap().then_signal_fence_and_flush().unwrap().wait(None).unwrap();

	let bufferContent = buffer.read().unwrap();
	let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &bufferContent[..]).unwrap();
	image.save("output/GraphicsKoch.png").unwrap();
}
