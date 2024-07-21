#[cfg(layout = "default")]
mod default;
#[cfg(layout = "default")]
use default as selected_layout;

pub use selected_layout::get_input_map;
