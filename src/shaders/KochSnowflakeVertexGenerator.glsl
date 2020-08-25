#version 450

//layout(lines) in;
//layout(lines, max_vertices = 5) out;
//layout(invocations = 1) in;

layout(triangles) in;
layout(triangle_strip, max_vertices = 3) out;

void main() {
	for (int i = 0; i < 3; i++) {
		gl_Position = gl_in[i].gl_Position;
		EmitVertex();
	}
	EndPrimitive();
}
