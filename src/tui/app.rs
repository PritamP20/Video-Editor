use std::path::Path;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ActiveTab {
    Combine,
    Compress,
    AddMusic,
    Timelapse,
    Info,
}

impl Default for ActiveTab {
    fn default() -> Self {
        ActiveTab::Combine
    }
}

#[derive(Debug, Default, Clone)]
pub struct InputField {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Default)]
pub struct App {
    pub active_tab: ActiveTab,
    pub running: bool,

    pub combine_inputs: InputField,
    pub combine_output: InputField,

    pub compress_input: InputField,
    pub compress_output: InputField,
    pub compress_crf: InputField,

    pub music_video: InputField,
    pub music_audio: InputField,
    pub music_output: InputField,
    pub music_reduce: InputField,

    pub time_input: InputField,
    pub time_output: InputField,
    pub time_speed: InputField,
    pub info_input: InputField,

    pub message: String,
    pub selected_field: usize,

    pub progress: f64,
    pub is_processing: bool,
    pub is_complete: bool,
    pub logs: Vec<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            active_tab: ActiveTab::default(),

            combine_inputs: InputField {
                label: "Inputs (space separated)".into(),
                ..Default::default()
            },
            combine_output: InputField {
                label: "Output Path".into(),
                ..Default::default()
            },

            compress_input: InputField {
                label: "Input Video".into(),
                ..Default::default()
            },
            compress_output: InputField {
                label: "Output Path".into(),
                ..Default::default()
            },
            compress_crf: InputField {
                label: "CRF (0-51, Default: 23)".into(),
                value: "23".into(),
            },

            music_video: InputField {
                label: "Video Path".into(),
                ..Default::default()
            },
            music_audio: InputField {
                label: "Audio Path".into(),
                ..Default::default()
            },
            music_output: InputField {
                label: "Output Path".into(),
                ..Default::default()
            },
            music_reduce: InputField {
                label: "Original Volume (0.0-1.0)".into(),
                value: "1.0".into(),
            },

            time_input: InputField {
                label: "Input Video".into(),
                ..Default::default()
            },
            time_output: InputField {
                label: "Output Path".into(),
                ..Default::default()
            },
            time_speed: InputField {
                label: "Speed Factor".into(),
                value: "10.0".into(),
            },

            info_input: InputField {
                label: "Video Path".into(),
                ..Default::default()
            },

            message: String::new(),
            selected_field: 0,
            progress: 0.0,
            is_processing: false,
            is_complete: false,
            logs: Vec::new(),
        }
    }

    pub fn next_tab(&mut self) {
        self.active_tab = match self.active_tab {
            ActiveTab::Combine => ActiveTab::Compress,
            ActiveTab::Compress => ActiveTab::AddMusic,
            ActiveTab::AddMusic => ActiveTab::Timelapse,
            ActiveTab::Timelapse => ActiveTab::Info,
            ActiveTab::Info => ActiveTab::Combine,
        };
        self.selected_field = 0;
        self.message.clear();
    }

    pub fn prev_tab(&mut self) {
        self.active_tab = match self.active_tab {
            ActiveTab::Combine => ActiveTab::Info,
            ActiveTab::Compress => ActiveTab::Combine,
            ActiveTab::AddMusic => ActiveTab::Compress,
            ActiveTab::Timelapse => ActiveTab::AddMusic,
            ActiveTab::Info => ActiveTab::Timelapse,
        };
        self.selected_field = 0;
        self.message.clear();
    }

    pub fn next_field(&mut self) {
        let max_fields = self.get_field_count();
        if self.selected_field < max_fields - 1 {
            self.selected_field += 1;
        } else {
            self.selected_field = 0;
        }
    }

    pub fn prev_field(&mut self) {
        let max_fields = self.get_field_count();
        if self.selected_field > 0 {
            self.selected_field -= 1;
        } else {
            self.selected_field = max_fields - 1;
        }
    }

    fn get_field_count(&self) -> usize {
        match self.active_tab {
            ActiveTab::Combine => 2,
            ActiveTab::Compress => 3,
            ActiveTab::AddMusic => 4,
            ActiveTab::Timelapse => 3,
            ActiveTab::Info => 1,
        }
    }

    pub fn input(&mut self, c: char) {
        let field = self.get_active_field_mut();
        field.value.push(c);
    }

    pub fn backspace(&mut self) {
        let field = self.get_active_field_mut();
        field.value.pop();
    }

    pub fn autocomplete(&mut self) {
        let is_multi_value =
            matches!(self.active_tab, ActiveTab::Combine) && self.selected_field == 0;
        let field = self.get_active_field_mut();
        let full_text = field.value.clone();

        let (prefix, current_word) = if is_multi_value {
            match full_text.rfind(char::is_whitespace) {
                Some(idx) => (&full_text[..=idx], &full_text[idx + 1..]),
                None => ("", full_text.as_str()),
            }
        } else {
            ("", full_text.as_str())
        };

        let (dir, file_part) = match current_word.rfind(std::path::MAIN_SEPARATOR) {
            Some(idx) => (&current_word[..=idx], &current_word[idx + 1..]),
            None => ("", current_word),
        };

        let search_dir = if dir.is_empty() { "." } else { dir };

        if let Ok(entries) = std::fs::read_dir(search_dir) {
            let mut matches: Vec<String> = entries
                .flatten()
                .map(|e| e.file_name().to_string_lossy().to_string())
                .filter(|name| name.starts_with(file_part))
                .collect();

            matches.sort();

            if let Some(match_name) = matches.first() {
                let mut new_path = String::from(dir);
                new_path.push_str(match_name);

                let path_check = Path::new(&new_path);
                if path_check.is_dir() {
                    new_path.push(std::path::MAIN_SEPARATOR);
                }

                field.value = format!("{}{}", prefix, new_path);
            }
        }
    }

    fn get_active_field_mut(&mut self) -> &mut InputField {
        match self.active_tab {
            ActiveTab::Combine => match self.selected_field {
                0 => &mut self.combine_inputs,
                1 => &mut self.combine_output,
                _ => &mut self.combine_inputs,
            },
            ActiveTab::Compress => match self.selected_field {
                0 => &mut self.compress_input,
                1 => &mut self.compress_output,
                2 => &mut self.compress_crf,
                _ => &mut self.compress_input,
            },
            ActiveTab::AddMusic => match self.selected_field {
                0 => &mut self.music_video,
                1 => &mut self.music_audio,
                2 => &mut self.music_output,
                3 => &mut self.music_reduce,
                _ => &mut self.music_video,
            },
            ActiveTab::Timelapse => match self.selected_field {
                0 => &mut self.time_input,
                1 => &mut self.time_output,
                2 => &mut self.time_speed,
                _ => &mut self.time_input,
            },
            ActiveTab::Info => &mut self.info_input,
        }
    }
}
