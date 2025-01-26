 
use bevy::asset::VisitAssetDependencies;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::*;

use bevy::pbr::ExtendedMaterial;

use bevy::render::render_asset::RenderAssets;

use bevy::pbr::StandardMaterialFlags;
use bevy::pbr::StandardMaterialUniform;

use bevy::pbr::MaterialExtension;


pub type TerrainMaterialExtension = ExtendedMaterial<StandardMaterial, TerrainMaterial>;



pub const TOON_LIGHTING_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(5475846082284647579);
pub const TERRAIN_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(5433284082028047579);

 

#[derive(Clone, ShaderType, Default, Debug)]
pub struct ChunkMaterialUniforms {
    pub color_texture_expansion_factor: f32,
    pub chunk_uv: Vec4, //start_x, start_y, end_x, end_y   -- used to subselect a region from the splat texture
}

#[derive(Clone, ShaderType, Default, Debug)]
pub struct ToolPreviewUniforms {
    pub tool_coordinates: Vec2,
    pub tool_radius: f32,
    pub tool_color: Vec3,
}


#[derive(Clone, ShaderType,  Debug)]
pub struct ToonShaderMaterial {
    pub color: Vec4,
    pub sun_dir: Vec3,
    pub sun_color: Vec4,
    pub ambient_color: Vec4,

}
impl Default for ToonShaderMaterial {


  fn default() -> Self {
        Self {
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),          // White base color
            sun_dir: Vec3::new(0.3, -0.8, 0.5).normalize(), // Tilted sunlight (afternoon angle)
            sun_color: Vec4::new(1.0, 0.95, 0.85, 1.0),    // Warm sunlight with a slight yellow tint
            ambient_color: Vec4::new(0.3, 0.3, 0.4, 1.0),  // Slightly bluish ambient light for realism
        }
    }

}



#[derive(Asset, AsBindGroup, TypePath, Clone, Debug, Default)]
pub struct TerrainMaterial {

     #[uniform(18)]
    pub toon_material: ToonShaderMaterial,



    #[uniform(20)]
    pub chunk_uniforms: ChunkMaterialUniforms,

    #[uniform(21)]
    pub tool_preview_uniforms: ToolPreviewUniforms,

    #[texture(22, dimension = "2d_array")]
    #[sampler(23)]
    pub diffuse_texture: Option<Handle<Image>>,

    #[texture(24, dimension = "2d_array")]
    #[sampler(25)]
    pub normal_texture: Option<Handle<Image>>,

    #[texture(26, dimension = "2d_array" )]
    #[sampler(27  )]
    pub blend_height_texture: Option<Handle<Image>>,




    #[texture(30, dimension = "2d",sample_type = "u_int")] //rgba8uint
    #[sampler(31 , sampler_type = "non_filtering")]
    pub splat_index_map_texture: Option<Handle<Image>>,

    
    #[texture(32, dimension = "2d" )]
    #[sampler(33 )]
    pub splat_strength_map_texture: Option<Handle<Image>>,

 


    #[texture(34, dimension = "2d",sample_type = "u_int")]  //rgba8uint
    #[sampler(35 , sampler_type = "non_filtering")]
    pub height_map_texture: Option<Handle<Image>>,

    // not used ? 
    #[texture(36)]
    #[sampler(37)]
    pub vertex_color_tint_texture: Option<Handle<Image>>,


    #[texture(38)]
    #[sampler(39)]
    pub hsv_noise_texture: Option<Handle<Image>>,


}

impl MaterialExtension for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(TERRAIN_SHADER_HANDLE)
    }

    fn deferred_fragment_shader() -> ShaderRef {
        ShaderRef::Handle(TERRAIN_SHADER_HANDLE)
    }
}


 

// ------- 

#[derive(Component)]
pub struct ToonShaderSun;

pub fn update_toon_shader(
 //   main_cam: Query<&Transform, With<ToonShaderMainCamera>>,
    sun: Query<(&Transform, &DirectionalLight), With<ToonShaderSun>>,
    ambient_light: Option<Res<AmbientLight>>,
    mut toon_materials: ResMut<Assets<TerrainMaterialExtension>>,
) {
    for (_, terrain_mat) in toon_materials.iter_mut() {
       /* if let Ok(cam_t) = main_cam.get_single() {
            toon_mat.camera_pos = cam_t.translation;
        }*/
        if let Ok((sun_t, dir_light)) = sun.get_single() {
            terrain_mat.extension.toon_material.sun_dir = *sun_t.back();
            terrain_mat.extension.toon_material.sun_color = dir_light.color.to_srgba().to_vec4();
        }
        if let Some(light) = &ambient_light {
            terrain_mat.extension.toon_material.ambient_color = light.color.to_srgba().to_vec4();
        }
    }
}