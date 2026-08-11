#[allow(clippy::module_inception)]
mod controls;
pub mod router;

pub use controls::{__path_create_control, __path_get_list_control, __path_get_list_control_paged};
pub use controls::{create_control, get_list_control, get_list_control_paged};
