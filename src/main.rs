#![allow(non_snake_case)]

use std::sync::Arc;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::device::{Device, Features, DeviceExtensions};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::sync::{GpuFuture, now};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::framebuffer::{Framebuffer, Subpass, RenderPassAbstract, FramebufferAbstract};
use vulkano::pipeline::viewport::Viewport;
use vulkano_win::VkSurfaceBuild;
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::{WindowBuilder, Window};
use winit::event::{Event, WindowEvent};
use vulkano::swapchain::{Swapchain, SurfaceTransform, PresentMode, FullscreenExclusive, ColorSpace, acquire_next_image};
use vulkano::image::{ImageUsage, SwapchainImage};

fn main() {
	let instance = Instance::new(None, &vulkano_win::required_extensions(), None)
		.expect("Failed to create instance.");
	let physicalDevice = PhysicalDevice::enumerate(&instance).next()
		.expect("No physical device available.");

	let eventsLoop = EventLoop::new();
	let surface = WindowBuilder::new().build_vk_surface(&eventsLoop, instance.clone()).unwrap();
	let surfaceCapabilities = surface.capabilities(physicalDevice)
		.expect("Cannot get window surface capabilities.");

	let queueFamily = physicalDevice.queue_families()
		.find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
		.expect("Couldn't find a suitable graphical queue family.");
	let (device, mut queues) = { Device::new(
		physicalDevice,
		&Features { geometry_shader: true, ..Features::none() },
		&DeviceExtensions{ khr_swapchain: true, ..DeviceExtensions::none() },
		[(queueFamily, 0.5)].iter().cloned()
	).expect("Failed to create device.") };
	let queue = queues.next().unwrap();

	let (mut swapchain, images) = {
		let supportedAlpha = surfaceCapabilities.supported_composite_alpha.iter().next().unwrap();
		let imageFormat = surfaceCapabilities.supported_formats[0].0;
		let dimensions = Into::<[u32; 2]>::into(surface.window().inner_size());
		Swapchain::new(
			device.clone(),
			surface.clone(),
			surfaceCapabilities.min_image_count,
			imageFormat,
			dimensions,
			1,
			ImageUsage::color_attachment(),
			&queue,
			SurfaceTransform::Identity,
			supportedAlpha,
			PresentMode::Fifo,
			FullscreenExclusive::Default,
			true,
			ColorSpace::SrgbNonLinear,
		).unwrap()
	};

	#[derive(Default, Copy, Clone)]
	struct Vertex {
		position: [f32; 2],
	}
	vulkano::impl_vertex!(Vertex, position);

	// Vertices define a centered equilateral triangle
	let vertices: Vec<Vertex> = [
		Vertex { position: [ 0.5,  1f32 / 3f32.sqrt() / 2f32] },
		Vertex { position: [-0.5,  1f32 / 3f32.sqrt() / 2f32] },
		Vertex { position: [ 0.0, -1f32 / 3f32.sqrt()       ] }
	].to_vec();
	let vertexBuffer = CpuAccessibleBuffer::from_iter(
		device.clone(),
		BufferUsage::all(),
		false,
		vertices.iter().cloned()
	).unwrap();

	mod VertexShader { vulkano_shaders::shader!{ ty: "vertex", path: "src/shaders/SetPositionVertexShader.glsl" } }
	mod GeometryShader { vulkano_shaders::shader!{ ty: "geometry", path: "src/shaders/PseudoKochSnowflakeVertexGenerator.glsl" } }
	mod FragmentShader { vulkano_shaders::shader!{ ty: "fragment", path: "src/shaders/FillWhiteFragmentShader.glsl" } }

	let vertexShader = VertexShader::Shader::load(device.clone()).expect("Failed to load vertex shader.");
	let geometryShader = GeometryShader::Shader::load(device.clone()).expect("Failed to load geometry shader.");
	let fragmentShader = FragmentShader::Shader::load(device.clone()).expect("Failed to load fragment shader.");

	let renderPass = Arc::new(vulkano::single_pass_renderpass!(device.clone(),
	    attachments: {
	        color: {
	            load: Clear,
	            store: Store,
	            format: swapchain.format(),
	            samples: 1,
	        }
	    },
	    pass: {
	        color: [color],
	        depth_stencil: {}
	    }
	).unwrap());

	let pipeline = Arc::new(GraphicsPipeline::start()
		.vertex_input_single_buffer::<Vertex>()
		.vertex_shader(vertexShader.main_entry_point(), ())
		.triangle_list()
		.viewports_dynamic_scissors_irrelevant(1)
		.geometry_shader(geometryShader.main_entry_point(), ())
		.fragment_shader(fragmentShader.main_entry_point(), ())
		.render_pass(Subpass::from(renderPass.clone(), 0).unwrap())
		.build(device.clone())
		.unwrap());

	let mut dynamicState = DynamicState {
		line_width: None,
		viewports: None,
		scissors: None,
		compare_mask: None,
		write_mask: None,
		reference: None
	};
	let mut frameBuffers = frameBuffersForWindowSize(&images, renderPass.clone(), &mut dynamicState);

	let mut shouldRecreateSwapchain = false;
	let mut previousFrameEnding = Some(now(device.clone()).boxed());

	eventsLoop.run(move |event, _, control_flow| {
		match event {
			Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
				*control_flow = ControlFlow::Exit;
			},
			Event::WindowEvent { event: WindowEvent::Resized(_), .. } => {
				shouldRecreateSwapchain = true;
			},
			Event::RedrawEventsCleared => {
				previousFrameEnding.as_mut().unwrap().cleanup_finished();
				if shouldRecreateSwapchain {
					shouldRecreateSwapchain = false;
					let newDimensions = Into::<[u32; 2]>::into(surface.window().inner_size());
					let (newSwapchain, newImages) = match swapchain.recreate_with_dimensions(newDimensions) {
						Ok(r) => r,
						Err(_) => {
							return;
						}
					};
					swapchain = newSwapchain;
					frameBuffers = frameBuffersForWindowSize(&newImages, renderPass.clone(), &mut dynamicState);
				}
				let (numberOfImages, isSuboptimal, acquireFuture) = match acquire_next_image(swapchain.clone(), None) {
					Ok(r) => r,
					Err(_) => {
						shouldRecreateSwapchain = true;
						return;
					}
				};
				if isSuboptimal {
					shouldRecreateSwapchain = true;
					return;
				}

				let mut builder = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap();
				builder
					.begin_render_pass(frameBuffers[numberOfImages].clone(), false, vec![[0.0, 0.0, 0.0, 1.0].into()]).unwrap()
					.draw(pipeline.clone(), &dynamicState, vertexBuffer.clone(), (), ()).unwrap()
					.end_render_pass().unwrap();
				let commandBuffer = builder.build().unwrap();

				let future = previousFrameEnding
					.take().unwrap()
					.join(acquireFuture)
					.then_execute(queue.clone(), commandBuffer).unwrap()
					.then_swapchain_present(queue.clone(), swapchain.clone(), numberOfImages)
					.then_signal_fence_and_flush();
				previousFrameEnding = match future {
					Ok(f) => Some(f.boxed()),
					Err(_) => {
						shouldRecreateSwapchain = true;
						Some(now(device.clone()).boxed())
					}
				}
			},
			_ => ()
		}
	});
}

fn frameBuffersForWindowSize(images: &[Arc<SwapchainImage<Window>>], renderPass: Arc<dyn RenderPassAbstract + Send + Sync>, dynamicState: &mut DynamicState) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
	let dimensions = images[0].dimensions();
	let viewport = Viewport {
		origin: [0.0, 0.0],
		dimensions: [dimensions[0] as f32, dimensions[1] as f32],
		depth_range: 0.0..1.0,
	};
	dynamicState.viewports = Some(vec![viewport]);
	return images.iter()
		.map(|image| Arc::new(
			Framebuffer::start(renderPass.clone()).add(image.clone()).unwrap().build().unwrap()) as Arc<dyn FramebufferAbstract + Send + Sync>
		).collect::<Vec<_>>()
}
