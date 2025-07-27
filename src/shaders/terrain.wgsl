
//see bindings in terrain_material.rs 
 
 //https://github.com/nicopap/bevy_mod_paramap/blob/main/src/parallax_map.wgsl
 
 //#import degen_toon_terrain::toon_lighting::{calculate_toon_lighting,ToonShaderMaterial}

 #import degen_toon_terrain::stochastic_sampling 

 #import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
      mesh_view_bindings::view,

      utils::coords_to_viewport_uv,


      pbr_bindings,
    
    pbr_fragment::pbr_input_from_standard_material,
      pbr_functions::{alpha_discard,calculate_tbn_mikktspace,apply_pbr_lighting, main_pass_post_lighting_processing,
      prepare_world_normal,
      apply_normal_mapping,
      calculate_view

      },
    // we can optionally modify the lit color before post-processing is applied
    pbr_types::{STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT,STANDARD_MATERIAL_FLAGS_UNLIT_BIT},
}

#import bevy_core_pipeline::tonemapping::tone_mapping
  
struct StandardMaterial {
    time: f32,
    base_color: vec4<f32>,
    emissive: vec4<f32>,
    perceptual_roughness: f32,
    metallic: f32,
    reflectance: f32,
    // 'flags' is a bit field indicating various options. u32 is 32 bits so we have up to 32 options.
    flags: u32,
    alpha_cutoff: f32,
};


struct ChunkMaterialUniforms {
    color_texture_expansion_factor: f32 ,
    chunk_uv: vec4<f32>,  //start_x, start_y, end_x, end_y   -- used to subselect a region from the splat texture 
    
};


struct ToolPreviewUniforms { 
    tool_coordinates: vec2<f32>,
    tool_radius: f32,
    tool_color: vec3<f32>    
};






//https://github.com/DGriffin91/bevy_mod_standard_material/blob/main/assets/shaders/pbr.wgsl

/*
@group(1) @binding(1)
var base_color_texture1: texture_2d<f32>;
@group(1) @binding(2)
var base_color_sampler1: sampler;
 

@group(1) @binding(3)
var emissive_texture: texture_2d<f32>;
@group(1) @binding(4)
var emissive_sampler: sampler;

@group(1) @binding(5)
var metallic_roughness_texture: texture_2d<f32>;
@group(1) @binding(6)
var metallic_roughness_sampler: sampler;

@group(1) @binding(7)
var occlusion_texture: texture_2d<f32>;
@group(1) @binding(8)
var occlusion_sampler: sampler;
*/

@group(2) @binding(100) var cel_mask: texture_2d<f32>;
@group(2) @binding(101) var cel_mask_sampler: sampler;



//@group(2) @binding(18)
//var<uniform> toon_material: ToonShaderMaterial;





@group(2) @binding(20)
var<uniform> chunk_uniforms: ChunkMaterialUniforms;

@group(2) @binding(21)
var<uniform> tool_preview_uniforms: ToolPreviewUniforms;

@group(2) @binding(22)
var base_color_texture: texture_2d_array<f32>;   // why is this bound twice..
@group(2) @binding(23)
var base_color_sampler: sampler;

@group(2) @binding(24)
var normal_texture: texture_2d_array<f32>;
@group(2) @binding(25)
var normal_sampler: sampler;

@group(2) @binding(26)
var blend_height_texture: texture_2d_array<f32>;
@group(2) @binding(27)
var blend_height_sampler: sampler;



@group(2) @binding(28)
var secondary_planar_texture: texture_2d_array<f32>;  //for triplanar junk 
@group(2) @binding(29)
var secondary_planar_sampler: sampler;




// see hypersplat.rs 
@group(2) @binding(30)
var splat_index_map_texture: texture_2d<u32>; 
 @group(2) @binding(31)
var splat_index_map_sampler: sampler;

/*
@group(2) @binding(32)
var splat_strength_map_texture: texture_2d<f32>; 
 @group(2) @binding(33)
var splat_strength_map_sampler: sampler;
*/


// we use a separate tex for this for NOW to make collision mesh building far easier (only need height map and not splat)
@group(2) @binding(34)
var height_map_texture: texture_2d<u32>; 
@group(2) @binding(35)
var height_map_sampler: sampler;
  
 

