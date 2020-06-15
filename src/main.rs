#![allow(non_snake_case)]

use std::sync::Arc;
use vulkano::instance::{Instance, PhysicalDevice, InstanceExtensions};
use vulkano::device::{Device, Features, DeviceExtensions};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBuffer};
use vulkano::sync::GpuFuture;
use vulkano::pipeline::ComputePipeline;
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::image::{StorageImage, Dimensions};
use vulkano::format::Format;
use image::{Rgba, ImageBuffer};

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

	//Should draw the Mandelbrot set to an image
	mod ComputeShader {
		vulkano_shaders::shader!{
	        ty: "compute",
	        path: "src/MandelbrotComputeShader.glsl"
	    }
	}
	let shader = ComputeShader::Shader::load(device.clone()).expect("Failed to create shader module.");
	let computePipeline = Arc::new(ComputePipeline::new(device.clone(), &shader.main_entry_point(), &()).expect("Failed to create compute pipeline."));
	let imageSize = 1024;
	let image = StorageImage::new(device.clone(), Dimensions::Dim2d { width: imageSize, height: imageSize }, Format::R8G8B8A8Unorm, Some(queue.family())).unwrap();
	let layout = computePipeline.layout().descriptor_set_layout(0).unwrap();
	let descriptorSet = Arc::new(PersistentDescriptorSet::start(layout.clone()).add_image(image.clone()).unwrap().build().unwrap());
	let buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, (0 .. imageSize * imageSize * 4).map(|_| 0u8)).expect("Failed to create Buffer.");
	let mut builder = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap();
	builder.dispatch([imageSize / 8, imageSize / 8, 1], computePipeline.clone(), descriptorSet.clone(), ()).unwrap().copy_image_to_buffer(image.clone(), buffer.clone()).unwrap();
	let commandBuffer = builder.build().unwrap();
	commandBuffer.execute(queue.clone()).unwrap().then_signal_fence_and_flush().unwrap().wait(None).unwrap();
	let bufferContent = buffer.read().unwrap();
	let image = ImageBuffer::<Rgba<u8>, _>::from_raw(imageSize, imageSize, &bufferContent[..]).unwrap();
	image.save("output/Mandelbrot.png").unwrap();
}
