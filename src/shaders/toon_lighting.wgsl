

#define_import_path degen_toon_terrain::toon_lighting




struct ToonShaderMaterial {
    color: vec4<f32>,  //not used for now 
    sun_dir: vec3<f32>,
    sun_color: vec4<f32>, 
    ambient_color: vec4<f32>,
};


fn calculate_toon_lighting(
    normal: vec3<f32>,
    view_dir: vec3<f32>,
     sun_dir: vec3<f32>,
     sun_color: vec4<f32>,
  
) -> vec4<f32> {
    // Diffuse lighting
    let n_dot_l = dot(sun_dir, normal);
    var light_intensity = 0.0;

    if n_dot_l > 0.0 {
        let bands = 2.0;
        var x = n_dot_l * bands;
        x = round(x);
        light_intensity = x / bands;
    }

    let light = light_intensity * sun_color;

    // Specular lighting
    let half_vector = normalize(sun_dir + view_dir);
    let n_dot_h = dot(normal, half_vector);
    let glossiness = 32.0;
    let specular_intensity = pow(n_dot_h, glossiness * glossiness);

    let specular_intensity_smooth = smoothstep(0.005, 0.01, specular_intensity);
    let specular = specular_intensity_smooth * vec4<f32>(0.9, 0.9, 0.9, 1.0);

    return light + specular;
}