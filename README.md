This is my first attempt at a GPU based ray-tracer based on Ray Tracing in One Weekend.

The project is called WiW for Wgpu-Imgui-Winit (the frameworks I used to implement this), 
though I haven't put in a GUI yet as Imgui hasn't been updated to the latest Winit version.

I'm currently only rendering the final scene is RTiOW, though I used a BVH implemented with
the SAH.  I'm also trying to clock the megakernel (both compute and render) using 
write_timestamps, though there seems to be an issue that is creating some problems.  Currently,
I just dump the output of the timing query to the console window.

To do (in no particular order):
- see if I can optimize this render time; I feel like 300ms for this image is too long, and I
want to do real-time rendering
- add some kind of accumulator rather than waiting for all the samples_per_pixel to complete
- figure out how to accurately get framerate and compute times
- implement a wavefront path tracing algorithm
- fix the colors
- add a GUI
- add other shapes (triangles, planes, quads, planes, etc)
- add an interactive camera
- add more complex rendering ideas from PBR book
- load in more complex obj files for some cool pictures
