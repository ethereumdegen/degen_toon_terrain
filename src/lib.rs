 

use crate::terrain_material::STOCHASTIC_SAMPLING_SHADER_HANDLE;
use bevy::asset::embedded_asset;
use bevy::time::common_conditions::on_timer;
use bevy::{asset::load_internal_asset, prelude::*};
/*use chunk::{
    build_chunk_height_data, build_chunk_meshes, finish_chunk_build_tasks, initialize_chunk_data,
    reset_chunk_height_data, update_chunk_visibility, ChunkHeightMapResource,
}; */
use terrain::{initialize_terrain, load_terrain_texture_from_image, load_terrain_normal_from_image,load_terrain_blend_height_from_image};

use std::time::Duration;

//use chunk::{activate_terrain_chunks, destroy_terrain_chunks, despawn_terrain_chunks, build_active_terrain_chunks, finish_chunk_build_tasks, ChunkEvent};
//use collision::spawn_chunk_collision_data;

use crate::terrain_material::TerrainMaterialExtension;
use crate::terrain_material::{TERRAIN_SHADER_HANDLE,TOON_LIGHTING_SHADER_HANDLE,CUSTOM_PBR_FUNCTIONS_SHADER_HANDLE};
 
use terrain_material::{  TerrainMaterial };

use edit::{
    apply_command_events, apply_tool_edits, EditTerrainEvent, TerrainBrushEvent,
    TerrainCommandEvent,
};

pub mod chunk;
//pub mod collision;
pub mod edit;
pub mod heightmap;
pub mod pre_mesh;
pub mod terrain;
pub mod terrain_config;
pub mod terrain_loading_state;
pub mod terrain_material;
pub mod tool_preview;

pub mod splat;

#[derive(Default, Hash,Eq,PartialEq,States,Debug,Clone )] 
pub enum TerrainEditMode {

    #[default]
    TerrainReadOnly,
    TerrainEditable


}

#[derive(Default)]
pub struct TerrainMeshPlugin {
    pub terrain_edit_mode: TerrainEditMode
}  
 



impl Plugin for TerrainMeshPlugin {
    fn build(&self, app: &mut App) {

        let task_update_rate = Duration::from_millis(250);


          load_internal_asset!(
            app,
            TOON_LIGHTING_SHADER_HANDLE,
            "shaders/toon_lighting.wgsl",
            Shader::from_wgsl
        );

          load_internal_asset!(
            app,
            STOCHASTIC_SAMPLING_SHADER_HANDLE,
            "shaders/stochastic_sampling.wgsl",
            Shader::from_wgsl
        );



          load_internal_asset!(
            app,
            CUSTOM_PBR_FUNCTIONS_SHADER_HANDLE,
            "shaders/custom_pbr_functions.wgsl",
            Shader::from_wgsl
        );




        // load terrain shader into cache
        load_internal_asset!(
            app,
            TERRAIN_SHADER_HANDLE,
            "shaders/terrain.wgsl",
            Shader::from_wgsl
        );

         embedded_asset!(app, "src/", "shaders/hsv_noise.png" );
       


         embedded_asset!(app, "src/",  "shaders/cel_mask.png" );

        


        app.insert_state( self.terrain_edit_mode.clone() ) ;
        
        app.add_plugins(MaterialPlugin::<TerrainMaterialExtension>::default());
        app.add_plugins(chunk::chunks_plugin);

        app.add_plugins(splat::splat_plugin);


        app.init_state::<terrain_loading_state::TerrainLoadingState>();

        app.init_resource::<tool_preview::ToolPreviewResource>();

        //app.add_event::<ChunkEvent>();
        app.add_event::<EditTerrainEvent>();
        app.add_event::<TerrainCommandEvent>();
        app.add_event::<TerrainBrushEvent>();
        
  
        app.add_systems(
            Update,
            initialize_terrain.run_if(on_timer(task_update_rate)),
        );

     /*   app.add_systems(
            Update,
            update_toon_shader.run_if( 
              any_with_component::<ToonShaderSun>  .and( resource_exists::<Assets<TerrainMaterialExtension>> ) ) ) // need to load resources THEN the  sun 
        ; */ 
         

        app.add_systems(Update, (
            load_terrain_texture_from_image,
            load_terrain_normal_from_image, 
            load_terrain_blend_height_from_image
            ));

        app.add_systems(Update, apply_tool_edits); //put this in a sub plugin ?
        app.add_systems(Update, apply_command_events);
    }
}