//should consider adding vertex painting to this .. need another binding of course.. performs a color shift 
// this could be used for baked shadows !! THIS IS PROB HOW TIRISFALL GLADES WORKS 
@group(2) @binding(36)
var vertex_color_tint_texture: texture_2d<f32>; 
@group(2) @binding(37)
var vertex_color_tint_sampler: sampler;



@group(2) @binding(38)
var hsv_noise_texture: texture_2d<f32>; 
@group(2) @binding(39)
var hsv_noise_sampler: sampler;



@group(2) @binding(40)
var shadow_noise_texture: texture_2d<f32>; 
@group(2) @binding(41)
var shadow_noise_sampler: sampler;


const BLEND_HEIGHT_OVERRIDE_THRESHOLD:f32 = 0.8;




// ok now just be sure the wrap the secondary texture around SIDEWAYS so it doesnt get stretched weird 

fn triplanar_mapping_lerp_output(
     world_normal: vec3<f32>,
 
   
) -> vec3<f32> {
    // Absolute value of the world normal to determine axis dominance
    let abs_normal = abs(world_normal);

    // Calculate blending weights for each axis (X, Y, Z)
    let sum = abs_normal.x + abs_normal.y + abs_normal.z;
    let weights = abs_normal / sum;

    // Optionally apply a bias or tweak to control blending
//    let x_weight = weights.x; // Contribution of X-axis (sides)
  //  let y_weight = weights.y; // Contribution of Y-axis (sides)
  //  let z_weight = weights.z; // Contribution of Z-axis (top)

    // Combine X and Y weights to control side texture blending
  //  let side_weight = (x_weight + z_weight)  ;


  //some clamping .. dont want so much blending ..
  //if the vector is quite upward facing, just render it as completely upward facing .  
   if weights.y > 0.33 {  
      return vec3<f32>(0.0,1.0,0.0) ;
   } 

    if weights.x > 0.33 {  
      return vec3<f32>(1.0,0.0,0.0) ;
    } 

    if weights.z > 0.33 {  
      return vec3<f32>(0.0,0.0,1.0) ;
   } 
 


  // the y weight is used for the normal diffuse tex !! 
    return  weights;
}
 





