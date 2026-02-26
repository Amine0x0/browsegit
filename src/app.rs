use crate::models::Repo;

#[derive(PartialEq)]
pub enum Mode {
    Normal,
    Renaming(String, String),
}

pub struct App {
    pub repos: Vec<Repo>,
    pub selected: usize,
    pub status: String,
    pub should_quit: bool,
    pub mode: Mode,
}

impl App {
    pub fn loading() -> Self {
        Self {
            repos: vec![],
            selected: 0,
            status: "Fetching repos…".to_string(),
            should_quit: false,
            mode: Mode::Normal,
        }
    }

    pub fn load_repos(&mut self, repos: Vec<Repo>) {
        self.status = if repos.is_empty() {
            "No repositories found".to_string()
        } else {
            "↑/↓ move, Enter clone, q quit".to_string()
        };
        self.repos = repos;
    }

    pub fn next(&mut self) {
        if self.repos.is_empty() {
            return;
        }
        self.selected = (self.selected + 1) % self.repos.len();
    }

    pub fn previous(&mut self) {
        if self.repos.is_empty() {
            return;
        }

        if self.selected == 0 {
            self.selected = self.repos.len() - 1;
        } else {
            self.selected -= 1;
        }
    }

    pub fn selected_repo(&self) -> Option<&Repo> {
        self.repos.get(self.selected)
    }
}
