struct VertexOutput {
	@builtin(position) position: vec4<f32>,
	@location(0) uv: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> mvp: mat4x4<f32>;

@vertex
fn vs_main(
	@location(0) position: vec3<f32>,
	@location(1) uv: vec2<f32>,
) -> VertexOutput {
	var result: VertexOutput;
	result.position = mvp * vec4<f32>(position, 1.0);
	result.uv = uv;
	return result;
}

/*@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var tex_sampler: sampler;*/

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
	//return textureSample(texture, tex_sampler, uv);
	return vec4(1.0, 1.0, 0.0, 1.0);
}
