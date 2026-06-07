mod app;
mod event;
mod registry;
mod theme;
mod tweaks;
mod ui;

use app::App;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use event::{AppEvent, EventHandler};
use ratatui::prelude::*;
use std::io;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Event handler (keyboard + tick)
    let tick_rate = Duration::from_millis(100);
    let mut events = EventHandler::new(tick_rate);

    // Main loop
    loop {
        // Draw
        terminal.draw(|frame| ui::draw(frame, &app))?;

        // Handle events
        match events.next().await {
            Some(AppEvent::Key(key)) => {
                // Ctrl+C force quit
                if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                    && key.code == crossterm::event::KeyCode::Char('c')
                {
                    break;
                }
                app.handle_key(key);
            }
            Some(AppEvent::Tick) => {}
            None => break,
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
