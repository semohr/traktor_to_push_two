@group(0)
@binding(0)
var<storage,read> knobs_state: Knobs;
struct Knobs {
    n_knobs: u32,
    values: array<f32,16>
};


struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) instance_index: u32,
};


@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32, @builtin(instance_index) in_instance_index: u32) -> VertexOutput {
    // Calculate the index of the knob based on vertex index
    var knob_val = knobs_state.values[in_instance_index];

    var width: f32 = knob_val / f32(knobs_state.n_knobs) * 2;
    var height: f32 = 0.1;

    // Define vertices for a rectangle
    var vertices: array<vec4<f32>, 6> = array<vec4<f32>, 6>(
        vec4<f32>(-1.0, 1.0-height, 0.0, 1.0),
        vec4<f32>(-1.0, 1.0, 0.0, 1.0),
        vec4<f32>(-1.0 + width, 1.0, 0.0, 1.0),

        vec4<f32>(-1.0, 1.0-height, 0.0, 1.0),
        vec4<f32>(-1.0 + width, 1.0, 0.0, 1.0),
        vec4<f32>(-1.0 + width, 1.0-height, 0.0, 1.0)
    );

    // Calculate position of the rectangle based on knob index
    var x_position: f32 = f32(in_instance_index) * 2 / f32(knobs_state.n_knobs);
    var y_position: f32 = 0.0; // Y position can be adjusted as needed

    // Translate the rectangle to its position
    for (var i: u32 = 0u; i < 6u; i = i + 1u) {
        vertices[i].x = vertices[i].x + x_position;
        vertices[i].y = vertices[i].y + y_position;
    }

    // Return the transformed vertex position and the instance index
    var out: VertexOutput;
    out.position = vertices[in_vertex_index % 6u];
    out.instance_index = in_instance_index;
    return out;
}

@fragment
fn fs_main(@location(0) instance_index: u32) -> @location(0) vec4<f32> {
    if (instance_index >= 4u) {
        return vec4<f32>(1.0, 0.2, 0.0, 1.0); // Orange color
    } else {
        return vec4<f32>(1.0, 0.0, 0.0, 1.0); // Red color
    }
}