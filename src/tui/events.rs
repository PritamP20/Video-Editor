use crate::tui::app::App;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::time::Duration;

pub fn handle_events(app: &mut App) -> Result<bool> {
    if event::poll(Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if app.is_complete {
                app.is_processing = false;
                app.is_complete = false;
                app.message.clear();
                return Ok(false);
            }

            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("debug_keys.log")
            {
                use std::io::Write;
                writeln!(file, "Key: {:?}, Modifiers: {:?}", key.code, key.modifiers).ok();
            }

            match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.running = false;
                    return Ok(true);
                }
                KeyCode::Tab => app.autocomplete(),
                KeyCode::BackTab => app.next_tab(),
                KeyCode::Up => app.prev_field(),
                KeyCode::Down => app.next_field(),
                KeyCode::Right => app.autocomplete(),
                KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(true)
                }
                KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => return Ok(true),
                KeyCode::Enter => app.next_field(),
                KeyCode::Char(c) => app.input(c),
                KeyCode::Backspace => app.backspace(),
                KeyCode::Esc => app.running = false,
                _ => {}
            }
        }
    }
    Ok(false)
}
