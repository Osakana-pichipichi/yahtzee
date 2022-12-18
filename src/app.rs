use crate::hand::Hand;
use crate::score_table::ScoreTable;
use crate::scoring::Boxes;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use enum_iterator::all;

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

#[derive(PartialEq, Eq)]
pub enum CursorPos {
    Role,
    Hand(usize),
    Dust(usize),
    Table(usize),
}

pub struct Play {
    pub player_id: usize,
    pub hand: Hand,
    pub is_held: [bool; 5],
    pub roll_count: u32,
}

impl Play {
    pub fn new(player_id: usize) -> Self {
        Self {
            player_id,
            hand: Hand::new_with_random_n_dice(0),
            is_held: [true; 5],
            roll_count: 0,
        }
    }
}

pub struct App {
    pub current_play: Play,
    pub cursor_pos: CursorPos,
    pub num_players: usize,
    pub scores: Vec<ScoreTable>,
}

impl App {
    pub fn new(num_players: usize) -> Self {
        Self {
            current_play: Play::new(0),
            cursor_pos: CursorPos::Role,
            num_players,
            scores: (0..num_players).map(|_| ScoreTable::new()).collect(),
        }
    }

    pub fn do_action(&mut self, key_event: KeyEvent) -> AppReturn {
        match key_event {
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => AppReturn::Exit,

            KeyEvent {
                code: KeyCode::Enter | KeyCode::Char(' '),
                ..
            } => {
                match self.cursor_pos {
                    CursorPos::Role => (),
                    CursorPos::Hand(pos) | CursorPos::Dust(pos) => {
                        self.current_play.is_held[pos] = !self.current_play.is_held[pos]
                    }
                    CursorPos::Table(..) => (),
                }
                AppReturn::Continue
            }

            KeyEvent {
                code: KeyCode::Left | KeyCode::Char('a'),
                ..
            } => {
                match self.cursor_pos {
                    CursorPos::Hand(pos) => {
                        if pos > 0 {
                            self.cursor_pos = CursorPos::Hand(pos - 1);
                        }
                    }
                    CursorPos::Dust(pos) => {
                        if pos > 0 {
                            self.cursor_pos = CursorPos::Dust(pos - 1);
                        }
                    }
                    CursorPos::Table(..) => {
                        self.cursor_pos = CursorPos::Hand(Hand::DICE_NUM - 1);
                    }
                    _ => (),
                }
                AppReturn::Continue
            }

            KeyEvent {
                code: KeyCode::Right | KeyCode::Char('d'),
                ..
            } => {
                match self.cursor_pos {
                    CursorPos::Role => {
                        self.cursor_pos = CursorPos::Table(0);
                    }
                    CursorPos::Hand(pos) => {
                        let new_pos = pos + 1;
                        if new_pos < Hand::DICE_NUM {
                            self.cursor_pos = CursorPos::Hand(new_pos);
                        } else {
                            self.cursor_pos = CursorPos::Table(0);
                        }
                    }
                    CursorPos::Dust(pos) => {
                        let new_pos = pos + 1;
                        if new_pos < Hand::DICE_NUM {
                            self.cursor_pos = CursorPos::Dust(new_pos);
                        } else {
                            self.cursor_pos = CursorPos::Table(0);
                        }
                    }
                    _ => (),
                }
                AppReturn::Continue
            }

            KeyEvent {
                code: KeyCode::Up | KeyCode::Char('w'),
                ..
            } => {
                match self.cursor_pos {
                    CursorPos::Hand(..) => {
                        self.cursor_pos = CursorPos::Role;
                    }
                    CursorPos::Dust(pos) => {
                        self.cursor_pos = CursorPos::Hand(pos);
                    }
                    CursorPos::Table(pos) => {
                        if pos > 0 {
                            self.cursor_pos = CursorPos::Table(pos - 1);
                        }
                    }
                    _ => (),
                }
                AppReturn::Continue
            }

            KeyEvent {
                code: KeyCode::Down | KeyCode::Char('s'),
                ..
            } => {
                match self.cursor_pos {
                    CursorPos::Role => {
                        self.cursor_pos = CursorPos::Hand(0);
                    }
                    CursorPos::Hand(pos) => {
                        self.cursor_pos = CursorPos::Dust(pos);
                    }
                    CursorPos::Table(pos) => {
                        let num_boxes = all::<Boxes>().count();
                        let new_pos = pos + 1;
                        if new_pos < num_boxes {
                            self.cursor_pos = CursorPos::Table(new_pos);
                        }
                    }
                    _ => (),
                }
                AppReturn::Continue
            }

            _ => AppReturn::Continue,
        }
    }
}
