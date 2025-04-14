#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var<uniform> bits: u32;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let solved = bits & 2;
    if solved == 0 && length(mesh.uv - vec2(0.5)) < 0.4 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    let selected = bits & 1;
    if selected != 0 {
        return vec4<f32>(1.0, 0.5, 0.2, 1.0);
    }
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
