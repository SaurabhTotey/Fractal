#![allow(non_snake_case)]

use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;
use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;

fn main() {
	let instance = Instance::new(None, &InstanceExtensions::none(), None).expect("Failed to create instance.");
	let physicalDevice = PhysicalDevice::enumerate(&instance).next().expect("No physical device available.");
	let queueFamily = physicalDevice.queue_families().find(|&q| q.supports_graphics()).expect("Couldn't find a graphical queue family.");
	let (device, mut queues) = { Device::new(
		physicalDevice,
		&Features::none(),
		&DeviceExtensions::none(),
		[(queueFamily, 0.5)].iter().cloned()
	).expect("Failed to create device.") };
	let queue = queues.next().unwrap();
}
