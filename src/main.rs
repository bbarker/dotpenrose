#![allow(clippy::unit_arg)]

use once_cell::sync::Lazy;
use penrose::{
    builtin::{
        actions::{exit, key_handler, modify_with, send_layout_message, spawn},
        layout::{
            messages::{ExpandMain, IncMain, Rotate, ShrinkMain},
            transformers::ReserveTop,
            MainAndStack,
        },
    },
    core::{
        bindings::{parse_keybindings_with_xmodmap, KeyEventHandler},
        layout::LayoutStack,
        Config, WindowManager,
    },
    extensions::{
        hooks::{add_ewmh_hooks, SpawnOnStartup},
        util::dmenu::{DMenu, DMenuConfig, DMenuKind, MenuMatch},
    },
    map, stack,
    x11rb::RustConn,
    Result,
};
use std::ops::Range;
use std::{collections::HashMap, ops::RangeBounds};
use tracing_subscriber::util::SubscriberInitExt;

use dotpenrose::bar::{status_bar, BAR_HEIGHT_PX_PRIMARY};

// Let's start with 29 tags
const WORKSPACES: Range<u16> = 1..30;
static ALL_TAGS: Lazy<Vec<String>> = Lazy::new(|| WORKSPACES.map(|ix| ix.to_string()).collect());

fn workspace_menu() -> Box<dyn KeyEventHandler<RustConn>> {
    key_handler(|state, _xcon| {
        let sc_ix = state.client_set.current_screen().index();
        // TODO: also replace dmenu program selector with dmenu-rs
        let dmenu = DMenu::new(
            &DMenuConfig {
                kind: DMenuKind::Suckless, // Rust doesn't seem to work yet
                custom_prompt: Some("workspace> ".to_string()),
                ..Default::default()
            },
            sc_ix,
        );
        if let Ok(MenuMatch::Line(_, choice)) = dmenu.build_menu(ALL_TAGS.clone()) {
            Ok(state.client_set.focus_tag(choice))
        } else {
            Ok(())
        }
    })
}

fn raw_key_bindings() -> HashMap<String, Box<dyn KeyEventHandler<RustConn>>> {
    let action_bindings = map! {
        map_keys: |k: &str| k.to_string();
        "M-g" => workspace_menu(),
        "M-n" => modify_with(|cs| cs.focus_down()),
        "M-a" => modify_with(|cs| cs.focus_up()),
        "M-S-n" => modify_with(|cs| cs.swap_down()),
        "M-S-a" => modify_with(|cs| cs.swap_up()),
        "M-S-c" => modify_with(|cs| cs.kill_focused()),
        "M-Tab" => modify_with(|cs| cs.toggle_tag()),
        "M-m" => modify_with(|cs| cs.next_screen()),
        "M-i" => modify_with(|cs| cs.previous_screen()),
        "M-space" => modify_with(|cs| cs.next_layout()),
        "M-S-space" => modify_with(|cs| cs.previous_layout()),
        "M-S-Up" => send_layout_message(|| IncMain(1)),
        "M-S-Down" => send_layout_message(|| IncMain(-1)),
        "M-S-Right" => send_layout_message(|| ExpandMain),
        "M-S-Left" => send_layout_message(|| ShrinkMain),
        "M-Return" => send_layout_message(|| Rotate),
        "M-p" => spawn("dmenu_run"),
        // "M-p" => spawn("yeganesh -x"), // not working for some reason
        "M-S-Return" => spawn("alacritty"),
        "M-A-Escape" => exit(),
    };

    (["1", "2", "3", "4", "5", "6", "7", "8", "9"])
        .into_iter()
        .flat_map(|tag| {
            [
                (
                    format!("M-{tag}"),
                    modify_with(move |client_set| client_set.focus_tag(tag)),
                ),
                (
                    format!("M-S-{tag}"),
                    modify_with(move |client_set| client_set.move_focused_to_tag(tag)),
                ),
            ]
        })
        // TODO: in the future, maybe enter a mode so that we can create an arbitrary
        // number of desktops - kind of like launching dmenu to pick an app, but
        // instead launch a workspace picker
        .chain(
            (["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"])
                .into_iter()
                .flat_map(|tag_base| {
                    let tag1 = format!("1{tag_base}");
                    let tag2 = tag1.clone();
                    [
                        (
                            format!("M-C-{tag_base}"),
                            modify_with(move |client_set| client_set.focus_tag(tag1.as_str())),
                        ),
                        (
                            format!("M-C-S-{tag_base}"),
                            modify_with(move |client_set| {
                                println!("DEBUG: move to {tag2}");
                                client_set.move_focused_to_tag(tag2.as_str())
                            }),
                        ),
                    ]
                }),
        )
        .chain(action_bindings)
        .collect::<HashMap<String, Box<dyn KeyEventHandler<RustConn>>>>()
}

fn layout() -> LayoutStack {
    let stack = MainAndStack::side(1, 0.5, 0.1);
    stack!(stack).map(|layout| ReserveTop::wrap(layout, BAR_HEIGHT_PX_PRIMARY))
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .finish()
        .init();

    let conn = RustConn::new()?;
    let key_bindings = parse_keybindings_with_xmodmap(raw_key_bindings())?;
    let config = add_ewmh_hooks(Config {
        default_layouts: layout(),
        tags: ALL_TAGS.clone(),
        // startup_hook: Some(SpawnOnStartup::boxed("polybar")),
        ..Default::default()
    });

    let bar = status_bar().unwrap(); // FIXME: properaly convert error
    let wm = bar.add_to(WindowManager::new(
        config,
        key_bindings,
        HashMap::new(),
        conn,
    )?);
    wm.run()
}
