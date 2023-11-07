struct VertexOutput {
	@builtin(position) position: vec4<f32>,
	@location(0) uv: vec2<f32>,
}

struct VertexUniforms {
	mvp: mat4x4<f32>,
	y_offset: f32,
}

@group(1) @binding(0)
var<uniform> uniforms: VertexUniforms;

@vertex
fn vs_main(
	@location(0) position: vec2<f32>,
	@location(1) uv: vec2<f32>,
) -> VertexOutput {
	var result: VertexOutput;
	result.uv = vec2(uv.x, uv.y + uniforms.y_offset);
	result.position = uniforms.mvp * vec4<f32>(position, 0.0, 1.0);
	return result;
}

@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var tex_sampler: sampler;

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
	return textureSample(texture, tex_sampler, uv);
}
