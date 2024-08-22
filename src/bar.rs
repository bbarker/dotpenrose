use crate::{
    log::*,
    workspaces::{workspace_app_info, TagAndAppInfo, SYSTEM},
    BLACK, BLUE, FONT, GREY, WHITE,
};
use do_notation::m;
use penrose::{
    core::State,
    pure::geometry::{Point, Rect},
    x::XConn,
    Color,
};
use penrose_ui::{
    bar::{
        widgets::{
            sys::{
                helpers::battery_file_search,
                interval::{amixer_volume, battery_summary, current_date_and_time, wifi_network},
            },
            ActiveWindowName, CurrentLayout, FocusState, Widget, WorkspacesUi, WorkspacesWidget,
            WsMeta,
        },
        PerScreen, Position, StatusBar,
    },
    Context, Result, TextStyle,
};
use std::time::Duration;

use once_cell::sync::Lazy;

pub static BATTERY: Lazy<String> =
    Lazy::new(|| battery_file_search().unwrap_or("BAT1".to_string()));
// TODO: Work out how to change the highlight color to RED when we pass a certain
//   date '+%H%M' might be a quick and dirty way to get the current hour/minute value for
//   comparison against a switch time for changing out the color of theactive window widget at
//   least?

pub const MAX_ACTIVE_WINDOW_CHARS: usize = 50;
pub const BAR_HEIGHT_PX_PRIMARY: u32 = 24;
pub const BAR_HEIGHT_PX_EXTERNAL: u32 = 18;
pub const BAR_POINT_SIZE_PRIMARY: u8 = 12;
pub const BAR_POINT_SIZE_EXTERNAL: u8 = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
struct AppInfo {
    titles: Vec<String>,
    processes: Vec<String>,
}

// TODO: have this be a bit more customizable by
//     : taking some kind of mapping or config struct
impl AppInfo {
    fn iconic_tag(&self, tag: String) -> String {
        log_penrose(&format!("Calling iconic_tag on tag {tag}")).unwrap();
        let icon_list = vec![
            if self.processes.iter().any(|pname| pname.contains("spotify")) {
                "🎵"
            } else {
                ""
            },
        ];
        let icons: String = icon_list.concat();
        log_penrose(&format!("icons for {tag}: {:?}", icons)).unwrap();

        if icons.is_empty() {
            tag
        } else {
            format!("[{tag}{icons}]")
        }
    }
}

