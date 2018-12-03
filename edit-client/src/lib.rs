#![feature(nll)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate oatie;
#[macro_use]
extern crate serde_derive;
#[allow(unused)]
#[macro_use]
extern crate lazy_static;

// Macros has to come first
#[macro_use]
pub mod log;

#[cfg(target_arch = "wasm32")]
#[macro_use]
pub mod wasm;

pub mod client;
pub mod monkey;
pub mod random;
pub mod walkers;

#[cfg(not(target_arch = "wasm32"))]
pub mod proxy;

pub use self::client::*;
pub use self::random::*;

// Use `wee_alloc` as the global allocator.
#[cfg(all(target_arch = "wasm32", feature = "use_wee_alloc"))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
