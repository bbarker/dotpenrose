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
use std::env;
use std::ops::RangeInclusive;
use tracing_subscriber::util::SubscriberInitExt;

use std::process::Command;
use sysinfo::{Pid, System};

use dotpenrose::bar::{status_bar, BAR_HEIGHT_PX_PRIMARY};

// Could possibly use alternatives from the nix crate
static HOSTNAME: Lazy<String> =
    Lazy::new(|| env::var("HOSTNAME").unwrap_or_else(|_| get_hostname()));
static USERNAME: Lazy<String> =
    Lazy::new(|| env::var("USER").unwrap_or_else(|_| "Unknown".to_string()));
static NU_SHELL_LOC: Lazy<String> = Lazy::new(|| format!("{}@{}:", *USERNAME, *HOSTNAME));

fn get_hostname() -> String {
    Command::new("hostname")
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|_| "Unknown".to_string())
}

// Let's start with 29 tags
const NUM_FAST_ACCESS_WORKSPACES: u16 = 9;
const WORKSPACES: RangeInclusive<u16> = 1..=(NUM_FAST_ACCESS_WORKSPACES + 20);
static ALL_TAGS: Lazy<Vec<String>> = Lazy::new(|| WORKSPACES.map(|ix| ix.to_string()).collect());
static SYSTEM: Lazy<System> = Lazy::new(System::new_all);

#[derive(Clone, Debug, Default)]
pub struct GotoWorkspaceConfig<'a> {
    name_substitutions: Vec<(&'a str, &'a str)>,
    title_substitutions: Vec<(&'a str, &'a str)>,
}

static GOTO_WS_CONFIG: Lazy<GotoWorkspaceConfig> = Lazy::new(|| GotoWorkspaceConfig {
    name_substitutions: vec![("-wrapped", ""), (".", "")],
    title_substitutions: vec![(&NU_SHELL_LOC, "local")],
});

type KeyHandler = Box<dyn KeyEventHandler<RustConn>>;

static GOTO_WS: Lazy<Box<dyn Fn() -> KeyHandler + Send + Sync>> =
    Lazy::new(|| goto_workspace_by_apps(&GOTO_WS_CONFIG));

fn workspace_menu() -> KeyHandler {
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

fn send_to_workspace_menu() -> KeyHandler {
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

fn goto_workspace_by_apps(
    conf: &'static GotoWorkspaceConfig<'static>,
) -> Box<dyn Fn() -> KeyHandler + Send + Sync> {
    fn extract_tag(str: &str) -> Option<&str> {
        let parts: Vec<_> = str.splitn(2, ':').collect();
        if parts.len() == 2 {
            parts.first().map(|part| part.trim())
        } else {
            None
        }
    }

    Box::new(|| {
        key_handler(|state: &mut State<RustConn>, xcon: &RustConn| {
            let conf_local = conf.clone();
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
                            .map(|title| {
                                conf_local
                                    .title_substitutions
                                    .iter()
                                    .fold(title, |new_title, (rep, sub)| {
                                        new_title.replace(rep, sub)
                                    })
                                    .trim()
                                    .to_owned()
                            })
                            .collect::<Vec<String>>();
                        let app_names = ws
                            .clients()
                            .map(|client| xcon.get_prop(*client, "_NET_WM_PID"))
                            .map(|prop_res| match prop_res {
                                Ok(Some(penrose::x::Prop::Cardinal(cardinals))) => cardinals
                                    .into_iter()
                                    .map(|pid| {
                                        if let Some(process) =
                                            SYSTEM.process(Pid::from(pid as usize))
                                        {
                                            let exe_name = process
                                                .exe()
                                                .and_then(|exe_path| {
                                                    std::path::Path::new(exe_path).file_name()
                                                })
                                                .map_or_else(
                                                    || "Unknown".to_string(),
                                                    |os_str| os_str.to_string_lossy().into_owned(),
                                                );
                                            conf_local
                                                .name_substitutions
                                                .iter()
                                                .fold(exe_name, |en, (rep, sub)| {
                                                    en.replace(rep, sub)
                                                })
                                                .trim()
                                                .to_owned()
                                        } else {
                                            String::new()
                                        }
                                    })
                                    .collect::<Vec<String>>()
                                    .join(","),
                                _ => String::new(),
                            })
                            .collect::<Vec<String>>();
                        let display_string = {
                            let display_strings = app_names
                                .into_iter()
                                .zip(window_titles)
                                .map(|(app, title)| format!("{app} ➥ {title}"))
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
    })
}

fn raw_key_bindings() -> HashMap<String, Box<dyn KeyEventHandler<RustConn>>> {
    let action_bindings = map! {
        map_keys: |k: &str| k.to_string();
        "M-f" => GOTO_WS(),
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
