// Function to perform smooth union of two signed distance functions
fn op_smooth_union(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return mix(d2, d1, h) - k * h * (1.0 - h);
}

// Function to compute signed distance of a sphere
fn sd_sphere(p: vec3<f32>, s: f32) -> f32 {
    return length(p) - s;
}

// Function to compute the signed distance field map
fn map(p: vec3<f32>, iTime: f32) -> f32 {
    var d = 2.0;
    let spread = vec3<f32>(10.0,4.0,0.2);
    for (var i: i32 = 0; i < 16; i = i + 1) {
        let fi = f32(i);
        let time = iTime * (fract(fi * 412.531 + 0.513) - 0.5) * 0.8;

        let xRandomOffset = sin(time + fi * 0.1) * 10.0;
        let spherePosition = vec3<f32>(xRandomOffset, sin(time + fi * 10.0), cos(time + fi * 5.0));
        let sphereSize = mix(0.5, 5.0, fract(fi * 300.561 + 0.5124));

        d = op_smooth_union(
            sd_sphere(p + spherePosition * spread, sphereSize),
            d,
            0.4
        );
    }
    return d;
}

// Function to compute the normal at a point by finite differences
fn calc_normal(p: vec3<f32>, iTime: f32) -> vec3<f32> {
    let h = 1e-5;
    let k = vec2<f32>(1.0, -1.0);
    let px = p + vec3<f32>(h, 0.0, 0.0);
    let nx = p + vec3<f32>(-h, 0.0, 0.0);
    let py = p + vec3<f32>(0.0, h, 0.0);
    let ny = p + vec3<f32>(0.0, -h, 0.0);
    let pz = p + vec3<f32>(0.0, 0.0, h);
    let nz = p + vec3<f32>(0.0, 0.0, -h);

    let dx = map(px, iTime) - map(nx, iTime);
    let dy = map(py, iTime) - map(ny, iTime);
    let dz = map(pz, iTime) - map(nz, iTime);

    let normal = vec3<f32>(dx, dy, dz);
    return normalize(normal);
}



const resolution = vec2<f32>(960.0,160.0);

@group(0) @binding(0)
var<storage, read> time: f32;


@vertex
fn vs_main(@builtin(vertex_index) v_ind: u32, @builtin(instance_index) i_ind: u32) -> @builtin(position) vec4<f32> {
    var pos = vec2<f32>(0.0, 0.0);
    if (v_ind == 0u) {
        pos = vec2<f32>(-1.0, -1.0);
    } else if (v_ind == 1u) {
        pos = vec2<f32>( 3.0, -1.0);
    } else if (v_ind == 2u) {
        pos = vec2<f32>(-1.0,  3.0);
    }
    return vec4<f32>(pos, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let uv = (pos.xy /resolution) * 2.0 - 1.0; // Converts to NDC;

  let rayOri = vec3<f32>(uv  * vec2<f32>(resolution.x /resolution.y, 1.0) *  15.0, 3.0);
  let rayDir = vec3<f32>(0.0, 0.0, -1.0);

    var depth = 0.0;
    var p: vec3<f32>;
    var hit = false; // Flag to check if a sphere is hit

    for (var i: i32 = 0; i < 64; i = i + 1) {
        p = rayOri + rayDir * depth;
        let dist = map(p, time);
        depth = depth + dist;
        if (dist < 1e-6) {
            hit = true; // Sphere is hit
            break;
        }
    }
    depth = min(6.0, depth);
    let n = calc_normal(p, time);
    let b = max(0.0, dot(n, vec3<f32>(0.577,0.577,0.577))); // Light direction

    // Define a time-based gradient that transitions between orange and red
    let colorFactor = 0.5 * (1.0 + sin(time * 0.05)); // Oscillates between 0 and 1

    // Map UV coordinates to a gradient factor
    let uvGradientFactor = (uv.x + 1.0) * 0.5; // Map UV x to range [0,1]
    let gradientStart = vec3<f32>(1.0, 0.5, 0.0); // Orange
    let gradientEnd = vec3<f32>(1.0, 0.0, 0.0);   // Red
    let gradientColor = mix(gradientStart, gradientEnd, colorFactor);
    
    let col = mix(gradientColor, gradientColor * (1.0 - uvGradientFactor), 0.5) * (0.85 + b * 0.35);


    /**
    let uv_cos_x = 0.5 + 0.5 * cos(b +  time * 3.0 + uv.x * 2.0);
    let uv_cos_y = 0.5 + 0.5 * cos(b +  time * 3.0 + uv.y * 2.0 + 2.0);
    let uv_cos_z = 0.5 + 0.5 * cos(b +  time * 3.0 + uv.x * 2.0 + 4.0);
    var col = vec3<f32>(uv_cos_x*0.8, uv_cos_y*0.01, uv_cos_z*0.01) * (0.85 + b * 0.35);
    col *= exp( -depth * 0.15 );
    */
    var finalColor = vec3<f32>(0.0, 0.0, 0.0);
    if (hit){
        finalColor = col * exp(-depth * 0.15);
    }

    return vec4<f32>(finalColor, 1.0 - (depth - 0.5) / 2.0);
}

