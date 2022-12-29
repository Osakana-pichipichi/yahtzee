use crate::events::{Actions, InputEvent};
use crate::hand::Hand;
use crate::score_table::ScoreTable;
use crate::scoring::{scoring, Boxes};
use array_macro::array;
use enum_iterator::{all, Sequence};

#[derive(PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

#[derive(PartialEq, Eq)]
pub enum CursorPos {
    Role,
    Hand(usize),
    Dust(usize),
    Table(Boxes),
    Disappear,
}

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
    Play(Play),
    Result,
}

pub struct App {
    pub state: AppState,
    pub cursor_pos: CursorPos,
    pub num_players: usize,
    pub scores: Vec<ScoreTable>,
}

impl App {
    const MAX_ROLL_COUNT: usize = 3;

    pub fn new(num_players: usize) -> Self {
        Self {
            state: AppState::Play(Play::new(0)),
            cursor_pos: CursorPos::Role,
            num_players,
            scores: (0..num_players).map(|_| ScoreTable::new()).collect(),
        }
    }

    pub fn do_action(&mut self, input_event: InputEvent) -> AppReturn {
        match self.state {
            AppState::Play(..) => self.do_action_in_play(input_event),
            AppState::Result => self.do_action_in_result(input_event),
        }
    }

    fn do_action_in_play(&mut self, input_event: InputEvent) -> AppReturn {
        let play = if let AppState::Play(play) = &self.state {
            play
        } else {
            panic!()
        };
        match play.game_phase {
            GamePhase::Init => self.do_action_in_init(input_event),
            GamePhase::Roll(..) => self.do_action_in_roll(input_event),
            GamePhase::SelectOrReroll(..) => self.do_action_in_select_or_reroll(input_event),
            GamePhase::Select => self.do_action_in_select(input_event),
        }
    }

    fn do_action_in_init(&mut self, input_event: InputEvent) -> AppReturn {
        let play = if let AppState::Play(play) = &mut self.state {
            play
        } else {
            panic!()
        };

        match input_event.action() {
            Actions::Exit => AppReturn::Exit,

            Actions::Select => {
                play.hand = Hand::new_with_random_n_dice(Hand::DICE_NUM);
                play.game_phase = GamePhase::Roll(0);
                self.cursor_pos = CursorPos::Role;
                AppReturn::Continue
            }

            _ => AppReturn::Continue,
        }
    }

    fn do_action_in_roll(&mut self, input_event: InputEvent) -> AppReturn {
        let play = if let AppState::Play(play) = &mut self.state {
            play
        } else {
            panic!()
        };

        match input_event.action() {
            Actions::Exit => AppReturn::Exit,

            Actions::Select => {
                let count = if let GamePhase::Roll(count) = play.game_phase {
                    count + 1
                } else {
                    panic!("Unexpected status!")
                };

                play.is_held = [true; Hand::DICE_NUM];
                if count < App::MAX_ROLL_COUNT {
                    play.game_phase = GamePhase::SelectOrReroll(count);
                    self.cursor_pos = CursorPos::Hand(0);
                } else {
                    play.game_phase = GamePhase::Select;
                    for pos in all::<Boxes>() {
                        if !self.scores[play.player_id].has_score_in(pos) {
                            self.cursor_pos = CursorPos::Table(pos);
                            break;
                        }
                    }
                }

                AppReturn::Continue
            }

            _ => {
                let dice = play.hand.get_dice();
                let removed_dice = dice
                    .iter()
                    .zip(play.is_held.iter())
                    .filter(|(.., &is_heled)| !is_heled)
                    .map(|(&d, ..)| d)
                    .collect::<Vec<_>>();
                play.hand.remove_dice(&removed_dice);
                play.hand
                    .add_dice(&Hand::new_with_random_n_dice(removed_dice.len()));
                AppReturn::Continue
            }
        }
    }

