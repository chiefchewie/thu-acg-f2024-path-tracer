# ray tracing project

## set up:
1. install `rust` and make sure you have `cargo` working too
2. in the root directory, do `cargo run` (debug build) or `cargo run -r` (release build)

## demos:
1. f bouncing balls demonstraing motion blur, textures. please edit main.rs to `x=1` and run. in the definition of `balls_scene`, can change the samples per pixel to 1000 to match demo

2. cornell box. there may be some slight differences (perhaps cubes not spheres) due to me editing the code, but the general scene layout should remain similar. for this, edit `main.rs` to `x=5`. the scene is defined in `cornell_box_scene()`, and in there feel free to up the samples per pixel to 1000 or 5000 to match the demo. 

3. earth image texture map: edit `main.rs` to `x=2`.

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

This link has an implementation, which I'm currently following for the overall layout. However, I plan to change it because there are some things that I don't like about it, such as how they are determining the lobe probabilities, some of the conditions for evaluating each lobe, and I think the sampling procedures can be optimized which I'd like to do. I'd like to acknowledge that my final implementation might still end up similar to this, since we're both implementing the same thing (Disney BRDF) at the end of the day.
https://github.com/knightcrawler25/GLSL-PathTracer/blob/master/src/shaders/common/disney.glsl 

When I get to participating media (volumetric rendering) and multiple importance sampling, I will likely reference Ray Tracing Gems and the pbrt book for their ideas and example implementations.

[![Review Assignment Due Date](https://classroom.github.com/assets/deadline-readme-button-22041afd0340ce965d47ae6ef1cefeee28c7c493a6346c4f15d667ab976d596c.svg)](https://classroom.github.com/a/cPlbGtcU)
