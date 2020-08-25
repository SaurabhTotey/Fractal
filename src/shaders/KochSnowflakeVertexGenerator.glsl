#version 450

layout(triangles) in;
layout(triangle_strip, max_vertices = 12) out;

float ROTATION = -3.14159265358 / 3.0;

void generateNewTriangleForSide(vec2 vertex1, vec2 vertex2) {
	vec2 base1 = vec2(2.0 / 3.0 * vertex1.x + 1.0 / 3.0 * vertex2.x, 2.0 / 3.0 * vertex1.y + 1.0 / 3.0 * vertex2.y);
	vec2 base2 = vec2(1.0 / 3.0 * vertex1.x + 2.0 / 3.0 * vertex2.x, 1.0 / 3.0 * vertex1.y + 2.0 / 3.0 * vertex2.y);
	vec2 baseVector = base2 - base1;

	mat3 baseVectorTransformation = mat3(
		cos(ROTATION), sin(ROTATION), 0.0,
		-sin(ROTATION), cos(ROTATION), 0.0,
		base1, 1.0
	);

	vec2 triangleTip = (baseVectorTransformation * vec3(baseVector, 1.0)).xy;

	gl_Position = vec4(base1, 0.0, 1.0);
	EmitVertex();
	gl_Position = vec4(base2, 0.0, 1.0);
	EmitVertex();
	gl_Position = vec4(triangleTip, 0.0, 1.0);
	EmitVertex();
	EndPrimitive();
}

void main() {
	// We want to keep our extant triangle
	for (int i = 0; i < 3; i++) {
		gl_Position = gl_in[i].gl_Position;
		EmitVertex();
	}
	EndPrimitive();

	// For each side, we generate a new triangle
	generateNewTriangleForSide(gl_in[0].gl_Position.xy, gl_in[1].gl_Position.xy);
	generateNewTriangleForSide(gl_in[1].gl_Position.xy, gl_in[2].gl_Position.xy);
	generateNewTriangleForSide(gl_in[2].gl_Position.xy, gl_in[0].gl_Position.xy);
}
