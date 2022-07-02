use std::cell::RefCell;

use crossterm::event::{Event, KeyCode, KeyEvent};
use tui::{
    style::{Modifier, Style},
    text::{Span, Spans, Text},
    widgets::Paragraph,
};

use crate::{
    game::{Difficulty::*, Settings},
    CustomBoard,
};
use numbox::{Action as NBAction, NumBox};

#[derive(Debug, Clone, Copy)]
pub struct FieldStyle {
    pub label: Style,
    pub field: Style,
    pub selected: Style,
    pub cursor: Style,
}

const DEFAULT_FIELD_STYLE: FieldStyle = FieldStyle {
    label: Style {
        fg: None,
        bg: None,
        add_modifier: Modifier::empty(),
        sub_modifier: Modifier::empty(),
    },
    field: Style {
        fg: None,
        bg: None,
        add_modifier: Modifier::empty(),
        sub_modifier: Modifier::empty(),
    },
    selected: Style {
        fg: None,
        bg: None,
        add_modifier: Modifier::REVERSED,
        sub_modifier: Modifier::empty(),
    },
    cursor: Style {
        fg: None,
        bg: None,
        add_modifier: Modifier::RAPID_BLINK,
        sub_modifier: Modifier::empty(),
    },
};

#[derive(Debug)]
pub struct SettingsForm<'a> {
    settings: Settings,
    state: State,
    focus: Option<Settings>, // None when out of focus and stores the old value when infocus.
    width_input: NumBox<'a>,
    height_input: NumBox<'a>,
    mines_input: NumBox<'a>,
    style: FieldStyle,
    content_cache: RefCell<Option<Paragraph<'a>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum State {
    #[default]
    FocusEasy,
    FocusMedium,
    FocusHard,
    FocusCustom,
    FocusWidth,
    FocusHeight,
    FocusMines,
    EditHeight,
    EditWidth,
    EditMines,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SFAction {
    None,
    Inactive,
    Change(Settings),
    Set(Settings),
    Reset(Settings),
}

impl<'a> SettingsForm<'a> {
    pub fn new(init_settings: Settings, style: FieldStyle) -> Self {
        let mut width_input = NumBox::new(
            "Width  : ".to_string(),
            2,
            init_settings.difficulty.width(),
            style,
        );
        width_input.set_min_bound(Some(2));
        width_input.set_max_bound(Some(99));
        let mut height_input = NumBox::new(
            "Height : ".to_string(),
            2,
            init_settings.difficulty.width(),
            style,
        );
        height_input.set_min_bound(Some(2));
        height_input.set_max_bound(Some(99));
        let mut mines_input = NumBox::new(
            "Mines  : ".to_string(),
            3,
            init_settings.difficulty.mines(),
            style,
        );
        mines_input.set_min_bound(Some(1));
        Self {
            settings: init_settings,
            state: State::default(),
            focus: None,
            width_input,
            height_input,
            mines_input,
            style,
            content_cache: RefCell::new(None),
        }
    }

