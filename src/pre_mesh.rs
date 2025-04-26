 
use crate::chunk::CardinalDirection;

use bevy::platform::collections::hash_map::HashMap;

use bevy::prelude::Vec3;
use crate::heightmap::HeightMapU16;
use bevy::prelude::{Mesh, Vec2};
use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology::TriangleList;

 /*
   using INFO or WARN in here will cause a segfault since it is in an async thread 
 */


  const MAX_RECURSION_AT_HIGHEST_LOD: u8  = 3;

const THRESHOLD: u16 = (0.0001 * 65535.0) as u16;

pub struct PreMesh {
    positions: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
   // normals: Vec<[f32; 3]>,
    indices: Vec<u32>,
}

impl PreMesh {
    fn new() -> Self {
        Self {
            positions: Vec::new(),
            uvs: Vec::new(),
         //   normals: Vec::new(),
            indices: Vec::new(),
        }
    }


    // use the fn that is in mesh !! 
 /*   fn calculate_smooth_normals(&mut self) {
        let mut vertex_normals_accum: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0]; self.positions.len()];

        // Step 1: Calculate face normals and accumulate them for each vertex
        for i in (0..self.indices.len()).step_by(3) {
            let idx0 = self.indices[i] as usize;
            let idx1 = self.indices[i + 1] as usize;
            let idx2 = self.indices[i + 2] as usize;

            let v0 = self.positions[idx0];
            let v1 = self.positions[idx1];
            let v2 = self.positions[idx2];

            let normal = compute_normal(v0, v1, v2);

            // Step 2: Accumulate normals for each vertex of the face
            for &idx in &[idx0, idx1, idx2] {
                vertex_normals_accum[idx][0] += normal[0];
                vertex_normals_accum[idx][1] += normal[1];
                vertex_normals_accum[idx][2] += normal[2];
            }
        }

        // Step 3: Normalize accumulated normals to average them
        for normal in vertex_normals_accum.iter_mut() {
            let len =
                f32::sqrt(normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]);
            if len > 0.0 {
                normal[0] /= len;
                normal[1] /= len;
                normal[2] /= len;
            }
        }

        // Step 4: Assign averaged normals to the mesh
        self.normals = vertex_normals_accum;
    }*/


fn compute_smooth_normals(&self) -> Vec<[f32; 3]> {
    let mut normals: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0]; self.positions.len()];
    let mut count: Vec<u32> = vec![0; self.positions.len()];

    // Iterate over each triangle using the indices
    for triangle in self.indices.chunks(3) {
        if let &[i0, i1, i2] = triangle {
            // Get the positions of the triangle vertices
            let p0 = self.positions[i0 as usize];
            let p1 = self.positions[i1 as usize];
            let p2 = self.positions[i2 as usize];

            // Calculate the normal for the triangle
            let v0 = [
                p1[0] - p0[0],
                p1[1] - p0[1],
                p1[2] - p0[2],
            ];
            let v1 = [
                p2[0] - p0[0],
                p2[1] - p0[1],
                p2[2] - p0[2],
            ];
            let normal = [
                v0[1] * v1[2] - v0[2] * v1[1],
                v0[2] * v1[0] - v0[0] * v1[2],
                v0[0] * v1[1] - v0[1] * v1[0],
            ];

            // Accumulate the normal into each vertex's normal
            for &index in &[i0, i1, i2] {
                let idx = index as usize;
                normals[idx][0] += normal[0];
                normals[idx][1] += normal[1];
                normals[idx][2] += normal[2];
                count[idx] += 1;
            }
        }
    }

    // Normalize the normals
    for (normal, &c) in normals.iter_mut().zip(&count) {
        if c > 0 {
            let scale = 1.0 / (c as f32);
            normal[0] *= scale;
            normal[1] *= scale;
            normal[2] *= scale;

            // Normalize the vector length
            let length = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
            if length > 0.0 {
                normal[0] /= length;
                normal[1] /= length;
                normal[2] /= length;
            }
        }
    }

    normals
}



    fn add_triangle(&mut self, positions: [[f32; 3]; 3], uvs: [[f32; 2]; 3]) {
        // Add vertices and indices
        for psn in &positions {
            //   println!("psn {:?}", psn);
            self.positions.push(*psn);
        }
        let start_idx = self.positions.len() as u32 - 3;
        self.indices
            .extend(&[start_idx, start_idx + 1, start_idx + 2]);

        //stubbed in for now ...
        // let normal = compute_normal(positions[0], positions[1], positions[2]);
        //self.normals.extend([normal, normal, normal]);

        self.uvs.extend(uvs);
    }

    pub fn build(self) -> Mesh {
        let mut mesh = Mesh::new(TriangleList, RenderAssetUsages::default());

        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.compute_smooth_normals());


        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
     
        mesh.insert_indices(Indices::U32(self.indices));




       // mesh.generate_tangents().unwrap(); 
        mesh   //doesnt work !? 
    }


    fn get_recursion_amount( lod_level: u8 ) -> u8 {

         (MAX_RECURSION_AT_HIGHEST_LOD - lod_level ).clamp( 0, 3 ) 

    }

    /*

    Could improve this so that NO MATTER WHAT lod level, the edges are never decimated at all and always full resolution.  Only decimate the middle. (interim fix for stitching) .

    */
    pub fn from_heightmap_subsection(
        sub_heightmap: & HeightMapU16,

        height_scale: f32,
        lod_level: u8, // 0 is full quality, higher levels decimate the mesh

        texture_dimensions: [f32; 2],

         adjacent_chunk_lods: HashMap<CardinalDirection, u8>
    ) -> Self {
        let mut premesh = Self::new();

       // println!("using lod level {:?}", lod_level);

      //  let step_size = 1 << lod_level; // doubles the step for each LOD level using bit shifting

        let height_data = &sub_heightmap ;
        

      //  let bounds_pct: [[f32; 2]; 2] = [[0.0, 0.0], [1.0, 1.0]]; //1.0 is the max right ?

      //  let sub_heightmap_height = height_data.len();
     //   let sub_heightmap_width = height_data[0].len();

        //println!("sub_heightmap_width {}", sub_heightmap_width);
        //println!("sub_heightmap_height {}", sub_heightmap_height);

        let tex_dim_x = texture_dimensions.get(0).unwrap().clone();
        let tex_dim_y = texture_dimensions.get(1).unwrap().clone();




        //tris completely below this will be skipped
       // let scaled_min_threshold = (THRESHOLD as f32) * height_scale;


          let threshold =  THRESHOLD as f32; // Adjust based on desired flatness sensitivity


          
             let max_recursion = Self::get_recursion_amount( lod_level ) ; // Limit recursion depth to prevent over-tessellation

             println!("max_recursion {}", max_recursion);

             let step_size = 1 << MAX_RECURSION_AT_HIGHEST_LOD ;   //base step size is static for true adaptive meshing 



             // if we arae on an edge, figure out the cardinality of that edge  and ALSO  figure out the LOD of the chunk in that direction 



        //there is a weird bug where there are gaps in betweeen each chunk ...
        for x in (0..(tex_dim_x as usize - step_size) as usize).step_by(step_size) {
         for z in (0..(tex_dim_y as usize - step_size) as usize).step_by(step_size) {
            



                // Check if sampling is allowed
                if x + step_size >= height_data[0].len() || z + step_size >= height_data.len() {
                    continue;
                }

                // Sample heights
                let lb = height_data[z][x]    ;
                let lf = height_data[z + step_size][x]  ;
                let rb = height_data[z][x + step_size]  ;
                let rf = height_data[z + step_size][x + step_size]  ;


                //adaptive tesselation 
            Self::refine_tile(
                &mut premesh,
                 sub_heightmap,
                 &adjacent_chunk_lods,
                texture_dimensions,
                x,
                z,
                step_size,
                lod_level,
                height_scale,
                threshold,
                max_recursion,
                0,

                lb as f32, lf  as f32, rb  as f32, rf   as f32
            );


           
            }
        }

        //this will happen later.. 
       // premesh.calculate_smooth_normals();

        premesh
    }




