use raylib::ffi::Font;
use once_cell::sync::OnceCell;

#[derive(Debug)]
pub struct GlobalFont(pub Font);

unsafe impl Sync for GlobalFont {}
unsafe impl Send for GlobalFont {}

pub static GLOBAL_FONT: OnceCell<GlobalFont> = OnceCell::new();

pub static GLOBAL_FONT_SIZE: OnceCell<f32> = OnceCell::new();
