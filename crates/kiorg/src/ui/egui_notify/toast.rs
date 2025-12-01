use crate::ui::egui_notify::{Anchor, TOAST_HEIGHT, TOAST_WIDTH};
use egui::{Color32, Pos2, Rect, WidgetText, pos2, vec2};
use std::{fmt::Debug, time::Duration};

/// Level of importance
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum ToastLevel {
    #[default]
    Info,
    Warning,
    Error,
    Success,
    None,
    Custom(String, Color32),
}

#[derive(Debug)]
/// State of the toast
pub enum ToastState {
    /// Toast is appearing
    Appear,
    /// Toast is disappearing
    Disappear,
    /// Toast has disappeared
    Disappeared,
    /// Toast is idling
    Idle,
}

impl ToastState {
    /// Returns `true` if the toast is appearing
    pub const fn appearing(&self) -> bool {
        matches!(self, Self::Appear)
    }

    /// Returns `true` if the toast is disappearing
    pub const fn disappearing(&self) -> bool {
        matches!(self, Self::Disappear)
    }

    /// Returns `true` if the toast has disappeared
    pub const fn disappeared(&self) -> bool {
        matches!(self, Self::Disappeared)
    }

    /// Returns `true` if the toast is idling
    pub const fn idling(&self) -> bool {
        matches!(self, Self::Idle)
    }
}

/// Container for options for initializing toasts
pub struct ToastOptions {
    duration: Option<Duration>,
    level: ToastLevel,
    closable: bool,
    show_progress_bar: bool,
}

/// Single notification or *toast*
pub struct Toast {
    pub(crate) level: ToastLevel,
    pub(crate) caption: WidgetText,
    // (initial, current)
    pub(crate) duration: Option<(f32, f32)>,
    pub(crate) height: f32,
    pub(crate) width: f32,
    pub(crate) closable: bool,
    pub(crate) show_progress_bar: bool,
    pub(crate) state: ToastState,
    pub(crate) value: f32,
}

impl Default for ToastOptions {
    fn default() -> Self {
        Self {
            duration: Some(Duration::from_millis(3500)),
            level: ToastLevel::None,
            closable: true,
            show_progress_bar: true,
        }
    }
}

fn duration_to_seconds_f32(duration: Duration) -> f32 {
    duration.as_nanos() as f32 * 1e-9
}

impl Toast {
    fn new(caption: impl Into<WidgetText>, options: ToastOptions) -> Self {
        Self {
            caption: caption.into(),
            height: TOAST_HEIGHT,
            width: TOAST_WIDTH,
            duration: options.duration.map(|dur| {
                let max_dur = duration_to_seconds_f32(dur);
                (max_dur, max_dur)
            }),
            closable: options.closable,
            show_progress_bar: options.show_progress_bar,
            level: options.level,
            value: 0.,
            state: ToastState::Appear,
        }
    }

    /// Creates new basic toast, can be closed by default.
    pub fn basic(caption: impl Into<WidgetText>) -> Self {
        Self::new(caption, ToastOptions::default())
    }

    /// Creates new success toast, can be closed by default.
    pub fn success(caption: impl Into<WidgetText>) -> Self {
        Self::new(
            caption,
            ToastOptions {
                level: ToastLevel::Success,
                ..ToastOptions::default()
            },
        )
    }

    /// Creates new info toast, can be closed by default.
    pub fn info(caption: impl Into<WidgetText>) -> Self {
        Self::new(
            caption,
            ToastOptions {
                level: ToastLevel::Info,
                ..ToastOptions::default()
            },
        )
    }

    /// Creates new warning toast, can be closed by default.
    pub fn warning(caption: impl Into<WidgetText>) -> Self {
        Self::new(
            caption,
            ToastOptions {
                level: ToastLevel::Warning,
                ..ToastOptions::default()
            },
        )
    }

    /// Creates new error toast, can not be closed by default.
    pub fn error(caption: impl Into<WidgetText>) -> Self {
        Self::new(
            caption,
            ToastOptions {
                closable: false,
                level: ToastLevel::Error,
                ..ToastOptions::default()
            },
        )
    }

    /// Creates new custom toast, can be closed by default.
    pub fn custom(caption: impl Into<WidgetText>, level: ToastLevel) -> Self {
        Self::new(
            caption,
            ToastOptions {
                level,
                ..ToastOptions::default()
            },
        )
    }

    /// Set the options with a [`ToastOptions`]
    pub fn options(&mut self, options: ToastOptions) -> &mut Self {
        self.closable(options.closable);
        self.duration(options.duration);
        self.level(options.level);
        self
    }

    /// Change the level of the toast
    pub fn level(&mut self, level: ToastLevel) -> &mut Self {
        self.level = level;
        self
    }

    /// Can the user close the toast?
    pub fn closable(&mut self, closable: bool) -> &mut Self {
        self.closable = closable;
        self
    }

    /// Should a progress bar be shown?
    pub fn show_progress_bar(&mut self, show_progress_bar: bool) -> &mut Self {
        self.show_progress_bar = show_progress_bar;
        self
    }

    /// In what time should the toast expire? Set to `None` for no expiry.
    pub fn duration(&mut self, duration: Option<Duration>) -> &mut Self {
        if let Some(duration) = duration {
            let max_dur = duration_to_seconds_f32(duration);
            self.duration = Some((max_dur, max_dur));
        } else {
            self.duration = None;
        }
        self
    }

    /// Toast's box height
    pub fn height(&mut self, height: f32) -> &mut Self {
        self.height = height;
        self
    }

    /// Toast's box width
    pub fn width(&mut self, width: f32) -> &mut Self {
        self.width = width;
        self
    }

    /// Dismiss this toast
    pub fn dismiss(&mut self) {
        self.state = ToastState::Disappear;
    }

    pub(crate) fn calc_anchored_rect(&self, pos: Pos2, anchor: Anchor) -> Rect {
        match anchor {
            Anchor::TopRight => Rect {
                min: pos2(pos.x - self.width, pos.y),
                max: pos2(pos.x, pos.y + self.height),
            },
            Anchor::TopLeft => Rect {
                min: pos,
                max: pos + vec2(self.width, self.height),
            },
            Anchor::BottomRight => Rect {
                min: pos - vec2(self.width, self.height),
                max: pos,
            },
            Anchor::BottomLeft => Rect {
                min: pos2(pos.x, pos.y - self.height),
                max: pos2(pos.x + self.width, pos.y),
            },
        }
    }

    pub(crate) fn adjust_next_pos(&self, pos: &mut Pos2, anchor: Anchor, spacing: f32) {
        match anchor {
            Anchor::TopRight | Anchor::TopLeft => pos.y += self.height + spacing,
            Anchor::BottomRight | Anchor::BottomLeft => pos.y -= self.height + spacing,
        }
    }
}
