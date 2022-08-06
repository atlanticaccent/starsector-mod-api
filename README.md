# Intro

This is a top level workspace intended to hold all code related to the mod info API.

Currently it contains:

- starsector-mod-info
  - This is an edge worker that receives requests from prospective "clients", typically containing payloads
  of what mods a user has installed and what version they are. It _should_ be reachable from the internet.
  _Some amount of work_ (how much may change) is performed on the edge, with the processed result pushed to a
  message broker.
- starsector-mod-info-storage
  - This is an internal worker intended to receive webhook requests from a message oriented middleware service
  (at this time CloudAMQP is the primary candidate). This allows us to solve for a problem I have made for myself,
  where R2 storage is last-write-wins, and so is impossible to asynchronously write to without potentially losing
  data. Our message oriented middleware will instead buffer messages from starsector-mod-info, then write them into
  our backing R2 storage synchronously (our broker will only have a concurrency of 1). This service will necessarily
  need to perform some amount of merging in cases where the primary data key is not unique (which is most of the
  time).
- starsector-mod-info-shared
  - A library containing shared data types and other code.

## WebAssembly

`workers-rs` (the Rust SDK for Cloudflare Workers used in this template) is meant to be executed as 
compiled WebAssembly, and as such so **must** all the code you write and depend upon. All crates and
modules used in Rust-based Workers projects have to compile to the `wasm32-unknown-unknown` triple. 

Read more about this on the [`workers-rs` project README](https://github.com/cloudflare/workers-rs).

## Issues

If you have any problems with the `worker` crate, please open an issue on the upstream project 
issue tracker on the [`workers-rs` repository](https://github.com/cloudflare/workers-rs).

