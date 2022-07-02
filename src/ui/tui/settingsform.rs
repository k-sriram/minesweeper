use tui::style::{Modifier, Style};

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

mod numbox {
    use std::{cell::RefCell, iter::once};

    use crossterm::event::{Event, KeyCode, KeyEvent};
    use tui::{
        text::{Span, Spans, Text},
        widgets::Paragraph,
    };

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
                let mut style = &self.style.field;
                if let Active(ActiveState {
                    old_value: _,
                    cursor,
                }) = self.state
                {
                    if cursor == i {
                        style = &self.style.cursor;
                    }
                }
                digits.push(Span::styled(digit, *style));
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
            self.state = Inactive;
            self.reset_cache();
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
