#version 450

layout(triangles) in;
layout(triangle_strip, max_vertices = 256) out;

const int ITERATIONS = 3;

const float ROTATION = -3.14159265358 / 3.0;
const int M = 3 * int(pow(2, ITERATIONS - 1)); // M is the max number of triangles possibly generated in a single iteration

/**
 * Given a sparsely populated array of triangles and the spacing of when real triangles should be found,
 * commits all real triangles to be drawn
 */
void commitTriangles(vec2[M][3] triangles, int spacingDenominator) {
	for (int i = 0; i < M; i += M / spacingDenominator) {
		for (int j = 0; j < 3; j++) {
			gl_Position = vec4(triangles[i][j], 0.0, 1.0);
			EmitVertex();
		}
		EndPrimitive();
	}
}

/**
 * Takes in the positions of two vertices of a triangle and returns the vertices for a new triangle
 * The last returned vertex is the vertex of the triangle that is not on the line formed by the given two vertices
 */
vec2[3] generateNewTriangleForSide(vec2 vertex1, vec2 vertex2) {
	// Get the positions for the 'base' of the new triangle
	// The base of the new triangle is the middle third of the line formed by the given vertices
	vec2 base1 = vec2(2.0 / 3.0 * vertex1.x + 1.0 / 3.0 * vertex2.x, 2.0 / 3.0 * vertex1.y + 1.0 / 3.0 * vertex2.y);
	vec2 base2 = vec2(1.0 / 3.0 * vertex1.x + 2.0 / 3.0 * vertex2.x, 1.0 / 3.0 * vertex1.y + 2.0 / 3.0 * vertex2.y);

	// Get the tip of the triangle off of the base
	// baseVector is the length of the base, which is the length of all sides because all generated triangles are equilateral
	// baseVector is then rotated by 60 degrees so that it can point to the tip relative to base1
	vec2 baseVector = base2 - base1;
	mat3 baseVectorTransformation = mat3(
		cos(ROTATION), sin(ROTATION), 0.0,
		-sin(ROTATION), cos(ROTATION), 0.0,
		base1, 1.0
	);
	vec2 triangleTip = (baseVectorTransformation * vec3(baseVector, 1.0)).xy;

	// Return our triangle
	return vec2[3](base1, base2, triangleTip);
}

void main() {
	// We want to keep our extant triangle
	for (int i = 0; i < 3; i++) {
		gl_Position = gl_in[i].gl_Position;
		EmitVertex();
	}
	EndPrimitive();

	if (ITERATIONS < 1) {
		return;
	}

	// triangles is an array of all triangles that is sparsely populated with actual triangles
	vec2[M][3] triangles;

	// How far apart in the array should actual triangles be
	// First spacing is 1/3 (every 1/3rd of the array should have triangles)
	// Gets doubled every iteration
	int spacingDenominator = 3;

	// First iteration needs to be hardcoded because is the only iteration that generates 3 triangles per triangle instead of 2
	triangles[0] = generateNewTriangleForSide(gl_in[0].gl_Position.xy, gl_in[1].gl_Position.xy);
	triangles[M / 3] = generateNewTriangleForSide(gl_in[1].gl_Position.xy, gl_in[2].gl_Position.xy);
	triangles[2 * M / 3] = generateNewTriangleForSide(gl_in[2].gl_Position.xy, gl_in[0].gl_Position.xy);
	commitTriangles(triangles, spacingDenominator);

	for (int iteration = 1; iteration < ITERATIONS; iteration++) {
		int newSpacingDenominator = spacingDenominator * 2;
		for (int i = 0; i < M; i += M / spacingDenominator) {
			vec2[3] triangle = triangles[i];
			triangles[i] = generateNewTriangleForSide(triangle[0], triangle[2]);
			triangles[i + M / newSpacingDenominator] = generateNewTriangleForSide(triangle[2], triangle[1]);
		}
		commitTriangles(triangles, newSpacingDenominator);
		spacingDenominator = newSpacingDenominator;
	}
}
