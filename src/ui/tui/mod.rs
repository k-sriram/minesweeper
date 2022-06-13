use std::{
    io::{self, Stdout},
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use crate::{game::Difficulty, Action, Cell, Game, Settings, UI};

const TICK: Duration = Duration::from_millis(100);
const ACTIVE_BORDER: Style = Style {
    fg: Some(Color::White),
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

const INACTIVE_BORDER: Style = Style {
    fg: None,
    bg: None,
    add_modifier: Modifier::DIM,
    sub_modifier: Modifier::empty(),
};

pub struct TUI {
    state: State,
    status: String,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    cursor: GameCursor,
}

#[derive(Debug)]
enum State {
    Game,
    Settings(SettingsEditor),
}

#[derive(Debug)]
struct GameCursor {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

impl GameCursor {
    fn new(width: usize, height: usize) -> Self {
        GameCursor {
            x: width / 2,
            y: height / 2,
            width,
            height,
        }
    }

    fn move_(&mut self, x: isize, y: isize) {
        self.x = ((self.x as isize + x) % self.width as isize) as usize;
        self.y = ((self.y as isize + y) % self.height as isize) as usize;
    }
}

#[derive(Debug)]
struct SettingsEditor {
    settings: Settings,
    cursor: SettingsCursor,
}

impl SettingsEditor {
    fn new(settings: &Settings) -> Self {
        SettingsEditor {
            settings: settings.clone(),
            cursor: SettingsCursor::Normal(0),
        }
    }

    fn cursor_index(&self) -> usize {
        match self.cursor {
            SettingsCursor::Normal(i) => i,
            SettingsCursor::Difficulty(_) => 3,
        }
    }
}

#[derive(Debug)]
enum SettingsCursor {
    Normal(usize),
    Difficulty(EditDifficulty),
}

#[derive(Debug)]
struct EditDifficulty {
    old_difficulty: Difficulty,
    stage: EditDifficultyStage,
    width: usize,
    height: usize,
    mines: usize,
}

#[derive(Debug)]
enum EditDifficultyStage {
    Width,
    Height,
    Mines,
}

impl TUI {
    pub fn new(settings: &Settings) -> Self {
        let terminal = initialize_terminal().expect("Terminal initialization failed");
        Self {
            state: State::Game,
            status: String::default(),
            terminal,
            cursor: GameCursor::new(settings.difficulty.width(), settings.difficulty.height()),
        }
    }

    fn draw(&mut self, game: &Game) {
        let (mut board_border_style, mut settings_border_style) =
            (INACTIVE_BORDER, INACTIVE_BORDER);
        match self.state {
            State::Game => {
                board_border_style = ACTIVE_BORDER;
            }
            State::Settings(_) => {
                settings_border_style = ACTIVE_BORDER;
            }
        };

        let board = self.draw_board(game).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(board_border_style),
        );
        let settings = self.draw_settings(game).block(
            Block::default()
                .title("Settings")
                .borders(Borders::ALL)
                .border_style(settings_border_style),
        );

        self.terminal
            .draw(|f| {
                let area = f.size();
                if area.width < 25 || area.height < 14 {
                    f.render_widget(
                        Paragraph::new(Text::raw("Window too small\n Please resize.")),
                        area,
                    );
                    return;
                }
                let outline = Block::default()
                    .title("Minesweeper")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL);
                let inner_area = outline.inner(area);
                f.render_widget(outline, area);

                let (settings_area, board_area, status_area, keybar_area) = {
                    let bars = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Min(10),
                            Constraint::Length(1),
                            Constraint::Length(1),
                        ])
                        .split(inner_area);
                    let panes = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Length(18), Constraint::Min(5)])
                        .split(bars[0]);
                    (panes[0], panes[1], bars[1], bars[2])
                };
                f.render_widget(settings, settings_area);
                f.render_widget(board, board_area);
                f.render_widget(Paragraph::new(Text::raw(self.status.as_str())), status_area);
                f.render_widget(Paragraph::new(Text::raw("KeyBar: TODO")), keybar_area);
            })
            .unwrap();
    }

    fn draw_board<'a>(&self, game: &Game) -> Paragraph<'a> {
        let board = game.board();
        let mut lines = Vec::new();
        for row in board.iter() {
            let mut cells = Vec::new();
            for cell in row.iter() {
                let (text, style) = match cell {
                    Cell::Hidden => (" ", Style::default().bg(Color::Gray)),
                    Cell::Flag => ("►", Style::default().fg(Color::LightGreen).bg(Color::Gray)),
                    Cell::Mine => ("*", Style::default().fg(Color::Green).bg(Color::DarkGray)),
                    Cell::FalseFlag => (
                        "►",
                        Style::default()
                            .fg(Color::Red)
                            .bg(Color::Gray)
                            .add_modifier(Modifier::RAPID_BLINK),
                    ),
                    Cell::TrippedMine => (
                        "*",
                        Style::default()
                            .fg(Color::Red)
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::RAPID_BLINK),
                    ),
                    Cell::Open(0) => (" ", Style::default().bg(Color::DarkGray)),
                    Cell::Open(1) => ("1", Style::default().fg(Color::Blue).bg(Color::DarkGray)),
                    Cell::Open(2) => (
                        "2",
                        Style::default().fg(Color::LightGreen).bg(Color::DarkGray),
                    ),
                    Cell::Open(3) => (
                        "3",
                        Style::default().fg(Color::LightRed).bg(Color::DarkGray),
                    ),
                    Cell::Open(4) => (
                        "4",
                        Style::default().fg(Color::LightBlue).bg(Color::DarkGray),
                    ),
                    Cell::Open(5) => ("5", Style::default().fg(Color::Yellow).bg(Color::DarkGray)),
                    Cell::Open(6) => (
                        "6",
                        Style::default().fg(Color::LightMagenta).bg(Color::DarkGray),
                    ),
                    Cell::Open(7) => (
                        "7",
                        Style::default().fg(Color::LightCyan).bg(Color::DarkGray),
                    ),
                    Cell::Open(8) => ("8", Style::default().fg(Color::Gray).bg(Color::DarkGray)),
                    _ => unreachable!(),
                };
                cells.push(Span::styled(text, style));
            }
            lines.push(Spans(cells));
        }
        if let State::Game = self.state {
            let cursor = &self.cursor;
            let cursor_style = &mut lines[cursor.y].0[cursor.x].style;
            *cursor_style = (*cursor_style)
                .clone()
                .patch(Style::default().bg(Color::White));
        }
        Paragraph::new(Text { lines })
    }

    fn draw_settings<'a>(&self, game: &Game) -> Paragraph<'a> {
        let settings = match &self.state {
            State::Game => game.settings(),
            State::Settings(editor) => &editor.settings,
        };
        let mut lines = Vec::new();
        lines.push(Spans(vec![Span::styled(
            "Difficulty:",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )]));
        lines.push(Spans(vec![]));
        lines.push(Spans(vec![
            Span::styled(" Easy       [", Style::default()),
            Span::styled(
                if settings.difficulty == Difficulty::Easy {
                    "*"
                } else {
                    " "
                },
                Style::default().fg(Color::Red),
            ),
            Span::styled("]", Style::default()),
        ]));
        lines.push(Spans(vec![
            Span::styled(" Medium     [", Style::default()),
            Span::styled(
                if settings.difficulty == Difficulty::Medium {
                    "*"
                } else {
                    " "
                },
                Style::default().fg(Color::Red),
            ),
            Span::styled("]", Style::default()),
        ]));
        lines.push(Spans(vec![
            Span::styled(" Hard       [", Style::default()),
            Span::styled(
                if settings.difficulty == Difficulty::Hard {
                    "*"
                } else {
                    " "
                },
                Style::default().fg(Color::Red),
            ),
            Span::styled("]", Style::default()),
        ]));
        lines.push(Spans(vec![
            Span::styled(" Custom     [", Style::default()),
            Span::styled(
                if let Difficulty::Custom(_) = settings.difficulty {
                    "*"
                } else {
                    " "
                },
                Style::default().fg(Color::Red),
            ),
            Span::styled("]", Style::default()),
        ]));
        lines.push(Spans(vec![]));
        lines.push(Spans(vec![
            Span::styled(" Width      ", Style::default()),
            Span::styled(
                format!("{:0>3}", settings.difficulty.width()),
                Style::default().fg(Color::LightGreen),
            ),
        ]));
        lines.push(Spans(vec![
            Span::styled(" Height     ", Style::default()),
            Span::styled(
                format!("{:0>3}", settings.difficulty.height()),
                Style::default().fg(Color::LightGreen),
            ),
        ]));
        lines.push(Spans(vec![
            Span::styled(" Mines      ", Style::default()),
            Span::styled(
                format!("{:0>3}", settings.difficulty.mines()),
                Style::default().fg(Color::LightGreen),
            ),
        ]));

        if let State::Settings(editor) = &self.state {
            let i = editor.cursor_index() + 2;
            for i in lines[i].0.iter_mut() {
                (*i).style = (*i)
                    .clone()
                    .style
                    .patch(Style::default().bg(Color::White).fg(Color::Black));
            }
        }
        Paragraph::new(Text { lines })
    }

    fn handle_event(&mut self, event: Event, game: &Game) -> Option<Action> {
        use KeyCode::*;
        // Check global hotkeys
        if event
            == Event::Key(KeyEvent {
                code: Char('c'),
                modifiers: KeyModifiers::CONTROL,
            })
        {
            return Some(Action::Quit);
        }
        match &mut self.state {
            State::Game => match event {
                Event::Key(KeyEvent { code, .. }) => match code {
                    Char('q') => Some(Action::Quit),
                    Char('r') => Some(Action::Reset),
                    Up => {
                        self.cursor.move_(0, -1);
                        None
                    }
                    Down => {
                        self.cursor.move_(0, 1);
                        None
                    }
                    Left => {
                        self.cursor.move_(-1, 0);
                        None
                    }
                    Right => {
                        self.cursor.move_(1, 0);
                        None
                    }
                    // TODO: Handle Numpad
                    Char('f') => Some(Action::Flag(self.cursor.x, self.cursor.y)),
                    Char('g') | Char(' ') => Some(Action::Open(self.cursor.x, self.cursor.y)),
                    Char('s') => {
                        self.state = State::Settings(SettingsEditor::new(game.settings()));
                        None
                    }
                    _ => {
                        self.status = "Invalid key".to_string();
                        None
                    }
                },
                _ => None,
            },
            State::Settings(ref mut editor) => match editor.cursor {
                SettingsCursor::Normal(i) => match event {
                    Event::Key(KeyEvent { code, .. }) => match code {
                        Up => {
                            editor.cursor = SettingsCursor::Normal(i.saturating_sub(1));
                            None
                        }
                        Down => {
                            editor.cursor = SettingsCursor::Normal(std::cmp::min(3, i + 1));
                            None
                        }
                        Char('q') => {
                            self.state = State::Game;
                            None
                        }
                        Char('w') => {
                            let settings = editor.settings.clone();
                            self.state = State::Game;
                            if settings.difficulty != game.settings().difficulty {
                                self.cursor = GameCursor::new(
                                    settings.difficulty.width(),
                                    settings.difficulty.height(),
                                );
                            }
                            Some(Action::ChangeSettings(settings))
                        }
                        Char(' ') | Enter => {
                            editor.settings.difficulty = match editor.cursor_index() {
                                0 => Difficulty::Easy,
                                1 => Difficulty::Medium,
                                2 => Difficulty::Hard,
                                _ => todo!(),
                            };
                            None
                        }
                        _ => None,
                    },
                    _ => None,
                },
                SettingsCursor::Difficulty(_) => todo!(),
            },
        }
    }
}

impl Drop for TUI {
    fn drop(&mut self) {
        deinitialize_terminal(&mut self.terminal).expect("Error deinitializing terminal");
    }
}

impl UI for TUI {
    fn get_action(&mut self, game: &Game) -> Action {
        loop {
            self.draw(game);
            if event::poll(TICK).unwrap() {
                self.status = String::default();
                let event = event::read().unwrap();
                match self.handle_event(event, game) {
                    Some(action) => return action,
                    None => continue,
                }
            }
        }
    }

    fn show_msg(&mut self, msg: &str) {
        self.status = msg.to_string();
    }
}

fn initialize_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;
    Ok(terminal)
}

fn deinitialize_terminal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), io::Error> {
    flush_events()?;
    terminal.show_cursor()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn flush_events() -> Result<(), io::Error> {
    while event::poll(Duration::ZERO)? {
        event::read()?;
    }
    Ok(())
}
