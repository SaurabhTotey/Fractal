#version 450

layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;

const float radius = 256.0;

void main() {
//  Commented out code section makes the circle fade away from the center
//	float normalizedDistanceFromCenter = distance(gl_GlobalInvocationID.xy, vec2(512, 512)) / radius;
//	vec4 color = vec4(vec3(normalizedDistanceFromCenter), 1.0);
	bool isWithinRadius = distance(gl_GlobalInvocationID.xy, vec2(512, 512)) <= radius;
	vec4 color = vec4(vec3(isWithinRadius? 1.0 : 0.0), 1.0);
	imageStore(img, ivec2(gl_GlobalInvocationID.xy), color);
}
