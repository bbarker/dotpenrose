use once_cell::sync::Lazy;
use penrose::builtin::actions::key_handler;
use penrose::core::State;
use penrose::extensions::util::dmenu::{DMenu, DMenuConfig, DMenuKind, MenuMatch};
use penrose::x11rb::RustConn;

use std::env;
use std::process::Command;

use crate::workspaces::workspace_app_info;
use crate::{KeyHandler, SYSTEM};

#[derive(Clone, Debug, Default)]
pub struct GotoWorkspaceConfig<'a> {
    name_substitutions: Vec<(&'a str, &'a str)>,
    title_substitutions: Vec<(&'a str, &'a str)>,
}

pub fn get_hostname() -> String {
    Command::new("hostname")
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|_| "Unknown".to_string())
}

// Could possibly use alternatives from the nix crate
pub static HOSTNAME: Lazy<String> =
    Lazy::new(|| env::var("HOSTNAME").unwrap_or_else(|_| get_hostname()));
pub static USERNAME: Lazy<String> =
    Lazy::new(|| env::var("USER").unwrap_or_else(|_| "Unknown".to_string()));
pub static NU_SHELL_LOC: Lazy<String> = Lazy::new(|| format!("{}@{}:", *USERNAME, *HOSTNAME));

/// My Config; I've left it here as an example. Though you could use it, you may
/// want to customize it or, most likely, just write your own.
pub static GOTO_WS_CONFIG: Lazy<GotoWorkspaceConfig> = Lazy::new(|| GotoWorkspaceConfig {
    name_substitutions: vec![("-wrapped", ""), (".", "")],
    title_substitutions: vec![(&NU_SHELL_LOC, "local")],
});

/// Navigate to a workspace by typing part of a process name
/// or window title running on the workspace.
pub fn goto_workspace_by_apps(
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
                    ignore_case: true,
                    ..Default::default()
                },
                sc_ix,
            );
            let tags_display_strings = {
                let workspaces = state.client_set.workspaces();
                let mut unsorted_tds: Vec<(String, String)> = workspaces
                    .map(|ws| {
                        let ws_app_info = workspace_app_info(&SYSTEM, state, xcon, ws);
                        let window_titles = ws_app_info
                            .titles
                            .into_iter()
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
                        let app_names = ws_app_info.processes.into_iter().map(|exe_name| {
                            conf_local
                                .name_substitutions
                                .iter()
                                .fold(exe_name, |en, (rep, sub)| en.replace(rep, sub))
                                .trim()
                                .to_owned()
                        });
                        let display_string = {
                            let display_strings = app_names
                                .into_iter()
                                .zip(window_titles)
                                .map(|(app, title)| format!("{app} ➥ {title}"))
                                .collect::<Vec<String>>();
                            display_strings.join(" | ")
                        };
                        (ws_app_info.tag, display_string)
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

/// Got a lot of workspaces? This function, and its sister function,
/// `send_to_workspace_menu`, can help you navigate to them using
/// a dmenu.
pub fn workspace_menu() -> KeyHandler {
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
        let all_tags = state.client_set.ordered_tags();
        if let Ok(MenuMatch::Line(_, choice)) = dmenu.build_menu(all_tags) {
            Ok(state.client_set.focus_tag(choice))
        } else {
            Ok(())
        }
    })
}

/// Got a lot of workspaces? This function, and its sister function,
/// `workspace_menu`, can help you navigate to them using
/// a dmenu.
pub fn send_to_workspace_menu() -> KeyHandler {
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
        let all_tags = state.client_set.ordered_tags();
        if let Ok(MenuMatch::Line(_, choice)) = dmenu.build_menu(all_tags) {
            Ok(state.client_set.move_focused_to_tag(choice))
        } else {
            Ok(())
        }
    })
}
