use crate::hand::Hand;
use crate::score_table::ScoreTable;
use crate::scoring::{scoring, Boxes};
use crate::InputEvent;
use array_macro::array;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use enum_iterator::Sequence;

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

pub enum CursorPos {
    Role,
    Hand(usize),
    Dust(usize),
    Table(Boxes),
    Disappear,
}

#[derive(Debug)]
pub enum GamePhase {
    Init,
    Roll(usize),
    SelectOrReroll(usize),
    Select,
}

pub struct Play {
    pub player_id: usize,
    pub hand: Hand,
    pub is_held: [bool; Hand::DICE_NUM],
    pub game_phase: GamePhase,
}

impl Play {
    pub fn new(player_id: usize) -> Self {
        Self {
            player_id,
            hand: Hand::new_with_random_n_dice(0),
            is_held: [false; 5],
            game_phase: GamePhase::Init,
        }
    }
}

pub enum AppState {
    Play,
    Result,
}

pub struct App {
    pub state: AppState,
    pub current_play: Play,
    pub cursor_pos: CursorPos,
    pub num_players: usize,
    pub scores: Vec<ScoreTable>,
}

impl App {
    const MAX_ROLL_COUNT: usize = 3;

    pub fn new(num_players: usize) -> Self {
        Self {
            state: AppState::Play,
            current_play: Play::new(0),
            cursor_pos: CursorPos::Role,
            num_players,
            scores: (0..num_players).map(|_| ScoreTable::new()).collect(),
        }
    }

    pub fn do_action(&mut self, input_event: InputEvent) -> AppReturn {
        match self.current_play.game_phase {
            GamePhase::Init => self.do_action_in_init(input_event),
            GamePhase::Roll(..) => self.do_action_in_roll(input_event),
            GamePhase::SelectOrReroll(..) => self.do_action_in_select_or_reroll(input_event),
            GamePhase::Select => self.do_action_in_select(input_event),
        }
    }