    pub fn render(&self) -> Paragraph<'a> {
        let content = self.content_cache.borrow();
        if content.is_some() {
            content.as_ref().unwrap().clone()
        } else {
            self.render_to_cache();
            self.content_cache.borrow().as_ref().unwrap().clone()
        }
    }

    pub fn focus(&mut self) {
        self.focus = Some(self.settings.clone());
        self.reset_cache();
    }

    pub fn unfocus(&mut self) {
        if let Some(ref old_settings) = self.focus {
            self.settings = old_settings.clone();
            match &self.state {
                State::EditWidth => {
                    self.width_input.unfocus();
                    self.state = State::FocusWidth;
                }
                State::EditHeight => {
                    self.height_input.unfocus();
                    self.state = State::FocusHeight;
                }
                State::EditMines => {
                    self.mines_input.unfocus();
                    self.state = State::FocusMines
                }
                _ => {}
            }
            self.width_input.set_value(self.settings.difficulty.width());
            self.height_input
                .set_value(self.settings.difficulty.height());
            self.mines_input.set_value(self.settings.difficulty.mines());
            self.reset_cache();
        }
    }

    pub fn handle_input(&mut self, event: Event) -> SFAction {
        use KeyCode::*;
        match self.focus {
            None => SFAction::Inactive,
            Some(ref old_settings) => {
                let (width, height, mut mines) = (
                    self.settings.difficulty.width(),
                    self.settings.difficulty.height(),
                    self.settings.difficulty.mines(),
                );
                match &self.state {
                    State::EditWidth => match self.width_input.handle_input(event) {
                        NBAction::None => SFAction::None,
                        NBAction::Inactive => unreachable!(),
                        NBAction::Change(width) => {
                            self.settings.difficulty =
                                Custom(CustomBoard::new(width, height, mines));
                            self.reset_cache();
                            SFAction::Change(self.settings.clone())
                        }
                        NBAction::Set(width) | NBAction::Reset(width) => {
                            if mines >= width * height {
                                mines = width * height - 1;
                                self.mines_input.set_value(mines);
                            }
                            self.settings.difficulty =
                                Custom(CustomBoard::new(width, height, mines));
                            self.state = State::FocusWidth;
                            self.reset_cache();
                            SFAction::Change(self.settings.clone())
                        }
                    },
                    State::EditHeight => match self.height_input.handle_input(event) {
                        NBAction::None => SFAction::None,
                        NBAction::Inactive => unreachable!(),
                        NBAction::Change(height) => {
                            self.settings.difficulty =
                                Custom(CustomBoard::new(width, height, mines));
                            self.reset_cache();
                            SFAction::Change(self.settings.clone())
                        }
                        NBAction::Set(height) | NBAction::Reset(height) => {
                            if mines >= width * height {
                                mines = width * height - 1;
                                self.mines_input.set_value(mines);
                            }
                            self.settings.difficulty =
                                Custom(CustomBoard::new(width, height, mines));
                            self.reset_cache();
                            self.state = State::FocusHeight;
                            SFAction::Change(self.settings.clone())
                        }
                    },
                    State::EditMines => match self.mines_input.handle_input(event) {
                        NBAction::None => SFAction::None,
                        NBAction::Inactive => unreachable!(),
                        NBAction::Change(mines) => {
                            self.settings.difficulty =
                                Custom(CustomBoard::new(width, height, mines));
                            self.reset_cache();
                            SFAction::Change(self.settings.clone())
                        }
                        NBAction::Set(mines) | NBAction::Reset(mines) => {
                            self.settings.difficulty =
                                Custom(CustomBoard::new(width, height, mines));
                            self.reset_cache();
                            self.state = State::FocusMines;
                            SFAction::Change(self.settings.clone())
                        }
                    },
                    _ => match event {
                        Event::Key(KeyEvent { code, .. }) => match code {
                            Up => {
                                self.state = match &self.state {
                                    State::FocusEasy => State::FocusMines,
                                    State::FocusMedium => State::FocusEasy,
                                    State::FocusHard => State::FocusMedium,
                                    State::FocusCustom => State::FocusHard,
                                    State::FocusWidth => State::FocusCustom,
                                    State::FocusHeight => State::FocusWidth,
                                    State::FocusMines => State::FocusHeight,
                                    State::EditHeight | State::EditWidth | State::EditMines => {
                                        unreachable!()
                                    }
                                };
                                self.reset_cache();
                                SFAction::None
                            }
                            Down => {
                                self.state = match &self.state {
                                    State::FocusEasy => State::FocusMedium,
                                    State::FocusMedium => State::FocusHard,
                                    State::FocusHard => State::FocusCustom,
                                    State::FocusCustom => State::FocusWidth,
                                    State::FocusWidth => State::FocusHeight,
                                    State::FocusHeight => State::FocusMines,
                                    State::FocusMines => State::FocusEasy,
                                    State::EditHeight | State::EditWidth | State::EditMines => {
                                        unreachable!()
                                    }
                                };
                                self.reset_cache();
                                SFAction::None
                            }
                            Char(' ') => match &self.state {
                                State::FocusEasy => {
                                    self.settings.difficulty = Easy;
                                    self.update_hwm();
                                    self.reset_cache();
                                    SFAction::Change(self.settings.clone())
                                }
                                State::FocusMedium => {
                                    self.settings.difficulty = Medium;
                                    self.update_hwm();
                                    self.reset_cache();
                                    SFAction::Change(self.settings.clone())
                                }
                                State::FocusHard => {
                                    self.settings.difficulty = Hard;
                                    self.update_hwm();
                                    self.reset_cache();
                                    SFAction::Change(self.settings.clone())
                                }
                                State::FocusCustom => {
                                    self.settings.difficulty =
                                        Custom(CustomBoard::new(width, height, mines));
                                    self.reset_cache();
                                    SFAction::Change(self.settings.clone())
                                }
                                State::FocusWidth | State::FocusHeight | State::FocusMines => {
                                    SFAction::None
                                }
                                State::EditHeight | State::EditWidth | State::EditMines => {
                                    unreachable!()
                                }
                            },
                            Enter => match &self.state {
                                State::FocusEasy
                                | State::FocusMedium
                                | State::FocusHard
                                | State::FocusCustom => SFAction::None,
                                State::FocusWidth => {
                                    self.state = State::EditWidth;
                                    self.width_input.focus();
                                    self.reset_cache();
                                    SFAction::None
                                }
                                State::FocusHeight => {
                                    self.state = State::EditHeight;
                                    self.height_input.focus();
                                    self.reset_cache();
                                    SFAction::None
                                }
                                State::FocusMines => {
                                    self.state = State::EditMines;
                                    self.mines_input.set_max_bound(Some(width * height - 1));
                                    self.mines_input.focus();
                                    self.reset_cache();
                                    SFAction::None
                                }
                                State::EditHeight | State::EditWidth | State::EditMines => {
                                    unreachable!()
                                }
                            },
                            Char('w') => {
                                self.focus = None;
                                self.reset_cache();
                                SFAction::Set(self.settings.clone())
                            }
                            Char('q') | Esc => {
                                self.settings = old_settings.clone();
                                self.focus = None;
                                self.update_hwm();
                                self.reset_cache();
                                SFAction::Reset(self.settings.clone())
                            }
                            _ => todo!(),
                        },
                        _ => SFAction::None,
                    },
                }
            }
        }
    }

    pub fn set(&mut self, settings: Settings) {
        self.settings = settings;
        self.width_input.set_value(self.settings.difficulty.width());
        self.height_input
            .set_value(self.settings.difficulty.height());
        self.mines_input.set_value(self.settings.difficulty.mines());
        self.reset_cache();
    }

    pub fn set_style(&mut self, style: FieldStyle) {
        self.style = style;
        self.reset_cache();
    }

    fn render_to_cache(&self) {
        let mut lines = Vec::new();
        lines.push(Spans(vec![Span::styled(
            "Difficulty".to_string(),
            self.style
                .label
                .patch(Style::default().add_modifier(Modifier::UNDERLINED)),
        )]));

        // Easy
        lines.push(render_radio_button(
            "Easy   ".to_string(),
            self.style,
            self.settings.difficulty == Easy,
            self.state == State::FocusEasy,
        ));

        // Medium
        lines.push(render_radio_button(
            "Medium  ".to_string(),
            self.style,
            self.settings.difficulty == Medium,
            self.state == State::FocusMedium,
        ));

        // Hard
        lines.push(render_radio_button(
            "Hard   ".to_string(),
            self.style,
            self.settings.difficulty == Hard,
            self.state == State::FocusHard,
        ));

        // Custom
        lines.push(render_radio_button(
            "Custom   ".to_string(),
            self.style,
            self.settings.difficulty != Easy
                && self.settings.difficulty != Medium
                && self.settings.difficulty != Hard,
            self.state == State::FocusCustom,
        ));

        //Width
        lines.push({
            let mut spans = self.width_input.render();
            if self.state == State::FocusWidth || self.state == State::EditWidth {
                patch_spans(&mut spans, self.style.selected)
            }
            spans
        });

        //Height
        lines.push({
            let mut spans = self.height_input.render();
            if self.state == State::FocusHeight || self.state == State::EditHeight {
                patch_spans(&mut spans, self.style.selected)
            }
            spans
        });

        //Mines
        lines.push({
            let mut spans = self.mines_input.render();
            if self.state == State::FocusMines || self.state == State::EditMines {
                patch_spans(&mut spans, self.style.selected)
            }
            spans
        });

        *(self.content_cache.borrow_mut()) = Some(Paragraph::new(Text { lines }));
    }

    fn reset_cache(&self) {
        *(self.content_cache.borrow_mut()) = None;
    }

    fn update_hwm(&mut self) {
        let (width, height, mines) = (
            self.settings.difficulty.width(),
            self.settings.difficulty.height(),
            self.settings.difficulty.mines(),
        );
        self.width_input.set_value(width);
        self.height_input.set_value(height);
        self.mines_input.set_value(mines);
    }
}

