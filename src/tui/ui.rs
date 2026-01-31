use crate::tui::app::{ActiveTab, App};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph, Tabs, Wrap},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title/Tabs
            Constraint::Min(0),    // Main Content
            Constraint::Length(3), // Message/Status
            Constraint::Length(1), // Help
        ])
        .split(frame.area());

    render_tabs(frame, app, chunks[0]);
    render_content(frame, app, chunks[1]);
    render_message(frame, app, chunks[2]);
    render_help(frame, chunks[3]);
}

// ... existing code ...

fn render_message(frame: &mut Frame, app: &App, area: Rect) {
    if app.is_processing {
        let (title, color, label) = if app.is_complete {
            ("Completed", Color::Cyan, Span::raw(&app.message))
        } else {
            let label = if let Some(last_log) = app.logs.last() {
                Span::raw(last_log)
            } else {
                Span::raw("Processing...")
            };
            ("Processing", Color::Green, label)
        };

        // Show progress bar
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(title))
            .gauge_style(Style::default().fg(color))
            .percent((app.progress * 100.0) as u16)
            .label(label);

        frame.render_widget(gauge, area);
    } else {
        let paragraph = Paragraph::new(app.message.as_str())
            .block(Block::default().borders(Borders::ALL).title("Status"))
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
    }
}

fn render_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles_list = vec!["Combine", "Compress", "Add Music", "Timelapse", "Info"];
    let inner_width = area.width.saturating_sub(2) as usize;
    let tab_width = inner_width / titles_list.len();

    let titles = titles_list
        .iter()
        .map(|t| Line::from(format!("{:^width$}", t, width = tab_width)))
        .collect::<Vec<_>>();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Framix")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded),
        )
        .select(app.active_tab as usize)
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .divider("");

    frame.render_widget(tabs, area);
}

fn render_content(frame: &mut Frame, app: &App, area: Rect) {
    match app.active_tab {
        ActiveTab::Combine => render_combine(frame, app, area),
        ActiveTab::Compress => render_compress(frame, app, area),
        ActiveTab::AddMusic => render_add_music(frame, app, area),
        ActiveTab::Timelapse => render_timelapse(frame, app, area),
        ActiveTab::Info => render_info(frame, app, area),
    }
}

fn render_input(frame: &mut Frame, label: &str, value: &str, is_selected: bool, area: Rect) {
    let style = if is_selected {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(label)
        .border_style(style);

    let paragraph = Paragraph::new(value).block(block);
    frame.render_widget(paragraph, area);
}

fn render_combine(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3)])
        .split(area);

    render_input(
        frame,
        &app.combine_inputs.label,
        &app.combine_inputs.value,
        app.selected_field == 0,
        chunks[0],
    );
    render_input(
        frame,
        &app.combine_output.label,
        &app.combine_output.value,
        app.selected_field == 1,
        chunks[1],
    );
}

fn render_compress(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(area);

    render_input(
        frame,
        &app.compress_input.label,
        &app.compress_input.value,
        app.selected_field == 0,
        chunks[0],
    );
    render_input(
        frame,
        &app.compress_output.label,
        &app.compress_output.value,
        app.selected_field == 1,
        chunks[1],
    );
    render_input(
        frame,
        &app.compress_crf.label,
        &app.compress_crf.value,
        app.selected_field == 2,
        chunks[2],
    );
}

fn render_add_music(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(area);

    render_input(
        frame,
        &app.music_video.label,
        &app.music_video.value,
        app.selected_field == 0,
        chunks[0],
    );
    render_input(
        frame,
        &app.music_audio.label,
        &app.music_audio.value,
        app.selected_field == 1,
        chunks[1],
    );
    render_input(
        frame,
        &app.music_output.label,
        &app.music_output.value,
        app.selected_field == 2,
        chunks[2],
    );
    render_input(
        frame,
        &app.music_reduce.label,
        &app.music_reduce.value,
        app.selected_field == 3,
        chunks[3],
    );
}

fn render_timelapse(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(area);

    render_input(
        frame,
        &app.time_input.label,
        &app.time_input.value,
        app.selected_field == 0,
        chunks[0],
    );
    render_input(
        frame,
        &app.time_output.label,
        &app.time_output.value,
        app.selected_field == 1,
        chunks[1],
    );
    render_input(
        frame,
        &app.time_speed.label,
        &app.time_speed.value,
        app.selected_field == 2,
        chunks[2],
    );
}

fn render_info(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3)])
        .split(area);

    render_input(
        frame,
        &app.info_input.label,
        &app.info_input.value,
        app.selected_field == 0,
        chunks[0],
    );
}

fn render_help(frame: &mut Frame, area: Rect) {
    let help = Span::raw("TAB: Autocomplete | SHIFT+TAB: Switch Tab | ENTER: Next Field | Ctrl+E / SHIFT+ENTER: Execute | Ctrl+C: Quit");
    let paragraph = Paragraph::new(Line::from(help)).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(paragraph, area);
}
