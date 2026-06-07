pub mod dashboard;
pub mod category;

use crate::app::{App, Screen};
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Vertical layout: title | content | nav | status
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title bar
            Constraint::Min(0),   // content
            Constraint::Length(1), // nav bar
            Constraint::Length(1), // status bar
        ])
        .split(area);

    // Title bar
    draw_title_bar(frame, app, chunks[0]);

    // Content
    match app.screen {
        Screen::Dashboard => dashboard::draw(frame, app, chunks[1]),
        Screen::Category => category::draw(frame, app, chunks[1]),
    }

    // Nav bar
    draw_nav_bar(frame, app, chunks[2]);

    // Status bar
    draw_status_bar(frame, app, chunks[3]);
}

fn draw_title_bar(frame: &mut Frame, app: &App, area: Rect) {
    let total_applied = app.total_applied();
    let total_tweaks = app.total_tweaks();

    let context = match app.screen {
        Screen::Dashboard => "Dashboard",
        Screen::Category => app.current_category().name,
    };

    let title = format!(
        " Win11 Optimizer │ {}/{} applied │ {} ",
        total_applied, total_tweaks, context
    );

    let para = Paragraph::new(Span::styled(&title, theme::title_bar()));
    frame.render_widget(para, area);
}

fn draw_nav_bar(frame: &mut Frame, app: &App, area: Rect) {
    let hints = match app.screen {
        Screen::Dashboard => {
            if app.confirm.is_some() {
                "[y] Confirm  [n] Cancel"
            } else {
                "[↑↓] Navigate  [Enter] Open  [A] Apply All  [R] Revert All  [:] Command  [q] Quit"
            }
        }
        Screen::Category => {
            if app.confirm.is_some() {
                "[y] Confirm  [n] Cancel"
            } else {
                "[↑↓] Navigate  [Enter] Toggle  [A] Apply All  [R] Revert All  [Esc] Back  [q] Quit"
            }
        }
    };

    let spans = vec![
        Span::styled(" ", theme::muted()),
        Span::styled(hints, theme::key_hint()),
    ];

    let para = Paragraph::new(Line::from(spans));
    frame.render_widget(para, area);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(ref msg) = app.status {
        let style = if msg.is_error {
            theme::status_err()
        } else {
            theme::status_ok()
        };
        let para = Paragraph::new(Span::styled(format!(" {}", msg.text), style));
        frame.render_widget(para, area);
    } else if let Some(action) = app.confirm {
        let text = match action {
            crate::app::ConfirmAction::ApplyAll => "Apply ALL optimizations? [y/n]",
            crate::app::ConfirmAction::RevertAll => "Revert ALL to Windows defaults? [y/n]",
            crate::app::ConfirmAction::ApplyTweak(_) => "Apply this tweak? [y/n]",
            crate::app::ConfirmAction::RevertTweak(_) => "Revert this tweak? [y/n]",
        };
        let para = Paragraph::new(Span::styled(
            format!(" ⚠ {}", text),
            theme::status_warn(),
        ));
        frame.render_widget(para, area);
    } else {
        let para = Paragraph::new(Span::styled(" Ready", theme::muted()));
        frame.render_widget(para, area);
    }
}