// build tile recursively with adaptive tesselation 

fn refine_tile(
    premesh: &mut Self,
    height_data: &HeightMapU16,
    adjacent_chunk_lods: &HashMap<CardinalDirection, u8>,
    texture_dimensions: [f32; 2],
    x: usize,
    z: usize,
    step_size: usize,
    lod_level: u8,
    height_scale: f32,
    threshold: f32, // not used rn
    max_recursion: u8,
    recursion_level: u8,
    lb: f32,
    lf: f32,
    rb: f32,
    rf: f32,
) {
    let fx = x as f32;
    let fz = z as f32;

    // Check if the section is flat
    let max_height = lb.max(lf).max(rb).max(rf);
    let min_height = lb.min(lf).min(rb).min(rf);
    let mut flat_section = (max_height - min_height) < threshold;

    // Detect if the tile is at a chunk edge and fetch neighbor LOD
    let mut chunk_edge_cardinality: Option<CardinalDirection> = None;
    let mut neighbor_lod: Option<u8> = None;

    if x == 0 {
        chunk_edge_cardinality = Some(CardinalDirection::West);
    } else if x == (texture_dimensions[0] as usize - 1) {
        chunk_edge_cardinality = Some(CardinalDirection::East);
    }

    if z == 0 {
        chunk_edge_cardinality = Some(CardinalDirection::North);
    } else if z == (texture_dimensions[1] as usize - 1) {
        chunk_edge_cardinality = Some(CardinalDirection::South);
    }

    if let Some(direction) = &chunk_edge_cardinality {
        neighbor_lod = adjacent_chunk_lods.get(direction).cloned();
    }

    // Adjust recursion depth for LOD stitching
    let neighbor_recursion_amount = neighbor_lod.map(Self::get_recursion_amount);
    let local_max_recursion = neighbor_recursion_amount.unwrap_or(max_recursion);

    if neighbor_recursion_amount.is_some() {
        flat_section = false; // Force refinement at chunk edges to ensure proper stitching
    }

    let should_build_tile = flat_section || recursion_level >= local_max_recursion;



    {

        let center = (lb + lf + rb + rf) / 4.0;
         if center < 1.1 { return };// do not render totally flat tiles - they are hidden

    }


    if should_build_tile {
        let use_extreme_resolution = recursion_level == max_recursion;

        if use_extreme_resolution {
            // More vertices for better transition
            let step_half = step_size as f32 / 2.0;

            let center = (lb + lf + rb + rf) / 4.0;
            let left_mid = (lb + lf) / 2.0;
            let right_mid = (rb + rf) / 2.0;
            let forward_mid = (lf + rf) / 2.0;
            let back_mid = (lb + rb) / 2.0;

            let uv_lb = compute_uv(fx, fz, texture_dimensions);
            let uv_rb = compute_uv(fx + step_size as f32, fz, texture_dimensions);
            let uv_rf = compute_uv(fx + step_size as f32, fz + step_size as f32, texture_dimensions);
            let uv_lf = compute_uv(fx, fz + step_size as f32, texture_dimensions);
            let uv_center = compute_uv(fx + step_half, fz + step_half, texture_dimensions);

            let left_back = [fx, lb * height_scale, fz];
            let right_back = [fx + step_size as f32, rb * height_scale, fz];
            let right_front = [fx + step_size as f32, rf * height_scale, fz + step_size as f32];
            let left_front = [fx, lf * height_scale, fz + step_size as f32];
            let center_pos = [fx + step_half, center * height_scale, fz + step_half];

            premesh.add_triangle([left_back, left_front, center_pos], [uv_lb, uv_lf, uv_center]);
            premesh.add_triangle([left_front, right_front, center_pos], [uv_lf, uv_rf, uv_center]);
            premesh.add_triangle([right_front, right_back, center_pos], [uv_rf, uv_rb, uv_center]);
            premesh.add_triangle([right_back, left_back, center_pos], [uv_rb, uv_lb, uv_center]);

        } else {
            // Render flat tile early
            let uv_lb = compute_uv(fx, fz, texture_dimensions);
            let uv_rb = compute_uv(fx + step_size as f32, fz, texture_dimensions);
            let uv_rf = compute_uv(fx + step_size as f32, fz + step_size as f32, texture_dimensions);
            let uv_lf = compute_uv(fx, fz + step_size as f32, texture_dimensions);

            let lb_scaled = lb * height_scale;
            let rb_scaled = rb * height_scale;
            let rf_scaled = rf * height_scale;
            let lf_scaled = lf * height_scale;

            let left_back = [fx, lb_scaled, fz];
            let right_back = [fx + step_size as f32, rb_scaled, fz];
            let right_front = [fx + step_size as f32, rf_scaled, fz + step_size as f32];
            let left_front = [fx, lf_scaled, fz + step_size as f32];

            premesh.add_triangle([left_front, right_back, left_back], [uv_lf, uv_rb, uv_lb]);
            premesh.add_triangle([right_front, right_back, left_front], [uv_rf, uv_rb, uv_lf]);
        }
    } else {
        // Recursive subdivision for adaptive tessellation
        let half_step = step_size / 2;

    /*    let center_sampled = height_data[z + half_step][x + half_step] as f32;
        let left_mid_sampled = height_data[z + half_step][x] as f32;
        let right_mid_sampled = height_data[z + half_step][x + step_size] as f32;
        let forward_mid_sampled = height_data[z + step_size][x + half_step] as f32;
        let back_mid_sampled = height_data[z][x + half_step] as f32; */

        let center = (lb + lf + rb + rf) / 4.0;
        let left_mid = (lb + lf) / 2.0;
        let right_mid = (rb + rf) / 2.0;
        let forward_mid = (lf + rf) / 2.0;
        let back_mid = (lb + rb) / 2.0;

       
        Self::refine_tile(premesh, height_data, adjacent_chunk_lods, texture_dimensions, x, z, half_step, lod_level, height_scale, threshold, max_recursion, recursion_level + 1, lb, left_mid, back_mid, center);
        Self::refine_tile(premesh, height_data, adjacent_chunk_lods, texture_dimensions, x + half_step, z, half_step, lod_level, height_scale, threshold, max_recursion, recursion_level + 1, back_mid, center, rb, right_mid);
        Self::refine_tile(premesh, height_data, adjacent_chunk_lods, texture_dimensions, x, z + half_step, half_step, lod_level, height_scale, threshold, max_recursion, recursion_level + 1, left_mid, lf, center, forward_mid);
        Self::refine_tile(premesh, height_data, adjacent_chunk_lods, texture_dimensions, x + half_step, z + half_step, half_step, lod_level, height_scale, threshold, max_recursion, recursion_level + 1, center, forward_mid, right_mid, rf);
    }
}