impl From<TagAndAppInfo> for AppInfo {
    fn from(tag_app_info: TagAndAppInfo) -> Self {
        Self {
            titles: tag_app_info.titles,
            processes: tag_app_info.processes,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MyWorkspaceUi {
    fg_1: Color,
    fg_2: Color,
    bg_1: Color,
    bg_2: Color,
    ws_apps: Vec<AppInfo>,
}

impl MyWorkspaceUi {
    fn new(style: TextStyle, highlight: impl Into<Color>, empty_fg: impl Into<Color>) -> Self {
        Self {
            fg_1: style.fg,
            fg_2: empty_fg.into(),
            bg_1: highlight.into(),
            bg_2: style.bg.unwrap_or_else(|| 0x000000.into()),
            ws_apps: Vec::new(),
        }
    }
}

impl WorkspacesUi for MyWorkspaceUi {
    fn update_from_state<X>(
        &mut self,
        workspace_meta: &[WsMeta],
        focused_tags: &[String],
        state: &State<X>,
        xcon: &X,
    ) -> bool
    where
        X: XConn,
    {
        let new_ws_apps = state
            .client_set
            .workspaces()
            .map(|ws| workspace_app_info(&SYSTEM, state, xcon, ws).into())
            .collect();
        if self.ws_apps == new_ws_apps {
            false
        } else {
            // DEBUG: remove log line
            log_penrose(&format!("updating workspace-ui state: {:?}", new_ws_apps)).unwrap();
            self.ws_apps = new_ws_apps;
            true
        }
    }

    fn background_color(&self) -> Color {
        self.bg_2
    }

    fn colors_for_workspace(
        &self,
        ws_meta: &WsMeta,
        focus_state: FocusState,
        screen_has_focus: bool,
    ) -> (Color, Color) {
        use FocusState::*;

        match focus_state {
            FocusedOnThisScreen if screen_has_focus && ws_meta.occupied() => (self.fg_1, self.bg_1),
            FocusedOnThisScreen if screen_has_focus => (self.fg_2, self.bg_1),
            FocusedOnThisScreen => (self.fg_1, self.fg_2),
            FocusedOnOtherScreen => (self.bg_1, self.fg_2),
            Unfocused if ws_meta.occupied() => (self.fg_1, self.bg_2),
            Unfocused => (self.fg_2, self.bg_2),
        }
    }

    fn ui_tag(&self, workspace_meta: &WsMeta) -> String {
        match workspace_meta.occupied() {
            true => {
                let tag_string = workspace_meta.tag().to_string();
                match m! {
                    tag_num <- tag_string.parse::<usize>()
                      .log_err(&format!("couldn't parse int from {tag_string}"));
                    ws_ix <- tag_num.checked_sub(1)
                      .log_err(&format!("In ui_tag: couldn't subtract 1 from {tag_num}"));
                    self.ws_apps.get(ws_ix)
                } {
                    Some(app_info) => app_info.iconic_tag(tag_string),
                    None => tag_string,
                }
            }
            false => String::new(),
        }
    }
}

type MyWorkspaces = WorkspacesWidget<MyWorkspaceUi>;

fn new_workspaces(
    style: TextStyle,
    highlight: impl Into<Color>,
    empty_fg: impl Into<Color>,
) -> MyWorkspaces {
    let ui = MyWorkspaceUi::new(style, highlight, empty_fg);

    WorkspacesWidget::new_with_ui(ui)
}

fn base_widgets<X: XConn>() -> Vec<Box<dyn Widget<X>>> {
    let highlight: Color = BLUE.into();
    let empty_ws: Color = GREY.into();
    let style = TextStyle {
        fg: WHITE.into(),
        bg: Some(BLACK.into()),
        padding: (2, 2),
    };

    let pstyle = TextStyle {
        padding: (5, 5),
        ..style
    };

    let ms = |n: u64| Duration::from_millis(n);

    vec![
        Box::new(Wedge::start(BLUE, BLACK)),
        Box::new(new_workspaces(style, highlight, empty_ws)),
        Box::new(CurrentLayout::new(style)),
        Box::new(Wedge::end(BLUE, BLACK).only_with_focus()),
        Box::new(ActiveWindowName::new(
            MAX_ACTIVE_WINDOW_CHARS,
            TextStyle {
                bg: Some(highlight),
                padding: (6, 4),
                ..style
            },
            true,
            false,
        )),
        Box::new(Wedge::start(BLUE, BLACK).only_with_focus()),
        // The wttr.in API is freaking out a bit recently and hanging / returning errors
        // so dropping this for now.
        // Box::new(IntervalText::new(pstyle, weather_text, ms(300_000))),
        Box::new(wifi_network(pstyle, ms(10_000))),
        Box::new(battery_summary(&BATTERY, pstyle, ms(60_000))),
        Box::new(amixer_volume("Master", pstyle, ms(1000))),
        Box::new(current_date_and_time(pstyle, ms(10_000))),
    ]
}

pub fn status_bar<X: XConn>() -> Result<StatusBar<X>> {
    let mut primary = base_widgets();
    primary.push(Box::new(Spacer::new(0.07))); // reserve space for trayer
    let external = base_widgets();

    let bar = StatusBar::try_new_per_screen(
        Position::Top,
        BLACK,
        FONT,
        vec![
            PerScreen::new(BAR_POINT_SIZE_PRIMARY, BAR_HEIGHT_PX_PRIMARY, primary),
            PerScreen::new(BAR_POINT_SIZE_EXTERNAL, BAR_HEIGHT_PX_EXTERNAL, external),
        ],
    )?;

    Ok(bar)
}

// pub fn weather_text() -> Option<String> {
//     Some(
//         spawn_for_output_with_args(
//             "curl",
//             &["-s", "--max-time", "5", "http://wttr.in?format=%c%t"],
//         )
//         .unwrap_or_default()
//         .trim()
//         .to_string(),
//     )
// }

/// A simple 45 degree wedge
#[derive(Debug, Clone, Copy)]
pub struct Wedge {
    only_with_focus: bool,
    start: bool,
    fg: Color,
    bg: Color,
}

impl Wedge {
    fn new(fg: impl Into<Color>, bg: impl Into<Color>, start: bool) -> Self {
        Self {
            only_with_focus: false,
            start,
            fg: fg.into(),
            bg: bg.into(),
        }
    }

    fn start(fg: impl Into<Color>, bg: impl Into<Color>) -> Self {
        Self::new(fg, bg, true)
    }

    fn end(fg: impl Into<Color>, bg: impl Into<Color>) -> Self {
        Self::new(fg, bg, false)
    }

    fn only_with_focus(mut self) -> Self {
        self.only_with_focus = true;
        self
    }
}

impl<X: XConn> Widget<X> for Wedge {
    fn draw(&mut self, ctx: &mut Context<'_>, _: usize, f: bool, w: u32, h: u32) -> Result<()> {
        ctx.fill_rect(Rect::new(0, 0, w, h), self.bg)?;
        if self.only_with_focus && !f {
            return Ok(());
        }

        let p = if self.start { 0 } else { h };
        ctx.fill_polygon(
            &[Point::new(p, p), Point::new(h, 0), Point::new(0, h)],
            self.fg,
        )
    }

    fn current_extent(&mut self, _: &mut Context<'_>, h: u32) -> Result<(u32, u32)> {
        Ok((h, h))
    }

    fn is_greedy(&self) -> bool {
        false
    }

    fn require_draw(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct Spacer {
    perc: f32,
    w: u32,
}

impl Spacer {
    pub fn new(perc: f32) -> Self {
        if !(0.0..=1.0).contains(&perc) {
            panic!("{perc} is an invalid percentage");
        }

        Self { perc, w: 0 }
    }
}

impl<X: XConn> Widget<X> for Spacer {
    fn draw(&mut self, ctx: &mut Context<'_>, _: usize, _: bool, w: u32, h: u32) -> Result<()> {
        ctx.fill_bg(Rect::new(0, 0, w, h))
    }

    fn current_extent(&mut self, _: &mut Context<'_>, h: u32) -> Result<(u32, u32)> {
        Ok((self.w, h))
    }

    fn is_greedy(&self) -> bool {
        false
    }

    fn require_draw(&self) -> bool {
        false
    }

    fn on_startup(&mut self, state: &mut State<X>, _: &X) -> Result<()> {
        self.w = state
            .client_set
            .screens()
            .next()
            .map(|s| (s.geometry().w as f32 * self.perc) as u32)
            .unwrap();

        Ok(())
    }
}
