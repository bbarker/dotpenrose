#![warn(clippy::all)]
#![warn(future_incompatible, rust_2024_compatibility)]

use once_cell::sync::Lazy;
use std::ops::RangeInclusive;
// #![deny(unused_crate_dependencies)]
pub mod bar;
pub mod workspaces;

pub const FONT: &str = "Hasklug Nerd Font Mono";

// Kanagawa
// https://github.com/rebelot/kanagawa.nvim?tab=readme-ov-file#color-palette
pub const BLACK: u32 = 0x252535ff; // #252535
pub const WHITE: u32 = 0xdcd7baff; // #dcd7ba
pub const GREY: u32 = 0x363646ff; //  #363646
pub const BLUE: u32 = 0x658594ff; //  #658594
pub const RED: u32 = 0xc34043ff; //   #C34043

// Let's start with 29 tags

pub const NUM_FAST_ACCESS_WORKSPACES: u16 = 9;
pub const NUM_WORKSPACES: u16 = NUM_FAST_ACCESS_WORKSPACES + 20;
pub const WORKSPACES: RangeInclusive<u16> = 1..=(NUM_WORKSPACES);
pub static ALL_TAGS: Lazy<Vec<String>> =
    Lazy::new(|| WORKSPACES.map(|ix| ix.to_string()).collect());
