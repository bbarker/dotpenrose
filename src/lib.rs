#![warn(clippy::all)]
#![warn(future_incompatible, rust_2024_compatibility)]
// #![deny(unused_crate_dependencies)]

pub mod bar;

pub const FONT: &str = "DejaVu Sans Mono";

// Kanagawa
// https://github.com/rebelot/kanagawa.nvim?tab=readme-ov-file#color-palette
pub const BLACK: u32 = 0x252535ff; // #252535
pub const WHITE: u32 = 0xdcd7baff; // #dcd7ba
pub const GREY: u32 = 0x363646ff; //  #363646
pub const BLUE: u32 = 0x658594ff; //  #658594
pub const RED: u32 = 0xc34043ff; //   #C34043
