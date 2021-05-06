# Scratch From Scratch (Under development)

This is supposed to be a clone of the Scratch VM. (Not the block editor.)

Requires the `nightly-2020-08-27` toolchain. Only tested in Debian.

```
cargo run vm <path to .sb3 scratch file> # Runs the VM
cargo run viewer <path to .sb3 scratch file> # Outputs information about the Scratch project
```

I used two projects to help guide development: [Mandelbrot](https://scratch.mit.edu/projects/182788/editor/) and [Pixel Snake](https://scratch.mit.edu/projects/72303326/editor/). They run very slowly and Pixel Snake is barely controllable but hey they run at least.