    fn do_action_in_select_or_reroll(&mut self, input_event: InputEvent) -> AppReturn {
        let play = if let AppState::Play(play) = &mut self.state {
            play
        } else {
            panic!()
        };

        match input_event.action() {
            Actions::Exit => AppReturn::Exit,

            Actions::Select => {
                match self.cursor_pos {
                    CursorPos::Role => {
                        let dice = play.hand.get_dice();
                        if !play.is_held.iter().all(|&x| x) {
                            let removed_dice = dice
                                .iter()
                                .zip(play.is_held.iter())
                                .filter(|(.., &is_heled)| !is_heled)
                                .map(|(&d, ..)| d)
                                .collect::<Vec<_>>();
                            play.hand.remove_dice(&removed_dice);
                            let rests_len = Hand::DICE_NUM - removed_dice.len();
                            play.is_held = array![i => i < rests_len; Hand::DICE_NUM];
                            play.hand
                                .add_dice(&Hand::new_with_random_n_dice(removed_dice.len()));
                            play.game_phase =
                                if let GamePhase::SelectOrReroll(count) = play.game_phase {
                                    GamePhase::Roll(count)
                                } else {
                                    panic!("Unexpected status!")
                                };
                        }
                    }
                    CursorPos::Hand(pos) | CursorPos::Dust(pos) => {
                        play.is_held[pos] = !play.is_held[pos]
                    }
                    CursorPos::Table(pos) => {
                        let pid = play.player_id;
                        let score_table = &mut self.scores[pid];
                        if !score_table.has_score_in(pos) {
                            let dice = play.hand.get_dice();
                            score_table.confirm_score(pos, scoring(pos, dice));
                            let new_pid = (pid + 1) % self.num_players;
                            *play = Play::new(new_pid);
                            if !self.scores[new_pid].has_all_scores() {
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

            Actions::Left => {
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

            Actions::Right => {
                match self.cursor_pos {
                    CursorPos::Role => {
                        for pos in all::<Boxes>() {
                            if !self.scores[play.player_id].has_score_in(pos) {
                                self.cursor_pos = CursorPos::Table(pos);
                                break;
                            }
                        }
                    }
                    CursorPos::Hand(pos) => {
                        let new_pos = pos + 1;
                        if new_pos < Hand::DICE_NUM {
                            self.cursor_pos = CursorPos::Hand(new_pos);
                        } else {
                            for pos in all::<Boxes>() {
                                if !self.scores[play.player_id].has_score_in(pos) {
                                    self.cursor_pos = CursorPos::Table(pos);
                                    break;
                                }
                            }
                        }
                    }
                    CursorPos::Dust(pos) => {
                        let new_pos = pos + 1;
                        if new_pos < Hand::DICE_NUM {
                            self.cursor_pos = CursorPos::Dust(new_pos);
                        } else {
                            for pos in all::<Boxes>() {
                                if !self.scores[play.player_id].has_score_in(pos) {
                                    self.cursor_pos = CursorPos::Table(pos);
                                    break;
                                }
                            }
                        }
                    }
                    _ => (),
                }
                AppReturn::Continue
            }

            Actions::Up => {
                match self.cursor_pos {
                    CursorPos::Hand(..) => {
                        if !play.is_held.iter().all(|&x| x) {
                            self.cursor_pos = CursorPos::Role;
                        }
                    }
                    CursorPos::Dust(pos) => {
                        self.cursor_pos = CursorPos::Hand(pos);
                    }
                    CursorPos::Table(pos) => {
                        let mut pos = pos;
                        while let Some(prev) = pos.previous() {
                            if !self.scores[play.player_id].has_score_in(prev) {
                                self.cursor_pos = CursorPos::Table(prev);
                                break;
                            }
                            pos = prev;
                        }
                    }
                    _ => (),
                }
                AppReturn::Continue
            }

            Actions::Down => {
                match self.cursor_pos {
                    CursorPos::Role => {
                        self.cursor_pos = CursorPos::Hand(0);
                    }
                    CursorPos::Hand(pos) => {
                        self.cursor_pos = CursorPos::Dust(pos);
                    }
                    CursorPos::Table(pos) => {
                        let mut pos = pos;
                        while let Some(next) = pos.next() {
                            if !self.scores[play.player_id].has_score_in(next) {
                                self.cursor_pos = CursorPos::Table(next);
                                break;
                            }
                            pos = next;
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
        let play = if let AppState::Play(play) = &mut self.state {
            play
        } else {
            panic!()
        };

        match input_event.action() {
            Actions::Exit => AppReturn::Exit,

            Actions::Select => {
                if let CursorPos::Table(pos) = self.cursor_pos {
                    let pid = play.player_id;
                    let score_table = &mut self.scores[pid];
                    if !score_table.has_score_in(pos) {
                        let dice = play.hand.get_dice();
                        score_table.confirm_score(pos, scoring(pos, dice));
                        let new_pid = (pid + 1) % self.num_players;
                        *play = Play::new(new_pid);
                        if !self.scores[new_pid].has_all_scores() {
                            self.cursor_pos = CursorPos::Role;
                        } else {
                            self.state = AppState::Result;
                            self.cursor_pos = CursorPos::Disappear;
                        }
                    }
                }
                AppReturn::Continue
            }

            Actions::Up => {
                if let CursorPos::Table(pos) = self.cursor_pos {
                    let mut pos = pos;
                    while let Some(prev) = pos.previous() {
                        if !self.scores[play.player_id].has_score_in(prev) {
                            self.cursor_pos = CursorPos::Table(prev);
                            break;
                        }
                        pos = prev;
                    }
                }
                AppReturn::Continue
            }

            Actions::Down => {
                match self.cursor_pos {
                    CursorPos::Role => {
                        self.cursor_pos = CursorPos::Hand(0);
                    }
                    CursorPos::Hand(pos) => {
                        self.cursor_pos = CursorPos::Dust(pos);
                    }
                    CursorPos::Table(pos) => {
                        let mut pos = pos;
                        while let Some(next) = pos.next() {
                            if !self.scores[play.player_id].has_score_in(next) {
                                self.cursor_pos = CursorPos::Table(next);
                                break;
                            }
                            pos = next;
                        }
                    }
                    _ => (),
                }
                AppReturn::Continue
            }

            _ => AppReturn::Continue,
        }
    }

    fn do_action_in_result(&mut self, input_event: InputEvent) -> AppReturn {
        match input_event.action() {
            Actions::Exit => AppReturn::Exit,

            Actions::Select => AppReturn::Exit,

            _ => AppReturn::Continue,
        }
    }
}
