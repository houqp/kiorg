// Include the generated PDFium bindings

mod bindgen;

#[cfg(feature = "static")]
mod static_lib;
#[cfg(feature = "static")]
pub use static_lib::*;

#[cfg(feature = "dynamic")]
mod dynamic_lib;
#[cfg(feature = "dynamic")]
pub use dynamic_lib::*;
