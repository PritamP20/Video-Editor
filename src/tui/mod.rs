pub mod app;
pub mod events;
pub mod ui;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

use crate::commands::{self, ProgressInfo};
use app::{ActiveTab, App};
use events::handle_events;
use ui::render;

pub enum AppEvent {
    Progress(ProgressInfo),
    Done,
    Error(String),
}

pub fn run() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let (tx, rx) = mpsc::channel();

    loop {
        terminal.draw(|f| render(f, &app))?;

        // Process channel messages
        while let Ok(event) = rx.try_recv() {
            match event {
                AppEvent::Progress(info) => match info {
                    ProgressInfo::Log(log) => app.logs.push(log),
                    ProgressInfo::Percentage(p) => app.progress = p,
                },
                AppEvent::Done => {
                    app.is_processing = true; // Keep processing view active
                    app.is_complete = true;
                    app.message =
                        "Process Completed Successfully! Press any key to continue.".to_string();
                    app.progress = 1.0;
                }
                AppEvent::Error(e) => {
                    app.is_processing = false;
                    app.message = format!("Error: {}", e);
                    app.logs.push(format!("Error: {}", e));
                }
            }
        }

        if !app.running {
            break;
        }

        if let Ok(should_run) = handle_events(&mut app) {
            if should_run {
                if app.is_processing {
                    app.message = "Already processing...".to_string();
                } else {
                    app.is_processing = true;
                    app.progress = 0.0;
                    app.logs.clear();
                    app.message = "Starting...".to_string();

                    let tx_clone = tx.clone();
                    execute_command(&app, tx_clone);
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn execute_command(app: &App, tx: mpsc::Sender<AppEvent>) {
    // Clone necessary data to move into thread
    let active_tab = app.active_tab;
    // We clone inputs because we can't pass reference to app into thread
    // This is a bit tedious but safe
    let combine_inputs = app.combine_inputs.value.clone();
    let combine_output = app.combine_output.value.clone();

    let compress_input = app.compress_input.value.clone();
    let compress_output = app.compress_output.value.clone();
    let compress_crf = app.compress_crf.value.clone();

    let music_video = app.music_video.value.clone();
    let music_audio = app.music_audio.value.clone();
    let music_output = app.music_output.value.clone();
    let music_reduce = app.music_reduce.value.clone();

    let time_input = app.time_input.value.clone();
    let time_output = app.time_output.value.clone();
    let time_speed = app.time_speed.value.clone();

    let info_input = app.info_input.value.clone();

    thread::spawn(move || {
        let res = match active_tab {
            ActiveTab::Combine => {
                let inputs: Vec<PathBuf> = combine_inputs
                    .split_whitespace()
                    .map(PathBuf::from)
                    .collect();
                let output = Path::new(&combine_output);
                commands::combine_videos(&inputs, output, |info| {
                    let _ = tx.send(AppEvent::Progress(info));
                })
            }
            ActiveTab::Compress => {
                let input = Path::new(&compress_input);
                let output = Path::new(&compress_output);
                let crf: u8 = compress_crf.parse().unwrap_or(23);
                commands::compress_video(input, output, crf, |info| {
                    let _ = tx.send(AppEvent::Progress(info));
                })
            }
            ActiveTab::AddMusic => {
                let video = Path::new(&music_video);
                let audio = Path::new(&music_audio);
                let output = Path::new(&music_output);
                let reduce = &music_reduce;
                commands::add_music(video, audio, output, reduce, |info| {
                    let _ = tx.send(AppEvent::Progress(info));
                })
            }
            ActiveTab::Timelapse => {
                let input = Path::new(&time_input);
                let output = Path::new(&time_output);
                let speed: f64 = time_speed.parse().unwrap_or(10.0);
                commands::timelapse(input, output, speed, |info| {
                    let _ = tx.send(AppEvent::Progress(info));
                })
            }
            ActiveTab::Info => {
                let input = Path::new(&info_input);
                commands::get_info(input, |info| {
                    let _ = tx.send(AppEvent::Progress(info));
                })
            }
        };

        match res {
            Ok(_) => {
                let _ = tx.send(AppEvent::Done);
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Error(e.to_string()));
            }
        }
    });
}
