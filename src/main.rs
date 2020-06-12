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

	//Should multiply all the data by 12
	mod ComputeShader {
		vulkano_shaders::shader!{
	        ty: "compute",
	        src: "
#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer Data {
    uint data[];
} buf;

void main() {
    uint idx = gl_GlobalInvocationID.x;
    buf.data[idx] *= 12;
}"
	    }
	}
	let dataToMultiply = 0 .. 65536;
	let dataBuffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, dataToMultiply).expect("Failed to create buffer.");
	let shader = ComputeShader::Shader::load(device.clone()).expect("Failed to create shader module.");
	let computePipeline = Arc::new(ComputePipeline::new(device.clone(), &shader.main_entry_point(), &()).expect("Failed to create compute pipeline."));
	let layout = computePipeline.layout().descriptor_set_layout(0).unwrap();
	let descriptorSet = Arc::new(PersistentDescriptorSet::start(layout.clone()).add_buffer(dataBuffer.clone()).unwrap().build().unwrap());
	let mut builder = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap();
	builder.dispatch([1024, 1, 1], computePipeline.clone(), descriptorSet.clone(), ()).unwrap();
	let commandBuffer = builder.build().unwrap();
	commandBuffer.execute(queue.clone()).unwrap().then_signal_fence_and_flush().unwrap().wait(None).unwrap();
	let content = dataBuffer.read().unwrap();
	for (n, val) in content.iter().enumerate() {
		assert_eq!(*val, n as u32 * 12);
	}
	print!("yay");
}
