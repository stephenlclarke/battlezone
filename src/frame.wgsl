struct Vertex {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}
@vertex fn vs(@builtin(vertex_index) index: u32) -> Vertex {
    let uv = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    var out: Vertex;
    out.position = vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, 0.0, 1.0);
    out.uv = uv;
    return out;
}
@group(0) @binding(0) var raster: texture_2d<f32>;
@group(0) @binding(1) var raster_sampler: sampler;
@fragment fn fs(vertex: Vertex) -> @location(0) vec4<f32> {
    return textureSample(raster, raster_sampler, vertex.uv);
}
