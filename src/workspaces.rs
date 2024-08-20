use once_cell::sync::Lazy;
use penrose::{
    core::State,
    pure::Workspace,
    x::{XConn, XConnExt},
    Xid,
};
use sysinfo::{Pid, System};

pub static SYSTEM: Lazy<System> = Lazy::new(System::new_all);

pub struct TagAndAppInfo {
    pub tag: String,
    pub titles: Vec<String>,
    pub processes: Vec<String>,
}

pub fn workspace_app_info<X>(
    system: &sysinfo::System,
    state: &State<X>,
    xcon: &X,
    ws: &Workspace<Xid>,
) -> TagAndAppInfo
where
    X: XConn,
{
    let tag = state
        .client_set
        .tag_for_workspace_id(ws.id())
        .unwrap_or_default();
    let titles = ws
        .clients()
        .map(|client| xcon.window_title(*client).unwrap_or_default())
        .collect::<Vec<String>>();
    let processes = ws
        .clients()
        .map(|client| xcon.get_prop(*client, "_NET_WM_PID"))
        .map(|prop_res| match prop_res {
            Ok(Some(penrose::x::Prop::Cardinal(cardinals))) => cardinals
                .into_iter()
                .map(|pid| {
                    if let Some(process) = system.process(Pid::from(pid as usize)) {
                        process
                            .exe()
                            .and_then(|exe_path| std::path::Path::new(exe_path).file_name())
                            .map_or_else(
                                || "Unknown".to_string(),
                                |os_str| os_str.to_string_lossy().into_owned(),
                            )
                    } else {
                        String::new()
                    }
                })
                .collect::<Vec<String>>()
                .join(","),
            _ => String::new(),
        })
        .collect::<Vec<String>>();
    TagAndAppInfo {
        tag,
        titles,
        processes,
    }
}
