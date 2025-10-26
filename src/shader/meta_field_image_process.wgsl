struct FrameMetadata {
    resolution: vec2<u32>,
    top_left: vec2<i32>,
}
@group(0) @binding(0)
var<uniform> frame_metadata: FrameMetadata;

struct MetaFieldMetadata {
    offset: vec2<i32>,
    cell_size: u32,
}
@group(0) @binding(1)
var<uniform> metadata: MetaFieldMetadata;

struct MetaImageMetadata {
    min: vec2<f32>,
    max: vec2<f32>,
    multiplier: f32,
    padding: u32,
}
@group(0) @binding(2)
var<storage> meta_image: MetaImageMetadata;

@group(0) @binding(3)
var image: texture_2d<f32>;

@group(0) @binding(4)
var meta_field: texture_storage_2d<r32float, read_write>;

override workgroup_size_x: u32;
override workgroup_size_y: u32;

@compute @workgroup_size(workgroup_size_x, workgroup_size_y)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if any(id.xy >= frame_metadata.resolution) {
        return;
    }

    let cell_pos = vec2<i32>(id.xy) * i32(metadata.cell_size) + metadata.offset + frame_metadata.top_left;
    let image_dim = vec2<f32>(textureDimensions(image));

    var magn_sum = 0.0;
    var count = 0u;
    for (var y = 0u; y < metadata.cell_size; y += 1u) {
        for (var x = 0u; x < metadata.cell_size; x += 1u) {
            let image_uv =
                (vec2<f32>(cell_pos) - meta_image.min) / (meta_image.max - meta_image.min)
                + vec2<f32>(f32(x), f32(y)) / image_dim;

            if any(image_uv < vec2<f32>(0.0)) || any(image_uv > vec2<f32>(1.0)) {
                return;
            }

            let image_coord = vec2<u32>(image_uv * image_dim);
            let magn = textureLoad(image, image_coord, 0).r;

            magn_sum += magn;
            count += 1u;
        }
    }

    let magn = magn_sum / f32(count);

    let value = textureLoad(meta_field, id.xy);

    textureStore(
        meta_field,
        id.xy,
        value + vec4<f32>(magn * meta_image.multiplier, 0.0, 0.0, 0.0),
    );
}