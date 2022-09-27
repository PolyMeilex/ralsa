# Ralsa
Just a playground for writing ALSA midi code in rust.

The idea is to create ergonomic and Rust firiendly API, by calling kernel APIs directly instead of using libalsa it is possible to enforce thread safety and ownership rules a lot more easily. So TL;DR is: let's use file descriptors instead of raw pointers, and enjoy the benefits.

`alsa-rs` already does that for one of it's modules for the same reason, so the idea is more or less battle proven.

I will mostly focus on MIDI related APIs as that is what I need, but the same idea can be applied to other pats of ALSA as well, and such additions would be welcome.

- `./alsa-ioctl` Bindings for ALSA kernel APIs
- `./ralsa-seq` High level Rust API for MIDI IO
- `./tools` Just some binaries used for testing 