@fragment
fn fragment(
    mesh: VertexOutput,
    
     
    @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {
    
   
    //let tiled_uv = chunk_uniforms.color_texture_expansion_factor*mesh.uv;  //cannot get this binding to work !? 
    let tiled_uv =  chunk_uniforms.color_texture_expansion_factor*mesh.uv;
    


 
  // var special_mesh_uv =  stochastic_sampling::get_stochastic_sample_uv( mesh.uv  , 0.5);
  //   special_mesh_uv = clamp(special_mesh_uv, vec2<f32>(0.0), vec2<f32>(1.0));


    
    // seems to be working !! yay ! makes our splat texture encompass all of the chunks 
    var  splat_uv = chunk_uniforms.chunk_uv.xy + mesh.uv * (chunk_uniforms.chunk_uv.zw - chunk_uniforms.chunk_uv.xy);
    
    //let splat_values = textureSample(splat_map_texture, splat_map_sampler, splat_uv );
        


  //  let s = smoothstep(0.4, 0.6, sin(globals.time * speed));
   // splat_uv =  stochastic_sampling::get_stochastic_sample_uv( splat_uv  , 0.5);  //uses simplex noise and sin/cos...
   // splat_uv = clamp(splat_uv, vec2<f32>(0.0), vec2<f32>(1.0));
 

      // Apply stochastic sampling to splat_uv
      //  splat_uv = stochastic_sampling::get_stochastic_sample_uv(splat_uv, 0.5);

        // Clamp splat_uv to ensure valid texture sampling
     //   splat_uv = clamp(splat_uv, vec2<f32>(0.0), vec2<f32>(1.0));




    let height_map_texture_dimensions = textureDimensions(height_map_texture);
   
    let height_map_sample_coord  = vec2<i32>(
        i32(splat_uv.x * f32(height_map_texture_dimensions.x)),
        i32(splat_uv.y * f32(height_map_texture_dimensions.y))
    );

          
    var height_map_value: u32  = textureLoad(height_map_texture,   vec2<i32>(  height_map_sample_coord  ) , 0 ).r;





    let hsv_noise_sample = textureSample(hsv_noise_texture, hsv_noise_sampler, splat_uv  ) ;




    // ----- sample splat ----

    let splat_map_texture_dimensions = textureDimensions(splat_index_map_texture);

    let noise_distortion_amount = 2.0;
    

     //we do this to 'sloppily' sample to break up the pixelation 
    let splat_uv_noise_1 =   vec2<f32>( hsv_noise_sample .r  , hsv_noise_sample .g ) * noise_distortion_amount * 0.5  ;

    let splat_map_sample_coord_float_1 = vec2<f32>(
      splat_uv.x * f32(splat_map_texture_dimensions.x)  + splat_uv_noise_1.x - noise_distortion_amount,
      splat_uv.y * f32(splat_map_texture_dimensions.y)  + splat_uv_noise_1.y - noise_distortion_amount 
    );
    

       let clamped_splat_map_sample_coord_float_1 = clamp(
        splat_map_sample_coord_float_1,
        vec2<f32>(0.0, 0.0),
        vec2<f32>(
            f32(splat_map_texture_dimensions.x - 1), 
            f32(splat_map_texture_dimensions.y - 1)
        )
    );

    let splat_map_sample_coord  = vec2<i32>(
        i32(  clamped_splat_map_sample_coord_float_1.x    ),
        i32(  clamped_splat_map_sample_coord_float_1.y   ),
    );





     let splat_index_values_at_pixel :vec4<u32> = textureLoad(splat_index_map_texture,   vec2<i32>(  splat_map_sample_coord  ) , 0 ).rgba;


     //we do this to 'sloppily' sample to break up the pixelation AGAIN and mix :D for even more slop
    let splat_uv_noise_2 =   vec2<f32>( hsv_noise_sample .b  , hsv_noise_sample .a ) * noise_distortion_amount * 0.5  ;


    let splat_map_sample_coord_float_2 = vec2<f32>(
      splat_uv.x * f32(splat_map_texture_dimensions.x)  + splat_uv_noise_2.x - noise_distortion_amount,
      splat_uv.y * f32(splat_map_texture_dimensions.y)  + splat_uv_noise_2.y - noise_distortion_amount 
    );

    let clamped_splat_map_sample_coord_float_2 = clamp(
        splat_map_sample_coord_float_2,
        vec2<f32>(0.0, 0.0),
        vec2<f32>(
            f32(splat_map_texture_dimensions.x - 1), 
            f32(splat_map_texture_dimensions.y - 1)
        )
    );

    let splat_map_sample_coord_distorted  = vec2<i32>(
        i32(  clamped_splat_map_sample_coord_float_2.x    ),
        i32(  clamped_splat_map_sample_coord_float_2.y   ),
    );
 


      let splat_index_values_at_pixel_distorted :vec4<u32> = textureLoad(splat_index_map_texture,   vec2<i32>(  splat_map_sample_coord_distorted  ) , 0 ).rgba;

    

 //    let splat_strength_values_at_pixel :vec4<f32> = textureSample(splat_strength_map_texture, splat_strength_map_sampler, splat_uv ).rgba;


     // --------- 



    //let alpha_mask_value = textureSample(alpha_mask_texture, alpha_mask_sampler, splat_uv );  //comes from height map atm but COULD come from splat map now 
    
       //comes from the  control map .. float -> integer 
    //let terrain_layer_index_0 = i32( splat_values.r * 255.0 );     ///* 255.0
    //let terrain_layer_index_1 = i32( splat_values.g * 255.0 );

 

    // Initialize an array to store the splat strength values and the index values
  //  var splat_strength_array: array<f32, 4> = array<f32, 4>(splat_strength_values_at_pixel.x  , splat_strength_values_at_pixel.y, splat_strength_values_at_pixel.z , splat_strength_values_at_pixel.w );

    var splat_index_array: array<u32, 4> = array<u32, 4>(splat_index_values_at_pixel.x, splat_index_values_at_pixel.y, splat_index_values_at_pixel.z, splat_index_values_at_pixel.w);

    var splat_index_array_distorted: array<u32, 4> = array<u32, 4>(splat_index_values_at_pixel_distorted.x, splat_index_values_at_pixel_distorted.y, splat_index_values_at_pixel_distorted.z, splat_index_values_at_pixel_distorted.w);


  
      


      // Initialize texture_layers_used and blended color
    

    var blended_color: vec4<f32> = vec4<f32>(0.0);
    var blended_normal: vec4<f32> = vec4<f32>(0.0);


      
    //is this right ? 
  
    var highest_drawn_pixel_height = 0.0;



    

      let hsv_noise_amount = hsv_noise_sample.r;


      let terrain_layer_A_index =  i32(splat_index_array[0]);
      let terrain_layer_A_index_distorted = i32(splat_index_array_distorted[0]); 


      let terrain_layer_B_index =  i32(splat_index_array[1]);
      let terrain_layer_B_index_distorted = i32(splat_index_array_distorted[1]); 


      let terrain_layer_B_strength = splat_index_array[2] ;
       let terrain_layer_B_strength_distorted = splat_index_array_distorted[2] ;


    // Loop through each layer (max 4 layers)
  //  for (var i: u32 = 0u; i < 2u; i = i + 1u) {

      //  let terrain_layer_index =  i32(splat_index_array[i]);
      //  let terrain_layer_index_distorted = i32(splat_index_array_distorted[i]);  //if this is different than the original, we can blend !!! 


        //if there is only a base layer, it is always full strength. This allows for better blends so the base layer can be low strength (1). 


        
       // var splat_strength = splat_strength_array[i];
  
           let blend_height_strength_f = textureSample(blend_height_texture, blend_height_sampler, tiled_uv, terrain_layer_A_index). r ;
 


 

            // Look up the terrain layer index and sample the corresponding texture
          
            let base_color_from_diffuse = textureSample(base_color_texture, base_color_sampler, tiled_uv, terrain_layer_A_index);
            let base_color_from_normal = textureSample(normal_texture, normal_sampler, tiled_uv, terrain_layer_A_index);  

            let base_color_from_diffuse_distorted = textureSample(base_color_texture, base_color_sampler, tiled_uv, terrain_layer_A_index_distorted);
 
            
            let base_distortion_lerp =   hsv_noise_sample.b    ; // make this lerp be noisy  
            let base_mixed_color_from_diffuse = mix(base_color_from_diffuse_distorted, base_color_from_diffuse , base_distortion_lerp ) ;  //final base color 



                // need these to be based on like .... something else 
           // let secondary_planar_uv = vec2<f32> (tiled_uv.y, tiled_uv.x);
            let secondary_planar_uv_A = vec2<f32>(mesh.world_position.z, mesh.world_position.y) * 0.125;  //use this w x weight 
            let secondary_planar_uv_B = vec2<f32>(mesh.world_position.x, mesh.world_position.y) * 0.125; // use this w z weight 

                /// --- planar stuff 
             let secondary_plane_color_from_diffuse_A = textureSample(secondary_planar_texture, secondary_planar_sampler, secondary_planar_uv_A  , terrain_layer_A_index);
              let secondary_plane_color_from_diffuse_B = textureSample(secondary_planar_texture, secondary_planar_sampler, secondary_planar_uv_B  , terrain_layer_A_index);
            // let secondary_plane_color_from_diffuse_distorted = textureSample(secondary_planar_texture, secondary_planar_sampler,  secondary_planar_uv, terrain_layer_A_index_distorted);


             
             let planar_lerp_weights = triplanar_mapping_lerp_output(  
                    mesh.world_normal.xyz  ,    //add some distortion 
              ) ;

            let planar_lerp_weights_distorted = triplanar_mapping_lerp_output(  
                    mesh.world_normal.xyz + hsv_noise_sample.rgb,    //add some distortion 
              ) ;

 


            let triplanar_color = base_mixed_color_from_diffuse * planar_lerp_weights.y
               + secondary_plane_color_from_diffuse_A * planar_lerp_weights.x
               + secondary_plane_color_from_diffuse_B * planar_lerp_weights.z

            ;
             let triplanar_color_distorted = base_mixed_color_from_diffuse * planar_lerp_weights_distorted.y
               + secondary_plane_color_from_diffuse_A * planar_lerp_weights_distorted.x
               + secondary_plane_color_from_diffuse_B * planar_lerp_weights_distorted.z
            ;

            blended_color = mix( triplanar_color ,  triplanar_color_distorted ,   0.25   );



            


            // -----------


            let secondary_color_from_diffuse = textureSample(base_color_texture, base_color_sampler, tiled_uv, terrain_layer_B_index);

            let secondary_color_from_diffuse_distorted = textureSample(base_color_texture, base_color_sampler, tiled_uv, terrain_layer_B_index_distorted);
    
            let secondary_color_strength = f32( terrain_layer_B_strength ) / 255.0;  
                    // terrain_layer_B_strength_distorted  exists ?? 

            blended_color = mix( blended_color  ,  secondary_color_from_diffuse ,  secondary_color_strength ); 



            // --------



 


            //  blended_color = mix( blended_color , secondary_plane_color_from_diffuse_A ,   planar_lerp_weights.x );
             //  blended_color = mix( blended_color , secondary_plane_color_from_diffuse_B ,   planar_lerp_weights.z );
 
          //   blended_color = vec4<f32>( planar_lerp ,0.0,0.0,1.0   );
           // var splat_strength_float =   splat_strength ;
            //from 0.0 to 1.0 




           //  blended_color = base_mixed_color_from_diffuse;

                blended_normal = base_color_from_normal;
                highest_drawn_pixel_height = blend_height_strength_f;

       
 
/*
            if i == 0u {
                blended_color = mixed_color_from_diffuse;
                blended_normal = color_from_normal;
                highest_drawn_pixel_height = blend_height_strength_f;
            }else {


                 // this junk happens to the secondaryh material !! 


                //renders above 
                 if ( blend_height_strength_f > highest_drawn_pixel_height 
                    || splat_strength_float >  BLEND_HEIGHT_OVERRIDE_THRESHOLD
                 ) {

                    //if we are higher, full render 
                    highest_drawn_pixel_height = blend_height_strength_f;
                    splat_strength_float = splat_strength_float; 

                }else if ( splat_strength_float > hsv_noise_amount   ) {
  
                    //if we are rendering below , some pixels will kind of show through w noise 
                    splat_strength_float = splat_strength_float * hsv_noise_amount ; 

                }else {
                    //artificially reduce our splat strength since we are below and unlucky --  
                   splat_strength_float = splat_strength_float * hsv_noise_amount  * hsv_noise_amount ; 
                }   


                // smooth out the blending slightly 
                let splat_strength_float_noisy =   splat_strength_float  + (hsv_noise_amount * 0.2 )  - 0.1  ;

                  // Accumulate the blended color based on splat strength 
                blended_color = mix( blended_color, mixed_color_from_diffuse,  splat_strength_float_noisy );
                 blended_normal = mix( blended_normal, color_from_normal, splat_strength_float  );

               
            }*/

          //  blended_color = vec4<f32>( hsv_noise_sample.r ,0.0,0.0,1.0   );
             
        
// }

    


    

    blended_normal = normalize(blended_normal); 
                    
   let blended_normal_vec3 = vec3<f32>( blended_normal.r, blended_normal.g, blended_normal.b );         
   
  // generate a PbrInput struct from the StandardMaterial bindings
  //remove this fn to make things faster as it duplicates work in gpu .. 
   var pbr_input = pbr_input_from_standard_material(mesh, is_front);
     
    
 
    //hack the material (StandardMaterialUniform)  so the color is from the terrain splat 
    pbr_input.material.base_color =  blended_color;

    //test for now 
   // pbr_input.material.base_color   = vec4(blended_normal_vec3.r,blended_normal_vec3.g,blended_normal_vec3.b,1.0);
    
    let double_sided = (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;



     
     
    pbr_input.world_position = mesh.world_position ;
    pbr_input.world_normal =  prepare_world_normal(
        mesh.world_normal ,
        double_sided,
        is_front,
    );

// https://github.com/bevyengine/bevy/blob/main/assets/shaders/array_texture.wgsl 
    
    
    let normal_from_material = normalize( blended_normal_vec3 );




    let world_normal_with_noise = normalize( mesh.world_normal   +  (   hsv_noise_sample.rgb * 0.2));


    //we mix the normal with our sample so shadows are affected by the normal map ! 
    let normal_mixed = mix( world_normal_with_noise  , normalize( normal_from_material ) , 0.9 );



    pbr_input.N  = normal_mixed;

    /*
    let tangent = normalize( blended_normal_vec3 );

    //we mix the normal with our sample so shadows are affected by the normal map ! 
    let normal_mixed = mix( normalize( mesh.world_normal ) , normalize( tangent ) , 0.7 );



    pbr_input.N  = normal_mixed;


    pbr_input.V =  calculate_view(mesh.world_position, pbr_input.is_orthographic);

  
    // apply lighting
  


   */




         //let viewport_uv = coords_to_viewport_uv(mesh.position.xy, view.viewport);
           
            
        
        let shadow_color = vec4<f32>(0.2,0.2,0.2,1.0);

        let highlight_color = vec4<f32>(1.0, 1.0, 1.0, 1.0);

  
    let base_color = pbr_input.material.base_color;   //save this for later !! 
     pbr_input.material.base_color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    
      var pbr_out: FragmentOutput; 
     pbr_out.color =  apply_pbr_lighting(pbr_input);  // apply lighting to the fake white image 
   
         // This prevents bright lights from causing values > 1.0
     pbr_out.color = clamp(pbr_out.color, vec4<f32>(0.0), vec4<f32>(0.9));


     let lighting_average  = (pbr_out.color.r + pbr_out.color.g + pbr_out.color.b ) / 3.0 ;

      let saturated_lighting_average = saturate(lighting_average);


       let cel_mask_uv = vec2<f32>(saturated_lighting_average, 0.0);
       let quantization = textureSample(cel_mask, cel_mask_sampler, cel_mask_uv);
        pbr_out.color = mix(shadow_color, highlight_color, quantization);


        /*

             let eye = normalize(view_bindings::view.world_position.xyz - in.world_position.xyz);
            let rim = 1.0 - abs(dot(eye, in.world_normal));
            let rim_factor = rim * rim * rim * rim;
            out.color = mix(out.color, rim_color, rim_factor);
        */

     pbr_out.color = pbr_out.color * base_color;   // re-add our true base color from before light calcs  ! 
       pbr_input.material.base_color = base_color;


       pbr_out.color = main_pass_post_lighting_processing(pbr_input, pbr_out.color);



   
    let vertex_world_psn = mesh.world_position.xz; // Assuming the vertex position is in world space

    let tool_coordinates = tool_preview_uniforms.tool_coordinates;
    let tool_radius = tool_preview_uniforms.tool_radius;
    let color_from_tool = tool_preview_uniforms.tool_color;

    let distance = length(vertex_world_psn - tool_coordinates);

    let within_tool_radius = f32(distance <= tool_radius);

    var final_color = mix(
        vec4(pbr_out.color.rgb, 1.0),
        vec4(pbr_out.color.rgb * color_from_tool, 1.0),
        within_tool_radius
    );
          

      // Implement alpha masking
    if (height_map_value < 8) { // Use your threshold value here
        discard;
    }
        

      final_color = clamp(final_color, vec4<f32>(0.0), vec4<f32>(1.0));
    
    return final_color;
    
}
 



 //mod the UV using parallax 
  // https://github.com/nicopap/bevy_mod_paramap/blob/main/src/parallax_map.wgsl

 //later ? 
/*
 fn quantize_color(color: vec4<f32>, bands: u32) -> vec4<f32> {
    // Ensure at least one band (no quantization if bands == 1)
    let num_bands = max(bands, 1u);

    // Calculate the band size
    let band_size = 1.0 / f32(num_bands);

    // Quantize each color component separately (RGB only, leave alpha untouched)
    let quantized_rgb = floor(color.rgb / band_size) * band_size;

    // Return quantized color with original alpha
    return vec4<f32>(quantized_rgb, color.a);
}
*/


/*
fn quantize_color(color: vec4<f32>, bands: u32) -> vec4<f32> {
    // Ensure at least one band (no quantization if bands <= 1)
    let num_bands = max(bands, 1u);

    // Calculate the band size
    let band_factor = f32(num_bands);

    // Guard against very small band sizes to avoid precision issues
    if (band_factor <= 0.0) {
        return color;
    }

    // Quantize each color component separately (RGB only, leave alpha untouched)
    let quantized_rgb = floor(color.rgb * band_factor) / band_factor;

    // Return quantized color with original alpha
    return vec4<f32>(quantized_rgb, color.a);
}*/