fn render_radio_button<'a>(
    label: String,
    style: FieldStyle,
    selected: bool,
    focussed: bool,
) -> Spans<'a> {
    let (label_style, field_style) = get_styles(style, focussed);
    Spans(vec![
        Span::styled(format!("{} (", label), label_style),
        Span::styled(
            if selected {
                "*".to_string()
            } else {
                " ".to_string()
            },
            field_style,
        ),
        Span::styled(")", label_style),
    ])
}

fn get_styles(style: FieldStyle, focussed: bool) -> (Style, Style) {
    let FieldStyle {
        mut label,
        mut field,
        selected,
        cursor: _,
    } = style;
    if focussed {
        label = label.patch(selected);
        field = field.patch(selected);
    }
    (label, field)
}

fn patch_spans(spans: &mut Spans, style: Style) {
    for span in spans.0.iter_mut() {
        span.style = span.style.patch(style);
    }
}

mod numbox {
    use std::{cell::RefCell, iter::once};

    use crossterm::event::{Event, KeyCode, KeyEvent};
    use tui::text::{Span, Spans};

    use super::FieldStyle;

    #[derive(Debug)]
    pub struct NumBox<'a> {
        label: String,
        digits: usize,
        value: usize,
        state: NumBoxState,
        style: FieldStyle,
        min_bound: Option<usize>,
        max_bound: Option<usize>,
        content_cache: RefCell<Option<Spans<'a>>>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Action {
        None,
        Inactive,
        Change(usize),
        Set(usize),
        Reset(usize),
    }

    #[derive(Debug, Clone, Copy)]
    enum NumBoxState {
        Inactive,
        Active(ActiveState),
    }

    use NumBoxState::*;

    #[derive(Debug, Clone, Copy)]
    struct ActiveState {
        old_value: usize,
        cursor: usize,
    }

    impl<'a> NumBox<'a> {
        pub fn new(label: String, digits: usize, value: usize, style: FieldStyle) -> Self {
            if digits < 1 {
                panic!("NumBox should have at least 1 digit.")
            }
            Self {
                label,
                digits,
                value,
                state: NumBoxState::Inactive,
                style,
                min_bound: None,
                max_bound: None,
                content_cache: RefCell::new(None),
            }
        }

        pub fn render(&self) -> Spans<'a> {
            let content = self.content_cache.borrow();
            if content.is_some() {
                content.as_ref().unwrap().clone()
            } else {
                self.render_to_cache();
                self.content_cache.borrow().as_ref().unwrap().clone()
            }
        }

        fn render_to_cache(&self) {
            let label = Span::styled(self.label.clone(), self.style.label);
            let mut digits = Vec::new();
            let mut place_value = 1;
            for i in 0..self.digits {
                let digit = ((self.value / place_value) % 10).to_string();
                let mut style = self.style.field;
                if let Active(ActiveState {
                    old_value: _,
                    cursor,
                }) = self.state
                {
                    if cursor == i {
                        style = style.patch(self.style.cursor);
                    }
                }
                digits.push(Span::styled(digit, style));
                place_value *= 10;
            }
            *(self.content_cache.borrow_mut()) =
                Some(Spans(once(label).chain(digits.into_iter().rev()).collect()));
        }

        fn reset_cache(&self) {
            *(self.content_cache.borrow_mut()) = None;
        }

        pub fn set_value(&mut self, value: usize) {
            self.value = value;
            self.reset_cache();
        }

        pub fn focus(&mut self) {
            self.state = Active(ActiveState {
                old_value: self.value,
                cursor: self.digits - 1,
            });
            self.reset_cache();
        }

        pub fn unfocus(&mut self) {
            if let Active(ActiveState {
                old_value,
                cursor: _,
            }) = self.state
            {
                self.value = old_value;
                self.state = Inactive;
                self.reset_cache();
            }
        }

        pub fn set_style(&mut self, style: FieldStyle) {
            self.style = style;
            self.reset_cache();
        }

        pub fn set_min_bound(&mut self, bound: Option<usize>) {
            self.min_bound = bound;
            if let Some(min) = self.min_bound {
                if self.value < min {
                    self.value = min;
                    self.reset_cache();
                }
            }
        }

        pub fn set_max_bound(&mut self, bound: Option<usize>) {
            self.max_bound = bound;
            if let Some(max) = self.max_bound {
                if self.value > max {
                    self.value = max;
                    self.reset_cache();
                }
            }
        }

        pub fn handle_input(&mut self, event: Event) -> Action {
            use KeyCode::*;
            match self.state {
                Active(ActiveState {
                    old_value,
                    ref mut cursor,
                }) => match event {
                    Event::Key(KeyEvent { code, .. }) => match code {
                        Up => {
                            self.value = (self.value + 10usize.pow(*cursor as u32))
                                % (10usize.pow(self.digits as u32));
                            self.reset_cache();
                            Action::Change(self.value)
                        }
                        Down => {
                            self.value = (self.value - 10usize.pow(*cursor as u32))
                                % (10usize.pow(self.digits as u32));
                            self.reset_cache();
                            Action::Change(self.value)
                        }
                        Left => {
                            *cursor = (*cursor + 1) % self.digits;
                            Action::None
                        }
                        Right => {
                            *cursor = if *cursor == 0 {
                                self.digits - 1
                            } else {
                                *cursor - 1
                            };
                            Action::None
                        }
                        Char(c) if c.is_ascii_digit() => {
                            let digit = c.to_digit(10).unwrap() as usize;
                            let place_value = 10usize.pow(*cursor as u32);
                            self.value -= (self.value / place_value) % 10;
                            self.value += digit * place_value;
                            *cursor = cursor.saturating_sub(1);
                            self.reset_cache();
                            Action::Change(self.value)
                        }
                        Enter => {
                            if let Some(max) = self.max_bound {
                                if self.value > max {
                                    self.value = max;
                                }
                            }
                            if let Some(min) = self.min_bound {
                                if self.value < min {
                                    self.value = min;
                                }
                            }
                            self.unfocus();
                            Action::Set(self.value)
                        }
                        Esc => {
                            self.value = old_value;
                            self.unfocus();
                            Action::Reset(self.value)
                        }
                        _ => Action::None,
                    },
                    _ => Action::None,
                },
                Inactive => Action::Inactive,
            }
        }
    }
}
