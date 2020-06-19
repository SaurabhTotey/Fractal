#version 450

layout(location = 0) out vec4 color;

const int MAX_ITERATIONS = 256;

vec2 f(vec2 z, vec2 c) {
	return vec2(z.x * z.x - z.y * z.y + c.x, 2 * z.y * z.x + c.y);
}

void main() {
	vec2 c = (((gl_FragCoord.xy + vec2(0.5)) / vec2(1024.0)) - vec2(0.5)) * 2.0 - vec2(1.0, 0.0);
	vec2 z = vec2(0.0);
	float i;
	for (i = 0; i < 1.0; i += 1.0 / MAX_ITERATIONS) {
		z = f(z, c);
		if (length(z) > 4.0) {
			break;
		}
	}
	color = vec4(vec3(i), 1.0);
}
