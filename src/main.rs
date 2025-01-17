#![allow(clippy::unit_arg)]

use nunny::NonEmpty;
use once_cell::sync::Lazy;
use penrose::{
    builtin::{
        actions::{exit, modify_with, send_layout_message, spawn},
        layout::{
            messages::{ExpandMain, IncMain, ShrinkMain},
            transformers::ReserveTop,
            MainAndStack,
        },
    },
    core::{
        bindings::{parse_keybindings_with_xmodmap, KeyEventHandler},
        hooks::StateHook,
        layout::LayoutStack,
        Config, WindowManager,
    },
    extensions::hooks::{add_ewmh_hooks, SpawnOnStartup},
    map, stack,
    x11rb::RustConn,
    Result,
};
use penrose_bbarker_contrib::{
    is_in_path, is_running,
    log::log_penrose,
    menus::finder::{
        goto_workspace_by_apps, send_to_workspace_menu, workspace_menu, GOTO_WS_CONFIG,
    },
    KeyHandler,
};

use std::collections::HashMap;
use tracing_subscriber::util::SubscriberInitExt;

use dotpenrose::{
    bar::{status_bar, BAR_HEIGHT_PX_PRIMARY},
    ALL_TAGS, NUM_FAST_ACCESS_WORKSPACES,
};

static GOTO_WS: Lazy<Box<dyn Fn() -> KeyHandler + Send + Sync>> =
    Lazy::new(|| goto_workspace_by_apps(&GOTO_WS_CONFIG));

fn raw_key_bindings() -> HashMap<String, Box<dyn KeyEventHandler<RustConn>>> {
    let action_bindings = map! {
        map_keys: |k: &str| k.to_string();
        "M-f" => GOTO_WS(),
        "M-g" => workspace_menu(),
        "M-S-g" => send_to_workspace_menu(),
        "M-Left" => modify_with(|cs| cs.focus_previous_workspace()),
        "M-Right" => modify_with(|cs| cs.focus_next_workspace()),
        "M-n" => modify_with(|cs| cs.focus_down()),
        "M-a" => modify_with(|cs| cs.focus_up()),
        "M-S-n" => modify_with(|cs| cs.swap_down()),
        "M-S-a" => modify_with(|cs| cs.swap_up()),
        "M-S-c" => modify_with(|cs| cs.kill_focused()),
        "M-Tab" => modify_with(|cs| cs.toggle_tag()),
        "M-m" => modify_with(|cs| cs.next_screen()),
        "M-i" => modify_with(|cs| cs.previous_screen()),
        "M-s" => modify_with(|cs| cs.drag_workspace_forward()),
        "M-S-s" => modify_with(|cs| cs.drag_workspace_backward()),
        "M-space" => modify_with(|cs| cs.next_layout()),
        "M-S-space" => modify_with(|cs| cs.previous_layout()),
        "M-S-Up" => send_layout_message(|| IncMain(1)),
        "M-S-Down" => send_layout_message(|| IncMain(-1)),
        "M-l" => send_layout_message(|| ExpandMain),
        "M-h" => send_layout_message(|| ShrinkMain),
        "M-Return" => modify_with(|cs| cs.swap_focus_and_head()),
        "M-p" => spawn("dmenu_run"),
        // "M-p" => spawn("yeganesh -x"), // not working for some reason
        "M-S-z" => spawn("xscreensaver-command -lock"),
        "M-S-Return" => spawn("alacritty"),
        "M-A-Escape" => exit(),
    };
    (1..=NUM_FAST_ACCESS_WORKSPACES)
        .map(|ws| ws.to_string())
        .flat_map(|tag| {
            let tag_copy = tag.clone();
            [
                (
                    format!("M-{tag_copy}"),
                    modify_with(move |client_set| client_set.focus_tag(tag_copy.clone())),
                ),
                (
                    format!("M-S-{}", tag.clone()),
                    modify_with(move |client_set| client_set.move_focused_to_tag(tag.clone())),
                ),
            ]
        })
        .chain(action_bindings)
        .collect::<HashMap<String, Box<dyn KeyEventHandler<RustConn>>>>()
}

fn layout() -> LayoutStack {
    let stack = MainAndStack::side(1, 0.5, 0.1);
    stack!(stack).map(|layout| ReserveTop::wrap(layout, BAR_HEIGHT_PX_PRIMARY))
}

fn main() -> Result<()> {
    log_penrose("DEBUG: entered main")?;
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .finish()
        .init();

    log_penrose("DEBUG: creeated tracing_subscriber")?;
    let startup_progs: NonEmpty<[(&str, &str); 2]> = nunny::array![
        ("xscreensaver", ""),
        ("nvidia-settings", "--load-config-only"),
    ];

    let progs_to_start: Vec<(&str, &str)> = startup_progs
        .into_iter()
        .filter(|(prog, _)| is_in_path(prog) && !is_running(prog))
        .collect();
    log_penrose("DEBUG: creeated progs_to_start")?;
    let startup_hook = progs_to_start
        .into_iter()
        .map(
            |(prog, args)| SpawnOnStartup::boxed(format!("{prog} {args}").trim().to_owned()), // .reduce(|ha, hi| ha.then(hi))
        )
        .collect::<Vec<_>>();

    log_penrose("DEBUG: creeated startup_hook")?;
    let conn = RustConn::new()?;
    let key_bindings = parse_keybindings_with_xmodmap(raw_key_bindings())?;
    let config = add_ewmh_hooks(Config {
        default_layouts: layout(),
        tags: ALL_TAGS.clone(),
        startup_hook: Some(StateHook::boxed(startup_hook)),
        ..Default::default()
    });

    log_penrose("DEBUG: defined config")?;
    let bar = status_bar().unwrap(); // FIXME: properaly convert error
    log_penrose("DEBUG: created status bar")?;
    let wm = bar.add_to(WindowManager::new(
        config,
        key_bindings,
        HashMap::new(),
        conn,
    )?);
    log_penrose("DEBUG: created wm")?;
    // FIXME: debugging bar on the desktop
    // let wm = WindowManager::new(config, key_bindings, HashMap::new(), conn)?;
    wm.run()
}
