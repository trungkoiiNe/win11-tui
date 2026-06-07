use ratatui::style::{Color, Modifier, Style};

pub const BG: Color = Color::Rgb(13, 17, 23);
pub const FG: Color = Color::Rgb(201, 209, 217);
pub const NVIDIA_GREEN: Color = Color::Rgb(118, 185, 0);
pub const EVERGLADE: Color = Color::Rgb(18, 49, 35);
pub const MUTED: Color = Color::Rgb(110, 118, 129);
pub const RED: Color = Color::Rgb(248, 81, 73);
pub const YELLOW: Color = Color::Rgb(210, 153, 34);
pub const BLUE: Color = Color::Rgb(56, 139, 253);

pub fn text() -> Style { Style::default().fg(FG) }
pub fn muted() -> Style { Style::default().fg(MUTED) }
pub fn heading() -> Style { Style::default().fg(FG).add_modifier(Modifier::BOLD) }
pub fn accent() -> Style { Style::default().fg(NVIDIA_GREEN) }
pub fn accent_bold() -> Style { Style::default().fg(NVIDIA_GREEN).add_modifier(Modifier::BOLD) }
pub fn selected() -> Style { Style::default().add_modifier(Modifier::BOLD) }
pub fn border() -> Style { Style::default().fg(EVERGLADE) }
pub fn border_focused() -> Style { Style::default().fg(NVIDIA_GREEN) }
pub fn status_ok() -> Style { Style::default().fg(NVIDIA_GREEN) }
pub fn status_warn() -> Style { Style::default().fg(YELLOW) }
pub fn status_err() -> Style { Style::default().fg(RED) }
pub fn key_hint() -> Style { Style::default().fg(NVIDIA_GREEN) }
pub fn title_bar() -> Style { Style::default().fg(FG).bg(EVERGLADE).add_modifier(Modifier::BOLD) }
pub fn badge() -> Style { Style::default().fg(BG).bg(NVIDIA_GREEN).add_modifier(Modifier::BOLD) }
