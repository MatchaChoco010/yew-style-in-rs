#![doc = include_str!("../README.md")]

#[cfg(not(feature = "dry-run"))]
pub use yew_style_in_rs_macro::style_with_write as style;
#[cfg(feature = "dry-run")]
pub use yew_style_in_rs_macro::style_without_write as style;

#[doc(hidden)]
pub use yew_style_in_rs_core::*;

#[doc(hidden)]
pub mod css;

#[doc(hidden)]
pub mod dyn_css;

#[doc(hidden)]
pub mod runtime_manager;

#[doc(hidden)]
pub mod cursor;