/*

fn bilinear_interpolate(
    height_data: &HeightMapU16,
    height_scale: f32,
    x: f32,
    z: f32,
) -> f32 {
    let x0 = x.floor() as usize;
    let x1 = x0 + 1;
    let z0 = z.floor() as usize;
    let z1 = z0 + 1;

    let q00 = height_data[z0][x0] as f32 * height_scale;
    let q01 = height_data[z1][x0] as f32 * height_scale;
    let q10 = height_data[z0][x1] as f32 * height_scale;
    let q11 = height_data[z1][x1] as f32 * height_scale;

    let tx = x - x0 as f32;
    let tz = z - z0 as f32;

    let a = q00 * (1.0 - tx) + q10 * tx;
    let b = q01 * (1.0 - tx) + q11 * tx;

    a * (1.0 - tz) + b * tz
}


*/


















    pub fn from_heightmap_subsection_greedy(
        sub_heightmap: & HeightMapU16,

        height_scale: f32,
        lod_level: u8, // 0 is full quality, higher levels decimate the mesh

        texture_dimensions: [f32; 2],

        chunk_lod_map: HashMap<CardinalDirection, u8>,
    ) -> Self {
        let mut premesh = Self::new();

        let step_size = 1 << 2; // doubles the step for each LOD level using bit shifting

        let height_data = &sub_heightmap ;
        //  let start_bound: Vec<usize> = vec![0, 0];

        //   let width = texture_dimensions[0]  ;
        //   let height = texture_dimensions[1]  ;

        // let bounds_pct = sub_heightmap.bounds_pct;

    //    let bounds_pct: [[f32; 2]; 2] = [[0.0, 0.0], [1.0, 1.0]]; //1.0 is the max right ?

        let sub_heightmap_height = height_data.len();
        let sub_heightmap_width = height_data[0].len();

        //println!("sub_heightmap_width {}", sub_heightmap_width);
        //println!("sub_heightmap_height {}", sub_heightmap_height);

        let tex_dim_x = texture_dimensions.get(0).unwrap().clone();
        let tex_dim_y = texture_dimensions.get(1).unwrap().clone();

        let width_scale = 1.0;

        //tris completely below this will be skipped
        let scaled_min_threshold = (THRESHOLD as f32) * height_scale;

        let similarity_threshold = scaled_min_threshold * 2.0;

        for x in (0..(tex_dim_x as usize - step_size) as usize).step_by(step_size) {
            // let mut greedy_y_start:Option<f32> = None;
            let mut current_greedy_height: Option<f32> = None;
            let mut greedy_points_z_start: Option<f32> = None; //fx

            let fx = (x) as f32 * width_scale;

            for y in (0..(tex_dim_y as usize - step_size) as usize).step_by(step_size) {
                let at_end_of_segment = y >= tex_dim_y as usize - (step_size * 2);

                let fz = (y) as f32 * width_scale;

                let mut sample_allowed = true;
                //cant sample so we just continue
                if x + step_size >= sub_heightmap_width as usize {
                    sample_allowed = false;
                    //warn!("x {}", x + step_size);
                    continue;
                }
                if y + step_size >= sub_heightmap_height as usize {
                    sample_allowed = false;
                    //warn!("y {}", y + step_size);
                    continue; 
                }

                // println!( "{} {} {} {} ", x , y , x+step_size, y + step_size   );
                let (mut lb, mut lf, mut rb, mut rf) = match sample_allowed {
                    true => {
                        let lb = height_data[y][x] as f32 * height_scale;
                        let lf = height_data[y+ step_size][x ] as f32 * height_scale;
                        let rb = height_data[y][x + step_size] as f32 * height_scale;
                        let rf = height_data[y + step_size][x + step_size] as f32 * height_scale;
                        (lb, lf, rb, rf)
                    }
                    false => (0.0, 0.0, 0.0, 0.0),
                };

                //if the triangle would be completely under the threshold,
                //do not add it to the mesh at all.  This makes a hole for collision
                //since this mesh is used to generate the collider
                if lb < scaled_min_threshold
                    && lf < scaled_min_threshold
                    && rb < scaled_min_threshold
                    && rf < scaled_min_threshold
                {
                    //this does goofy things w greedy !?
                    // continue;
                }

                if let Some(greedy_height) = current_greedy_height {
                    // let mut total_step_size = step_size.clone();

                    let differences = [
                        (lb - greedy_height).abs(),
                        (lf - greedy_height).abs(),
                        (rf - greedy_height).abs(),
                        (rb - greedy_height).abs(),
                    ];

                    // Check if all differences are within the threshold
                    let current_points_are_similar =
                        differences.iter().all(|&diff| diff <= similarity_threshold);

                    if current_points_are_similar && !at_end_of_segment {
                        //keep going -- continue

                        continue;
                    } else {
                        //end this segment and render the triangles

                        let start_fz = greedy_points_z_start.unwrap();

                        let uv_lb = compute_uv(fx, start_fz, texture_dimensions);
                        let uv_rb = compute_uv(
                            fx + step_size as f32,
                            start_fz,
                           
                            texture_dimensions,
                        );
                        let uv_rf = compute_uv(
                            fx + step_size as f32,
                            start_fz + step_size as f32,
                             
                            texture_dimensions,
                        );
                        let uv_lf = compute_uv(
                            fx,
                            start_fz + step_size as f32,
                            
                            texture_dimensions,
                        );

                        let left_back = [fx, greedy_height, start_fz];
                        let right_back = [fx + step_size as f32, greedy_height, start_fz];
                        let right_front = [fx + step_size as f32, greedy_height, fz];
                        let left_front = [fx, greedy_height, fz];

                       
                         if greedy_height >= scaled_min_threshold 
                        {
                                   premesh.add_triangle(
                                        [left_front, right_back, left_back],
                                        [uv_lf, uv_rb, uv_lb],
                                    );
                                    premesh.add_triangle(
                                        [right_front, right_back, left_front],
                                        [uv_rf, uv_rb, uv_lf],
                                    ); 
                                        
                        }
                          greedy_points_z_start = None;
                        current_greedy_height = None;


                       

                       
                    }
                } else {
                    let differences = [
                        (lb - lf).abs(),
                        (lb - rb).abs(),
                        (lb - rf).abs(),
                        (lf - rb).abs(),
                        (lf - rf).abs(),
                        (rb - rf).abs(),
                    ];

                    // Check if all differences are within the threshold
                    let current_points_are_similar =
                        differences.iter().all(|&diff| diff <= similarity_threshold);

                    // we start the greedy meshing here
                    if current_points_are_similar && !at_end_of_segment {
                        current_greedy_height = Some((lb + lf + rb + rf) / 4.0);
                        greedy_points_z_start = Some(fz);
                        continue;
                    }
                }

                //if the 4 points heights are very similar, skip adding this triangle and instead increment the
                //greedy counter and continue .   lf and rf will be way out, the others will be saved.

                //add normal mini tris here
                let uv_lb = compute_uv(fx, fz, texture_dimensions);
                let uv_rb = compute_uv(fx + step_size as f32, fz,   texture_dimensions);
                let uv_rf = compute_uv(
                    fx + step_size as f32,
                    fz + step_size as f32,
                     
                    texture_dimensions,
                );
                let uv_lf = compute_uv(fx, fz + step_size as f32,   texture_dimensions);

                let left_back = [fx, lb, fz];
                let right_back = [fx + step_size as f32, rb, fz];
                let right_front = [fx + step_size as f32, rf, fz + step_size as f32];
                let left_front = [fx, lf, fz + step_size as f32];


                if lb < scaled_min_threshold
                    && lf < scaled_min_threshold
                    && rb < scaled_min_threshold
                    && rf < scaled_min_threshold
                {
                     
                     continue;
                }
                premesh.add_triangle([left_front, right_back, left_back], [uv_lf, uv_rb, uv_lb]);
                premesh.add_triangle([right_front, right_back, left_front], [uv_rf, uv_rb, uv_lf]);
            } // z loop

            //if there is still a greedy segment left over ... lets render it !
            if let Some(greedy_height) = current_greedy_height {
                if let Some(fz) = greedy_points_z_start {
                    let start_fz = greedy_points_z_start.unwrap();

                    let uv_lb = compute_uv(fx, start_fz,  texture_dimensions);
                    let uv_rb = compute_uv(
                        fx + step_size as f32,
                        start_fz,
                       
                        texture_dimensions,
                    );
                    let uv_rf =
                        compute_uv(fx + step_size as f32, fz,  texture_dimensions);
                    let uv_lf = compute_uv(fx, fz,  texture_dimensions);

                    let left_back = [fx, greedy_height, start_fz];
                    let right_back = [fx + step_size as f32, greedy_height, start_fz];
                    let right_front = [fx + step_size as f32, greedy_height, fz];
                    let left_front = [fx, greedy_height, fz];


                    if greedy_height < scaled_min_threshold 
                        {
                             
                             continue;
                        }


                    premesh
                        .add_triangle([left_front, right_back, left_back], [uv_lf, uv_rb, uv_lb]);
                    premesh
                        .add_triangle([right_front, right_back, left_front], [uv_rf, uv_rb, uv_lf]);
                } //if
            } // x loop
        } // x loop

        //this will be done later ! 
     //   premesh.calculate_smooth_normals();

        premesh
    }
}

