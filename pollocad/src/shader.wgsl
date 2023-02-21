struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

struct Camera {
    view: mat4x4<f32>,
};

struct Uniforms {
    @size(16) angle: f32,
};

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(model: VertexIn) -> VertexOut {
    var out: VertexOut;

    let light_dir: vec3<f32> = normalize(vec3<f32>(1.0, 1.0, 1.0));

    let c_1: vec4<f32> = vec4<f32>(cos(uniforms.angle), 0.0, sin(uniforms.angle), 0.0);
    let c_2: vec4<f32> = vec4<f32>(0.0, 1.0, 0.0, 0.0);
    let c_3: vec4<f32> = vec4<f32>(-sin(uniforms.angle), 0.0, cos(uniforms.angle), 0.0);
    let c_4: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let rotation_matrix: mat4x4<f32> = mat4x4<f32>(c_1, c_2, c_3, c_4);

    out.clip_position = camera.view * rotation_matrix * vec4<f32>(model.position, 1.0);
    out.color = vec3<f32>(1.0, 1.0, 0.0) * max(dot((rotation_matrix * vec4<f32>(normalize(-model.normal), 1.0)).xyz, light_dir), 0.3);

    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
