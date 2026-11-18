//! The selected instance's replays on Home (`horizon_launcher::replays`): the takes Rezyth
//! Settings' replay recorder saved. A list of takes, and a viewer: the camera's path from above,
//! a scrubber along the take, play/pause and speeds, and the pose at the playhead.
use super::*;
use horizon_launcher::replays::{self, Track};
use std::path::PathBuf;
use std::rc::Rc;
use std::cell::Cell;

/// The playback speeds, cycled by the speed button.
const SPEEDS: [f64; 4] = [0.5, 1.0, 2.0, 4.0];
/// The path plot's dots, at most (a take samples every world tick).
const DOTS: usize = 120;

#[derive(Default)]
pub(super) struct Replays {
    home: Option<PathBuf>,
    pub(super) list: Vec<replays::Replay>,
    fresh: bool,
    /// The take open in the viewer, by its place in `list`.
    pub(super) viewing: Option<usize>,
    /// The open take's track, parsed.
    loaded: Option<Track>,
    /// The playhead, in the take's world ticks.
    playhead: f64,
    playing: bool,
    /// The playback speed, by its place in [`SPEEDS`].
    speed: usize,
    /// The scrubber's bar on screen, for a click's fraction of it.
    bar: Rc<Cell<Option<Bounds<Pixels>>>>,
    dragging: bool,
    /// When the last playback step ran, for the ticks it advances by.
    last_step: Option<std::time::Instant>,
}

impl Replays {
    pub(super) fn stale(&mut self) {
        self.fresh = false;
    }
}

impl Launcher {
    pub(super) fn load_replays(&mut self, home: PathBuf) {
        if self.replays.home.as_ref() != Some(&home) {
            self.replays = Replays {
                home: Some(home.clone()),
                ..Replays::default()
            };
        }
        if !self.replays.fresh {
            self.replays.list = replays::list(&home);
            self.replays.fresh = true;
            let last = self.replays.list.len().checked_sub(1);
            self.replays.viewing = self.replays.viewing.and_then(|i| last.map(|l| i.min(l)));
        }
    }

    /// How many replays the selected instance has.
    pub(super) fn replays_count(&mut self) -> usize {
        let Some(home) = self.selected_instance().map(|i| i.home.clone()) else {
            return 0;
        };
        self.load_replays(home);
        self.replays.list.len()
    }

    pub(super) fn replays_escape(&mut self) -> bool {
        let open = self.replays.viewing.take().is_some();
        self.replays.playing = false;
        open
    }

