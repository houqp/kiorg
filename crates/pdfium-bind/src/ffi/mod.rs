// Include the generated PDFium bindings

mod bindgen;

#[cfg(feature = "static")]
mod static_lib;
#[cfg(feature = "static")]
pub use static_lib::*;

#[cfg(not(feature = "static"))]
mod dynamic_lib;
#[cfg(not(feature = "static"))]
pub use dynamic_lib::*;
