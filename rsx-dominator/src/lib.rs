#![warn(unreachable_pub)]
//#![deny(warnings)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![cfg_attr(feature = "nightly", allow(incomplete_features))]
#![cfg_attr(feature = "nightly", feature(adt_const_params, generic_const_exprs))]

#[macro_use]
mod macros;
mod bindings;
mod callbacks;
mod dom;
mod fragment;
mod operations;
mod utils;

pub use {dom::*, fragment::*, web_sys::ShadowRootMode};
pub mod animation;
pub mod events;
pub mod routing;
pub mod traits;
