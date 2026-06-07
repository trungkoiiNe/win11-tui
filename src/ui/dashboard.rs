use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // category list
            Constraint::Percentage(50), // detail panel
        ])
        .split(area);

    draw_category_list(frame, app, chunks[0]);
    draw_detail_panel(frame, app, chunks[1]);
}

fn draw_category_list(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(" Categories ", theme::heading()))
        .borders(Borders::ALL)
        .border_style(theme::border_focused())
        .style(theme::text());

    let items: Vec<ListItem> = app
        .categories
        .iter()
        .enumerate()
        .map(|(i, cat)| {
            let applied = cat.applied_count();
            let total = cat.tweaks.len();
            let is_selected = i == app.selected_category;

            let status_color = if applied == total {
                theme::NVIDIA_GREEN
            } else if applied > 0 {
                theme::YELLOW
            } else {
                theme::MUTED
            };

            let marker = if is_selected { "▌" } else { " " };

            let line = Line::from(vec![
                Span::styled(marker, theme::accent()),
                Span::styled(format!(" {} ", cat.icon), theme::text()),
                Span::styled(cat.name, if is_selected { theme::selected() } else { theme::text() }),
                Span::styled(
                    format!("  {}/{}", applied, total),
                    Style::default().fg(status_color),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn draw_detail_panel(frame: &mut Frame, app: &App, area: Rect) {
    let cat = app.current_category();
    let applied = cat.applied_count();
    let total = cat.tweaks.len();

    let block = Block::default()
        .title(Span::styled(
            format!(" {} {} ", cat.icon, cat.name),
            theme::heading(),
        ))
        .borders(Borders::ALL)
        .border_style(theme::border())
        .style(theme::text());

    let mut lines = vec![
        Line::from(Span::styled(cat.description, theme::muted())),
        Line::from(""),
        Line::from(vec![
            Span::styled("Applied: ", theme::text()),
            Span::styled(
                format!("{}/{}", applied, total),
                if applied == total {
                    theme::status_ok()
                } else {
                    theme::status_warn()
                },
            ),
        ]),
        Line::from(""),
    ];

    // Show per-category breakdown
    for tweak in &cat.tweaks {
        let status = tweak.get_status();
        let icon = style_for_status(status);
        let line = Line::from(vec![
            Span::styled(format!(" {} ", status.symbol()), icon),
            Span::styled(tweak.name, theme::text()),
        ]);
        lines.push(line);
    }

    let para = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(para, area);
}

fn style_for_status(status: crate::tweaks::TweakStatus) -> Style {
    match status {
        crate::tweaks::TweakStatus::Applied => theme::status_ok(),
        crate::tweaks::TweakStatus::Default => theme::status_err(),
        crate::tweaks::TweakStatus::Unknown => theme::status_warn(),
        crate::tweaks::TweakStatus::Error => theme::status_err(),
    }
}
