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
        Config, State, WindowManager,
    },
    extensions::{
        hooks::{add_ewmh_hooks, SpawnOnStartup},
        util::dmenu::{DMenu, DMenuConfig, DMenuKind, MenuMatch},
    },
    map, stack,
    x::{XConn, XConnExt},
    x11rb::RustConn,
    Result,
};
use std::collections::HashMap;
use std::ops::RangeInclusive;
use tracing_subscriber::util::SubscriberInitExt;

use sysinfo::{Pid, System};

use dotpenrose::{
    bar::{status_bar, BAR_HEIGHT_PX_PRIMARY},
    log::log_penrose,
};

// Let's start with 29 tags
const NUM_FAST_ACCESS_WORKSPACES: u16 = 9;
const WORKSPACES: RangeInclusive<u16> = 1..=(NUM_FAST_ACCESS_WORKSPACES + 20);
static ALL_TAGS: Lazy<Vec<String>> = Lazy::new(|| WORKSPACES.map(|ix| ix.to_string()).collect());
static SYSTEM: Lazy<System> = Lazy::new(System::new_all);

fn workspace_menu() -> Box<dyn KeyEventHandler<RustConn>> {
    key_handler(|state, _xcon| {
        let sc_ix = state.client_set.current_screen().index();
        let dmenu = DMenu::new(
            &DMenuConfig {
                kind: DMenuKind::Rust,
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

fn send_to_workspace_menu() -> Box<dyn KeyEventHandler<RustConn>> {
    key_handler(|state, _xcon| {
        let sc_ix = state.client_set.current_screen().index();
        let dmenu = DMenu::new(
            &DMenuConfig {
                kind: DMenuKind::Rust,
                show_on_bottom: true,
                custom_prompt: Some("send to> ".to_string()),
                ..Default::default()
            },
            sc_ix,
        );
        if let Ok(MenuMatch::Line(_, choice)) = dmenu.build_menu(ALL_TAGS.clone()) {
            Ok(state.client_set.move_focused_to_tag(choice))
        } else {
            Ok(())
        }
    })
}

// TODO: filter out empty workspaces
// TODO: sort workspaces -> use func-iter, but fix mutability and add test
//     : in func-iter
fn goto_workspace_by_apps() -> Box<dyn KeyEventHandler<RustConn>> {
    fn extract_tag(str: &str) -> Option<&str> {
        let parts: Vec<_> = str.splitn(2, ':').collect();
        if parts.len() == 2 {
            parts.first().map(|part| part.trim())
        } else {
            None
        }
    }

    key_handler(|state: &mut State<RustConn>, xcon: &RustConn| {
        let sc_ix = state.client_set.current_screen().index();
        let dmenu = DMenu::new(
            &DMenuConfig {
                kind: DMenuKind::Rust,
                custom_prompt: Some("workspace> ".to_string()),
                ..Default::default()
            },
            sc_ix,
        );
        let tags_display_strings = {
            let mut unsorted_tds: Vec<(String, String)> = state
                .client_set
                .workspaces()
                .map(|ws| {
                    let tag = state
                        .client_set
                        .tag_for_workspace_id(ws.id())
                        .unwrap_or_default();
                    let window_titles = ws
                        .clients()
                        .map(|client| xcon.window_title(*client).unwrap_or_default())
                        .map(|title| title[..20].to_string())
                        .collect::<Vec<String>>();
                    let app_names = ws
                        .clients()
                        .map(|client| xcon.get_prop(*client, "_NET_WM_PID"))
                        .map(|prop_res| match prop_res {
                            Ok(Some(penrose::x::Prop::Cardinal(cardinals))) => cardinals
                                .into_iter()
                                .map(|pid| {
                                    if let Some(process) = SYSTEM.process(Pid::from(pid as usize)) {
                                        process.name().to_string()
                                    } else {
                                        String::new()
                                    }
                                })
                                .collect::<Vec<String>>()
                                .join(","),
                            _ => String::new(),
                        })
                        .collect::<Vec<String>>();
                    log_penrose(&format!("{:?}", app_names)).unwrap_or_default();
                    let display_string = {
                        let display_strings = app_names
                            .into_iter()
                            .zip(window_titles)
                            .map(|(app, title)| format!("{app} > {title}"))
                            .collect::<Vec<String>>();
                        display_strings.join(" | ")
                    };
                    (tag, display_string)
                })
                .filter(|(_, display)| !display.is_empty())
                .collect();
            unsorted_tds.sort_by_key(|(tag, _dsp)| tag.parse::<u16>().unwrap_or(999));
            unsorted_tds // now sorted
        };

        let entries = tags_display_strings
            .into_iter()
            .map(|(tag, display_string)| format!("{}: {}", tag, display_string))
            .collect();
        if let Ok(MenuMatch::Line(_, choice)) = dmenu.build_menu(entries) {
            extract_tag(&choice)
                .ok_or(penrose::Error::Custom("No tag for workspace".to_string()))
                .map(|tag| state.client_set.focus_tag(tag))
        } else {
            Ok(())
        }
    })
}

fn raw_key_bindings() -> HashMap<String, Box<dyn KeyEventHandler<RustConn>>> {
    let action_bindings = map! {
        map_keys: |k: &str| k.to_string();
        "M-f" => goto_workspace_by_apps(),
        "M-g" => workspace_menu(),
        "M-S-g" => send_to_workspace_menu(),
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
        "M-l" => send_layout_message(|| ExpandMain),
        "M-h" => send_layout_message(|| ShrinkMain),
        "M-Return" => send_layout_message(|| Rotate),
        "M-p" => spawn("dmenu_run"),
        // "M-p" => spawn("yeganesh -x"), // not working for some reason
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