    /// Home's Replays tab: its buttons and its takes; None while it has none.
    pub(super) fn replays_section(&mut self, cx: &mut Context<Self>) -> Option<(Div, Div)> {
        let t = self.theme;
        let home = self.selected_instance()?.home.clone();
        self.load_replays(home.clone());
        let list = self.replays.list.clone();
        if list.is_empty() {
            return None;
        }
        let folder = replays::folder(&home);
        let buttons = div()
            .flex()
            .items_center()
            .gap(px(8.))
            .child(
                tab_action("replays-folder", "Open folder", &t)
                    .on_click(move |_, _, cx| cx.open_with_system(&folder)),
            );
        let rows = div().flex().flex_col().gap(px(6.)).children(
            list.into_iter().enumerate().map(|(i, take)| {
                let secs = take.length_ticks as f64 / 20.0;
                div()
                    .id(SharedString::from(format!("replay-{i}")))
                    .flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .px(px(12.))
                    .py(px(8.))
                    .border_2()
                    .border_color(t.outline)
                    .bg(t.well)
                    .rounded(px(8.))
                    .cursor_pointer()
                    .hover(|s| s.border_color(t.accent))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(
                                text(theme::LABEL)
                                    .text_color(t.ink)
                                    .child(take.name.clone()),
                            )
                            .child(
                                text(theme::CAPTION).text_color(t.muted).child(trf(
                                    "{dimension} · {seconds} s · {samples} samples",
                                    &[
                                        (
                                            "dimension",
                                            take.dimension.rsplit(':').next().unwrap_or("world"),
                                        ),
                                        ("seconds", &format!("{secs:.0}")),
                                        ("samples", &take.samples.to_string()),
                                    ],
                                )),
                            ),
                    )
                    .child(text(theme::CAPTION).text_color(t.muted).child(tr("Watch")))
                    .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                        this.open_replay(i, cx);
                    }))
            }),
        );
        Some((buttons, div().flex().flex_col().gap(px(8.)).child(rows)))
    }

    /// Opens a take in the viewer, loaded and parked at its start.
    fn open_replay(&mut self, i: usize, cx: &mut Context<Self>) {
        let Some(take) = self.replays.list.get(i).cloned() else {
            return;
        };
        match replays::load(&take.path) {
            Ok(track) => {
                self.replays.loaded = Some(track);
                self.replays.viewing = Some(i);
                self.replays.playhead = 0.0;
                self.replays.playing = false;
                self.replays.speed = 1;
                self.replays.last_step = Some(std::time::Instant::now());
                self.replays.note = None;
            }
            Err(e) => self.replays.note = Some(e),
        }
        cx.notify();
    }

    /// The viewer over the whole window, while a take is open.
    pub(super) fn replay_viewer(&mut self, cx: &mut Context<Self>) -> Option<Stateful<Div>> {
        let t = self.theme;
        let i = self.replays.viewing?;
        let take = self.replays.list.get(i)?.clone();
        let track = self.replays.loaded.as_ref()?;
        let length = match (track.samples.first(), track.samples.last()) {
            (Some(f), Some(l)) => (l.tick - f.tick).max(0),
            _ => 0,
        };
        let playhead = self.replays.playhead.clamp(0.0, length as f64);
        let fraction = match length {
            0 => 0.0,
            l => playhead / l as f64,
        };
        let speed = SPEEDS[self.replays.speed];
        let play_label = if self.replays.playing {
            tr("Pause").to_string()
        } else {
            tr("Play").to_string()
        };
        let close = cx.listener(|this, _: &ClickEvent, _, cx| {
            this.replays.viewing = None;
            this.replays.playing = false;
            cx.notify();
        });
        let top = div()
            .flex()
            .items_center()
            .gap(px(12.))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .child(blocks::pixel(17., FontWeight::MEDIUM).text_color(t.ink).child(take.name.clone()))
                    .child(
                        text(theme::CAPTION).text_color(t.muted).child(trf(
                            "{dimension} · {seconds} s · {speed}×",
                            &[
                                ("dimension", track.dimension.rsplit(':').next().unwrap_or("world")),
                                ("seconds", &format!("{:.0}", length as f64 / 20.0)),
                                ("speed", &format!("{}", speed)),
                            ],
                        )),
                    ),
            )
            .child(
                button("replay-close", "Close", Kind::Secondary, &t).on_click(close),
            );
        // The camera's path from above: dots along the track, the playhead's
        // marker on top. Dots sit by percent of the track's bounding box.
        let (mut min_x, mut max_x, mut min_z, mut max_z) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
        for s in &track.samples {
            min_x = min_x.min(s.x);
            max_x = max_x.max(s.x);
            min_z = min_z.min(s.z);
            max_z = max_z.max(s.z);
        }
        let span_x = (max_x - min_x).max(1.0);
        let span_z = (max_z - min_z).max(1.0);
        let step = track.samples.len().div_ceil(DOTS).max(1);
        let dots = div()
            .absolute()
            .size_full()
            .children(
                track
                    .samples
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| i % step == 0)
                    .map(|(_, s)| {
                        div()
                            .absolute()
                            .w(px(4.))
                            .h(px(4.))
                            .rounded(px(2.))
                            .bg(t.muted.opacity(0.7))
                            .left(relative(
                                ((s.x - min_x) / span_x * 0.96 + 0.02) as f32,
                            ))
                            .top(relative(
                                ((s.z - min_z) / span_z * 0.96 + 0.02) as f32,
                            ))
                    }),
            );
        let (mark_left, mark_top) = match replays::pose_at(track, playhead as i64) {
            Some((x, _, z, _, _)) => (
                ((x - min_x) / span_x * 0.96 + 0.02) as f32,
                ((z - min_z) / span_z * 0.96 + 0.02) as f32,
            ),
            None => (0.02, 0.02),
        };
        let plot = div()
            .relative()
            .w_full()
            .h(px(240.))
            .border_2()
            .border_color(t.outline)
            .bg(t.well)
            .child(dots)
            .child(
                div()
                    .absolute()
                    .w(px(9.))
                    .h(px(9.))
                    .rounded(px(4.5))
                    .bg(t.accent)
                    .left(relative(mark_left - 0.01))
                    .top(relative(mark_top - 0.01)),
            );
        // The scrubber: a bar the playhead fills; press, drag and release to
        // seek (the launch page's memory slider).
        let bar = self.replays.bar.clone();
        let track_len = length;
        let scrubber = div()
            .id("replay-timeline")
            .relative()
            .w_full()
            .h(px(20.))
            .border_2()
            .border_color(t.outline)
            .bg(t.well)
            .cursor_pointer()
            .child(
                canvas(
                    move |bounds, _, _| bar.set(Some(bounds)),
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full(),
            )
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top_0()
                    .h_full()
                    .w(relative(fraction as f32))
                    .bg(t.accent.opacity(0.55)),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                    this.seek_replay(ev.position, track_len, cx);
                }),
            )
            .on_mouse_move(cx.listener(move |this, ev: &MouseMoveEvent, _, cx| {
                if this.replays.dragging {
                    this.seek_replay(ev.position, track_len, cx);
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _, _| {
                    this.replays.dragging = false;
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _, _| {
                    this.replays.dragging = false;
                }),
            );
        // The pose the playhead sits at, and the transport.
        let pose = replays::pose_at(track, playhead as i64);
        let readout = match pose {
            Some((x, y, z, yaw, pitch)) => trf(
                "tick {tick} · {pos} · yaw {yaw}° pitch {pitch}°",
                &[
                    ("tick", &format!("{playhead:.0}")),
                    ("pos", &format!("{x:.0} {y:.0} {z:.0}")),
                    ("yaw", &format!("{yaw:.0}")),
                    ("pitch", &format!("{pitch:.0}")),
                ],
            ),
            None => tr("No camera samples.").to_string(),
        };
        let transport = div()
            .flex()
            .items_center()
            .gap(px(8.))
            .child(
                button("replay-play", play_label, Kind::Secondary, &t).on_click(cx.listener(
                    |this, _: &ClickEvent, _, cx| {
                        this.replays.playing = !this.replays.playing;
                        this.replays.last_step = Some(std::time::Instant::now());
                        cx.notify();
                    },
                )),
            )
            .child(
                button(
                    "replay-speed",
                    format!("{speed}×"),
                    Kind::Ghost,
                    &t,
                )
                .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                    this.replays.speed = (this.replays.speed + 1) % SPEEDS.len();
                    cx.notify();
                })),
            )
            .child(
                button("replay-start", tr("Start"), Kind::Ghost, &t).on_click(cx.listener(
                    |this, _: &ClickEvent, _, cx| {
                        this.replays.playhead = 0.0;
                        cx.notify();
                    },
                )),
            )
            .child(text(theme::CAPTION).text_color(t.muted).child(readout));
        let body = div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(plot)
            .child(scrubber)
            .child(transport);
        Some(
            div()
                .id("replay-viewer")
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .flex()
                .flex_col()
                .gap(px(14.))
                .px(px(32.))
                .py(px(24.))
                .bg(t.page.opacity(0.96))
                .on_scroll(|_, _, cx| cx.stop_propagation())
                .occlude()
                .child(top)
                .child(body),
        )
    }

    /// Seeks the scrubber: the click's share of the bar, in ticks.
    fn seek_replay(&mut self, at: gpui::Point<Pixels>, length: i64, cx: &mut Context<Self>) {
        let Some(area) = self.replays.bar.get() else {
            return;
        };
        let width: f32 = area.size.width.into();
        if width <= 0. {
            return;
        }
        let across: f32 = (at.x - area.origin.x).into();
        let fraction = (across / width).clamp(0., 1.);
        self.replays.playhead = fraction * length as f64;
        self.replays.last_step = Some(std::time::Instant::now());
        cx.notify();
    }

    /// Playback: the playhead follows the wall clock at the take's speed.
    pub(super) fn replay_play_step(&mut self, cx: &mut Context<Self>) {
        let Some(track) = &self.replays.loaded else {
            return;
        };
        let length = match (track.samples.first(), track.samples.last()) {
            (Some(f), Some(l)) => (l.tick - f.tick).max(0) as f64,
            _ => 0.0,
        };
        if !self.replays.playing || length == 0.0 {
            return;
        }
        let now = std::time::Instant::now();
        let dt = self
            .replays
            .last_step
            .replace(now)
            .map_or(0.0, |t| now.duration_since(t).as_secs_f64());
        let next = self.replays.playhead + dt * 20.0 * SPEEDS[self.replays.speed];
        if next >= length {
            self.replays.playhead = length;
            self.replays.playing = false;
        } else {
            self.replays.playhead = next;
        }
        cx.notify();
    }
}
