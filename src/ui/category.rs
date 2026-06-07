use crate::app::App;
use crate::theme;
use crate::tweaks::TweakStatus;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // tweak list
            Constraint::Percentage(40), // detail panel
        ])
        .split(area);

    draw_tweak_list(frame, app, chunks[0]);
    draw_tweak_detail(frame, app, chunks[1]);
}

fn draw_tweak_list(frame: &mut Frame, app: &App, area: Rect) {
    let cat = app.current_category();
    let block = Block::default()
        .title(Span::styled(
            format!(" {} Tweaks ", cat.icon),
            theme::heading(),
        ))
        .borders(Borders::ALL)
        .border_style(theme::border_focused())
        .style(theme::text());

    let area_inner = block.inner(area);
    frame.render_widget(block, area);

    let total = cat.tweaks.len();
    if total == 0 {
        return;
    }

    // Visible rows
    let visible = area_inner.height as usize;
    let mut offset = app.scroll_offset;
    if app.selected_tweak >= offset + visible {
        offset = app.selected_tweak.saturating_sub(visible - 1);
    }

    let items: Vec<ListItem> = cat
        .tweaks
        .iter()
        .enumerate()
        .skip(offset)
        .take(visible)
        .map(|(i, tweak)| {
            let status = tweak.get_status();
            let is_selected = i == app.selected_tweak;

            let status_style = match status {
                TweakStatus::Applied => theme::status_ok(),
                TweakStatus::Default => theme::status_err(),
                TweakStatus::Unknown => theme::status_warn(),
                TweakStatus::Error => theme::status_err(),
            };

            let marker = if is_selected { "▌" } else { " " };

            let line = Line::from(vec![
                Span::styled(marker, theme::accent()),
                Span::styled(
                    format!(" {} ", status.symbol()),
                    status_style,
                ),
                Span::styled(
                    tweak.name,
                    if is_selected {
                        theme::selected()
                    } else {
                        theme::text()
                    },
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, area_inner);

    // Scroll indicator
    if total > visible {
        let scroll_text = format!(
            " {}/{} ",
            app.selected_tweak + 1,
            total
        );
        let scroll_area = Rect {
            x: area_inner.x + area_inner.width.saturating_sub(scroll_text.len() as u16 + 2),
            y: area_inner.y,
            width: scroll_text.len() as u16 + 2,
            height: 1,
        };
        let para = Paragraph::new(Span::styled(scroll_text, theme::muted()));
        frame.render_widget(para, scroll_area);
    }
}

fn draw_tweak_detail(frame: &mut Frame, app: &App, area: Rect) {
    let cat = app.current_category();
    let tweak = &cat.tweaks[app.selected_tweak];
    let status = tweak.get_status();

    let block = Block::default()
        .title(Span::styled(" Detail ", theme::heading()))
        .borders(Borders::ALL)
        .border_style(theme::border())
        .style(theme::text());

    let status_label = match status {
        TweakStatus::Applied => Span::styled(" APPLIED ", theme::badge()),
        TweakStatus::Default => Span::styled(
            " DEFAULT ",
            Style::default().fg(theme::BG).bg(theme::RED).add_modifier(Modifier::BOLD),
        ),
        TweakStatus::Unknown => Span::styled(
            " UNKNOWN ",
            Style::default().fg(theme::BG).bg(theme::YELLOW).add_modifier(Modifier::BOLD),
        ),
        TweakStatus::Error => Span::styled(
            " ERROR ",
            Style::default().fg(theme::BG).bg(theme::RED).add_modifier(Modifier::BOLD),
        ),
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Name: ", theme::muted()),
            Span::styled(tweak.name, theme::accent_bold()),
        ]),
        Line::from(""),
        Line::from(Span::styled(tweak.description, theme::text())),
        Line::from(""),
        Line::from(vec![
            Span::styled("Status: ", theme::muted()),
            status_label,
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Hive: ", theme::muted()),
            Span::styled(tweak.hive.as_str(), theme::text()),
        ]),
        Line::from(vec![
            Span::styled("Key: ", theme::muted()),
            Span::styled(tweak.subkey, theme::text()),
        ]),
        Line::from(vec![
            Span::styled("Value: ", theme::muted()),
            Span::styled(tweak.value_name, theme::text()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Optimized: ", theme::muted()),
            Span::styled(tweak.optimized.to_display(), theme::status_ok()),
        ]),
        Line::from(vec![
            Span::styled("Default: ", theme::muted()),
            Span::styled(tweak.default.to_display(), theme::status_err()),
        ]),
    ];

    if !tweak.note.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Note: ", theme::muted()),
            Span::styled(tweak.note, theme::status_warn()),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[Enter] Toggle  [a] Apply  [r] Revert",
        theme::muted(),
    )));

    let para = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(para, area);
}