fn compute_normal(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

    // Cross product
    let normal = [
        edge1[1] * edge2[2] - edge1[2] * edge2[1],
        edge1[2] * edge2[0] - edge1[0] * edge2[2],
        edge1[0] * edge2[1] - edge1[1] * edge2[0],
    ];

    // normal

    [normal[0], normal[1], normal[2]] //is this busted ?
}

fn compute_uv(x: f32, y: f32,   texture_dimensions: [f32; 2]) -> [f32; 2] {
    

    //x and y are the origin coords

   

    let uv = [
        x / texture_dimensions[0],
        y / texture_dimensions[1],
    ];

    // println!("uv {:?}", uv);

    uv
}

/*
fn compute_uvs(  fx :f32, fz: f32, step_size: usize,   texture_dimensions: [f32; 2] ) -> [[f32; 2];4] {


                let uv_lb = compute_uv(fx, fz,  texture_dimensions);
                let uv_rb = compute_uv(fx + step_size as f32, fz,   texture_dimensions);
                let uv_rf = compute_uv(
                    fx + step_size as f32,
                    fz + step_size as f32,
                     
                    texture_dimensions,
                );
                let uv_lf = compute_uv(fx, fz + step_size as f32,  texture_dimensions);


                [uv_lb, uv_rb, uv_rf, uv_lf]
}
*/