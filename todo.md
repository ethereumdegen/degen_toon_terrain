



- probably need to store each chunk LOD in a map 
    - then can use that data when building a chunk to rebuild it .



- custom apply_pbr_lighting  for cel shading ..  iterative bands of the shadows that come in from the point and direct lighting !  (bands code from the bevy toon shader) 


- noise for normals on terrain .! 



1.  add a subtool to CLEAR all splat for each pixel   bc there can be too many and it gets too crazy 
2. invent some way to better inspect all of the materials that on a hypersplat pixel..  a better eyedrop  ?

3. lods 
  BUG :  Painting is broken when using 512/512  splat res 
   
   