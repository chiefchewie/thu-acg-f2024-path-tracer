# ray tracing project

## set up:
1. install `rust` and make sure you have `cargo` working too
2. in the root directory, do `cargo run` (debug build) or `cargo run -r` (release build)
or you can run the executable directly

command line arguments
`-q` flag enable this for higher quality rendering. right now, high quality is FHD at 4000 samples per pixel, and low quality is 800 pixels wide at 100 samples per pixel. 

`-s <scene>` pick the scene you would like to see. defaults to 1, which is the bouncing balls.

## demos:
1. bouncing balls demonstraing motion blur, textures. 

2. scene showing image texture map and the depth of field effect

3. cornell box

4. very simple demo of a specular sphere showing HDR skylight environment map

5. Principled BSDF demo - these are all the same material, with varying roughness, metalness

6. A scene I put together showing off as many things as I can think of, also loading of OBJ models. Notice the caustics caused by the environment lighting of the glass ball to the left!

7. Cornell box showing the normal map implementation, the left wall is not using a normal map and the right wall is using a normal map, notice the shadows on the right side.

I have kept the resolution of the demos fairly low for faster iteration.
for all three demos, at 5000 samples per pixel it might take a while (up to 10 minutes on)

# resources/code referenced:
Ray Tracing in One Weekend for the initial framework

The following links were referenced for an implementation of a principled BSDF. I am mostly going by the Disney BSDF as described in their 2012 and 2015 talks. 
https://schuttejoe.github.io/post/disneybsdf/ 
https://cseweb.ucsd.edu/~tzli/cse272/wi2023/homework1.pdf 
https://blog.selfshadow.com/publications/s2012-shading-course/burley/s2012_pbs_disney_brdf_notes_v3.pdf 
https://blog.selfshadow.com/publications/s2015-shading-course/burley/s2015_pbs_disney_bsdf_notes.pdf 
https://boksajak.github.io/blog/BRDF 
https://github.com/knightcrawler25/GLSL-PathTracer/blob/master/src/shaders/common/disney.glsl 

For help understanding importance sampling and multiple importance sampling
https://lisyarus.github.io/blog/posts/multiple-importance-sampling.html#section-importance-sampling

[![Review Assignment Due Date](https://classroom.github.com/assets/deadline-readme-button-22041afd0340ce965d47ae6ef1cefeee28c7c493a6346c4f15d667ab976d596c.svg)](https://classroom.github.com/a/cPlbGtcU)