    fn do_action_in_init(&mut self, input_event: InputEvent) -> AppReturn {
        match input_event {
            InputEvent::Input(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => AppReturn::Exit,

            InputEvent::Input(KeyEvent {
                code: KeyCode::Enter | KeyCode::Char(' '),
                ..
            }) => {
                self.current_play.hand = Hand::new_with_random_n_dice(Hand::DICE_NUM);
                self.current_play.game_phase = GamePhase::Roll(0);
                self.cursor_pos = CursorPos::Role;
                AppReturn::Continue
            }

            _ => AppReturn::Continue,
        }
    }

    fn do_action_in_roll(&mut self, input_event: InputEvent) -> AppReturn {
        match input_event {
            InputEvent::Input(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => AppReturn::Exit,

            InputEvent::Input(KeyEvent {
                code: KeyCode::Enter | KeyCode::Char(' '),
                ..
            }) => {
                let count = if let GamePhase::Roll(count) = self.current_play.game_phase {
                    count + 1
                } else {
                    panic!("Unexpected status!")
                };

                self.current_play.is_held = [true; Hand::DICE_NUM];
                if count < App::MAX_ROLL_COUNT {
                    self.current_play.game_phase = GamePhase::SelectOrReroll(count);
                    self.cursor_pos = CursorPos::Hand(0);
                } else {
                    self.current_play.game_phase = GamePhase::Select;
                    self.cursor_pos = CursorPos::Table(Boxes::Aces);
                }

                AppReturn::Continue
            }

            _ => {
                let dice = self.current_play.hand.get_dice();
                let removed_dice = dice
                    .iter()
                    .zip(self.current_play.is_held.iter())
                    .filter(|(.., &is_heled)| !is_heled)
                    .map(|(&d, ..)| d)
                    .collect::<Vec<_>>();
                self.current_play.hand.remove_dice(&removed_dice);
                self.current_play
                    .hand
                    .add_dice(&Hand::new_with_random_n_dice(removed_dice.len()));
                AppReturn::Continue
            }
        }
    }

    fn do_action_in_select_or_reroll(&mut self, input_event: InputEvent) -> AppReturn {
        match input_event {
            InputEvent::Input(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => AppReturn::Exit,

            InputEvent::Input(KeyEvent {
                code: KeyCode::Enter | KeyCode::Char(' '),
                ..
            }) => {
                match self.cursor_pos {
                    CursorPos::Role => {
                        let dice = self.current_play.hand.get_dice();
                        let removed_dice = dice
                            .iter()
                            .zip(self.current_play.is_held.iter())
                            .filter(|(.., &is_heled)| !is_heled)
                            .map(|(&d, ..)| d)
                            .collect::<Vec<_>>();
                        self.current_play.hand.remove_dice(&removed_dice);
                        self.current_play.is_held =
                            array![i => i < Hand::DICE_NUM - removed_dice.len(); Hand::DICE_NUM];
                        self.current_play
                            .hand
                            .add_dice(&Hand::new_with_random_n_dice(removed_dice.len()));
                        self.current_play.game_phase = if let GamePhase::SelectOrReroll(count) =
                            self.current_play.game_phase
                        {
                            GamePhase::Roll(count)
                        } else {
                            panic!("Unexpected status!")
                        };
                    }
                    CursorPos::Hand(pos) | CursorPos::Dust(pos) => {
                        self.current_play.is_held[pos] = !self.current_play.is_held[pos]
                    }
                    CursorPos::Table(pos) => {
                        let pid = self.current_play.player_id;
                        let score_table = &mut self.scores[pid];
                        if !score_table.is_filled(pos) {
                            let dice = self.current_play.hand.get_dice();
                            score_table.confirm_score(pos, scoring(pos, dice));
                            let new_pid = (pid + 1) % self.num_players;
                            self.current_play = Play::new(new_pid);
                            if !self.scores[new_pid].are_all_cells_filled() {
                                self.cursor_pos = CursorPos::Role;
                            } else {
                                self.state = AppState::Result;
                                self.cursor_pos = CursorPos::Disappear;
                            }
                        }
                    }
                    _ => (),
                }
                AppReturn::Continue
            }

            InputEvent::Input(KeyEvent {
                code: KeyCode::Left | KeyCode::Char('a'),
                ..
            }) => {
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

            InputEvent::Input(KeyEvent {
                code: KeyCode::Right | KeyCode::Char('d'),
                ..
            }) => {
                match self.cursor_pos {
                    CursorPos::Role => {
                        self.cursor_pos = CursorPos::Table(Boxes::Aces);
                    }
                    CursorPos::Hand(pos) => {
                        let new_pos = pos + 1;
                        if new_pos < Hand::DICE_NUM {
                            self.cursor_pos = CursorPos::Hand(new_pos);
                        } else {
                            self.cursor_pos = CursorPos::Table(Boxes::Aces);
                        }
                    }
                    CursorPos::Dust(pos) => {
                        let new_pos = pos + 1;
                        if new_pos < Hand::DICE_NUM {
                            self.cursor_pos = CursorPos::Dust(new_pos);
                        } else {
                            self.cursor_pos = CursorPos::Table(Boxes::Aces);
                        }
                    }
                    _ => (),
                }
                AppReturn::Continue
            }

            InputEvent::Input(KeyEvent {
                code: KeyCode::Up | KeyCode::Char('w'),
                ..
            }) => {
                match self.cursor_pos {
                    CursorPos::Hand(..) => {
                        self.cursor_pos = CursorPos::Role;
                    }
                    CursorPos::Dust(pos) => {
                        self.cursor_pos = CursorPos::Hand(pos);
                    }
                    CursorPos::Table(pos) => {
                        if pos != Boxes::first().unwrap() {
                            self.cursor_pos = CursorPos::Table(pos.previous().unwrap());
                        }
                    }
                    _ => (),
                }
                AppReturn::Continue
            }

            InputEvent::Input(KeyEvent {
                code: KeyCode::Down | KeyCode::Char('s'),
                ..
            }) => {
                match self.cursor_pos {
                    CursorPos::Role => {
                        self.cursor_pos = CursorPos::Hand(0);
                    }
                    CursorPos::Hand(pos) => {
                        self.cursor_pos = CursorPos::Dust(pos);
                    }
                    CursorPos::Table(pos) => {
                        if pos != Boxes::last().unwrap() {
                            self.cursor_pos = CursorPos::Table(pos.next().unwrap());
                        }
                    }
                    _ => (),
                }
                AppReturn::Continue
            }

            _ => AppReturn::Continue,
        }
    }

    fn do_action_in_select(&mut self, input_event: InputEvent) -> AppReturn {
        match input_event {
            InputEvent::Input(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => AppReturn::Exit,

            InputEvent::Input(KeyEvent {
                code: KeyCode::Enter | KeyCode::Char(' '),
                ..
            }) => {
                match self.cursor_pos {
                    CursorPos::Role => {
                        let dice = self.current_play.hand.get_dice();
                        let removed_dice = dice
                            .iter()
                            .zip(self.current_play.is_held.iter())
                            .filter(|(.., &is_heled)| !is_heled)
                            .map(|(&d, ..)| d)
                            .collect::<Vec<_>>();
                        self.current_play.hand.remove_dice(&removed_dice);
                        self.current_play.is_held =
                            array![i => i < removed_dice.len(); Hand::DICE_NUM];
                        self.current_play
                            .hand
                            .add_dice(&Hand::new_with_random_n_dice(removed_dice.len()));
                    }
                    CursorPos::Hand(pos) | CursorPos::Dust(pos) => {
                        self.current_play.is_held[pos] = !self.current_play.is_held[pos]
                    }
                    CursorPos::Table(pos) => {
                        let pid = self.current_play.player_id;
                        let score_table = &mut self.scores[pid];
                        if !score_table.is_filled(pos) {
                            let dice = self.current_play.hand.get_dice();
                            score_table.confirm_score(pos, scoring(pos, dice));
                            let new_pid = (pid + 1) % self.num_players;
                            self.current_play = Play::new(new_pid);
                            if !self.scores[new_pid].are_all_cells_filled() {
                                self.cursor_pos = CursorPos::Role;
                            } else {
                                self.state = AppState::Result;
                                self.cursor_pos = CursorPos::Disappear;
                            }
                        }
                    }
                    _ => (),
                }
                AppReturn::Continue
            }

            InputEvent::Input(KeyEvent {
                code: KeyCode::Up | KeyCode::Char('w'),
                ..
            }) => {
                match self.cursor_pos {
                    CursorPos::Hand(..) => {
                        self.cursor_pos = CursorPos::Role;
                    }
                    CursorPos::Dust(pos) => {
                        self.cursor_pos = CursorPos::Hand(pos);
                    }
                    CursorPos::Table(pos) => {
                        if pos != Boxes::first().unwrap() {
                            self.cursor_pos = CursorPos::Table(pos.previous().unwrap());
                        }
                    }
                    _ => (),
                }
                AppReturn::Continue
            }

            InputEvent::Input(KeyEvent {
                code: KeyCode::Down | KeyCode::Char('s'),
                ..
            }) => {
                match self.cursor_pos {
                    CursorPos::Role => {
                        self.cursor_pos = CursorPos::Hand(0);
                    }
                    CursorPos::Hand(pos) => {
                        self.cursor_pos = CursorPos::Dust(pos);
                    }
                    CursorPos::Table(pos) => {
                        if pos != Boxes::last().unwrap() {
                            self.cursor_pos = CursorPos::Table(pos.next().unwrap());
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
