use crate::events::{Actions, InputEvent};
use crate::hand::Hand;
use crate::score_table::ScoreTable;
use crate::scoring::{scoring, Boxes};
use anyhow::{anyhow, Result};
use array_macro::array;
use enum_iterator::{all, Sequence};
use thiserror::Error;

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
            hand: Hand::new_with_random_n_dice(Hand::DICE_NUM),
            is_held: [false; 5],
            game_phase: GamePhase::Roll(0),
        }
    }

    pub fn reroll_dice(&mut self) -> usize {
        let dice = self.hand.get_dice();
        let removed_dice = dice
            .iter()
            .zip(self.is_held.iter())
            .filter(|(.., &is_heled)| !is_heled)
            .map(|(&d, ..)| d)
            .collect::<Vec<_>>();
        self.hand.remove_dice(&removed_dice);

        let reroll_dices = removed_dice.len();
        let rests = Hand::DICE_NUM - reroll_dices;
        self.is_held = array![i => i < rests; Hand::DICE_NUM];
        self.hand
            .add_dice(&Hand::new_with_random_n_dice(reroll_dices));

        reroll_dices
    }
}

pub enum AppState {
    Play(Option<Play>),
    Result,
}

#[derive(Debug, Error)]
pub enum AppStateError {
    #[error("Unexpected AppState")]
    UnexpectedState,
    #[error("Play data does not exist")]
    NoPlayData,
    #[error("Play data has already existed")]
    ExistPlayData,
}

pub struct GameData {
    num_players: usize,
    scores: Vec<ScoreTable>,
}

impl GameData {
    pub fn new(num_players: usize) -> Self {
        Self {
            num_players,
            scores: (0..num_players).map(|_| ScoreTable::new()).collect(),
        }
    }

    pub fn get_score_table(&self, player_id: usize) -> &ScoreTable {
        if player_id >= self.num_players {
            panic!(
                "Unexpected player_id: {} (total players: {})",
                player_id, self.num_players
            );
        };

        &self.scores[player_id]
    }

    pub fn get_mut_score_table(&mut self, player_id: usize) -> &mut ScoreTable {
        if player_id >= self.num_players {
            panic!(
                "Unexpected player_id: {} (total players: {})",
                player_id, self.num_players
            );
        };

        &mut self.scores[player_id]
    }

    pub fn get_num_players(&self) -> usize {
        self.num_players
    }

    pub fn current_player_id(&self) -> usize {
        let pid_to_filled_scores = self
            .scores
            .iter()
            .map(|e| e.get_num_filled_scores())
            .collect::<Vec<_>>();
        (1..self.get_num_players())
            .find(|&i| pid_to_filled_scores[i - 1] > pid_to_filled_scores[i])
            .unwrap_or(0)
    }
}

pub struct App {
    pub state: AppState,
    pub cursor_pos: CursorPos,
    game_data: Option<GameData>,
}

impl App {
    const MAX_ROLL_COUNT: usize = 3;

    pub fn new(num_players: usize) -> Self {
        Self {
            state: AppState::Play(None),
            cursor_pos: CursorPos::Role,
            game_data: Some(GameData::new(num_players)),
        }
    }

    /* helper functions */
    pub fn get_game_data(&self) -> &GameData {
        self.game_data.as_ref().unwrap()
    }

    fn get_mut_game_data(&mut self) -> &mut GameData {
        self.game_data.as_mut().unwrap()
    }

    pub fn get_play_data(&self) -> Result<&Play> {
        match &self.state {
            AppState::Play(p) => p.as_ref().ok_or_else(|| anyhow!(AppStateError::NoPlayData)),
            _ => Err(anyhow!(AppStateError::UnexpectedState)),
        }
    }

    fn get_mut_play_data(&mut self) -> Result<&mut Play> {
        match &mut self.state {
            AppState::Play(p) => p.as_mut().ok_or_else(|| anyhow!(AppStateError::NoPlayData)),
            _ => Err(anyhow!(AppStateError::UnexpectedState)),
        }
    }

    fn initialize_play_data(&mut self, player_id: usize) -> Result<&Play> {
        match &self.state {
            AppState::Play(p) => {
                if p.is_none() {
                    self.state = AppState::Play(Some(Play::new(player_id)));
                    self.get_play_data()
                } else {
                    Err(anyhow!(AppStateError::ExistPlayData))
                }
            }
            _ => Err(anyhow!(AppStateError::UnexpectedState)),
        }
    }

    fn cleanup_play_data(&mut self) -> Result<()> {
        match &self.state {
            AppState::Play(..) => {
                self.state = AppState::Play(None);
                Ok(())
            }
            _ => Err(anyhow!(AppStateError::UnexpectedState)),
        }
    }

    /* action handlers */
    pub fn do_action(&mut self, input_event: InputEvent) -> AppReturn {
        match self.state {
            AppState::Play(..) => self.do_action_in_play(input_event),
            AppState::Result => self.do_action_in_result(input_event),
        }
    }

    fn do_action_in_play(&mut self, input_event: InputEvent) -> AppReturn {
        match self.get_play_data() {
            Ok(play) => match play.game_phase {
                GamePhase::Roll(..) => self.do_action_in_roll(input_event),
                GamePhase::SelectOrReroll(..) => self.do_action_in_select_or_reroll(input_event),
                GamePhase::Select => self.do_action_in_select(input_event),
            },
            Err(e) => match e.downcast_ref::<AppStateError>().unwrap() {
                AppStateError::NoPlayData => self.do_action_in_init(input_event),
                _ => panic!("{}", e),
            },
        }
    }

    fn do_action_in_init(&mut self, input_event: InputEvent) -> AppReturn {
        match input_event.action() {
            Actions::Exit => AppReturn::Exit,

            Actions::Select => {
                let pid = self.get_game_data().current_player_id();
                self.initialize_play_data(pid).unwrap();
                AppReturn::Continue
            }

            _ => AppReturn::Continue,
        }
    }

    fn move_cursor_pos_to_table(&mut self) {
        let pid = self.get_mut_play_data().unwrap().player_id;
        for pos in all::<Boxes>() {
            if !self.get_game_data().get_score_table(pid).has_score_in(pos) {
                self.cursor_pos = CursorPos::Table(pos);
                break;
            }
        }
    }

    fn do_action_in_roll(&mut self, input_event: InputEvent) -> AppReturn {
        match input_event.action() {
            Actions::Exit => AppReturn::Exit,

            Actions::Select => {
                let play = self.get_mut_play_data().unwrap();
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
                    self.move_cursor_pos_to_table();
                }

                AppReturn::Continue
            }

            _ => {
                self.get_mut_play_data().unwrap().reroll_dice();
                AppReturn::Continue
            }
        }
    }

    fn up_action_in_score_table(&mut self) {
        if let CursorPos::Table(pos) = self.cursor_pos {
            let mut pos = pos;
            while let Some(prev) = pos.previous() {
                let pid = self.get_play_data().unwrap().player_id;
                if !self.get_game_data().get_score_table(pid).has_score_in(prev) {
                    self.cursor_pos = CursorPos::Table(prev);
                    break;
                }
                pos = prev;
            }
        } else {
            panic!("Unexpected curson position");
        }
    }

    fn down_action_in_score_table(&mut self) {
        if let CursorPos::Table(pos) = self.cursor_pos {
            let mut pos = pos;
            while let Some(next) = pos.next() {
                let pid = self.get_play_data().unwrap().player_id;
                if !self.get_game_data().get_score_table(pid).has_score_in(next) {
                    self.cursor_pos = CursorPos::Table(next);
                    break;
                }
                pos = next;
            }
        } else {
            panic!("Unexpected curson position");
        }
    }

    fn confirm_score_action(&mut self) {
        if let CursorPos::Table(pos) = self.cursor_pos {
            let play = self.get_play_data().unwrap();
            let pid = play.player_id;
            if !self.get_game_data().get_score_table(pid).has_score_in(pos) {
                let dice = play.hand.get_dice().to_vec();
                let score_table = self.get_mut_game_data().get_mut_score_table(pid);
                score_table.confirm_score(pos, scoring(pos, &dice));
                let next_pid = (pid + 1) % self.get_game_data().get_num_players();
                self.cleanup_play_data().unwrap();

                let next_score_table = self.get_game_data().get_score_table(next_pid);
                if !next_score_table.has_all_scores() {
                    self.cursor_pos = CursorPos::Role;
                } else {
                    self.state = AppState::Result;
                    self.cursor_pos = CursorPos::Disappear;
                }
            }
        } else {
            panic!("Unexpected curson position");
        }
    }

    fn do_action_in_select_or_reroll(&mut self, input_event: InputEvent) -> AppReturn {
        match input_event.action() {
            Actions::Exit => AppReturn::Exit,

            Actions::Select => {
                match self.cursor_pos {
                    CursorPos::Role => {
                        let play = self.get_mut_play_data().unwrap();
                        if !play.is_held.iter().all(|&x| x) {
                            play.reroll_dice();
                            play.game_phase =
                                if let GamePhase::SelectOrReroll(count) = play.game_phase {
                                    GamePhase::Roll(count)
                                } else {
                                    panic!("Unexpected status!")
                                };
                        }
                    }
                    CursorPos::Hand(pos) | CursorPos::Dust(pos) => {
                        let play = self.get_mut_play_data().unwrap();
                        play.is_held[pos] = !play.is_held[pos]
                    }
                    CursorPos::Table(..) => {
                        self.confirm_score_action();
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
                        self.move_cursor_pos_to_table();
                    }
                    CursorPos::Hand(pos) => {
                        let new_pos = pos + 1;
                        if new_pos < Hand::DICE_NUM {
                            self.cursor_pos = CursorPos::Hand(new_pos);
                        } else {
                            self.move_cursor_pos_to_table();
                        }
                    }
                    CursorPos::Dust(pos) => {
                        let new_pos = pos + 1;
                        if new_pos < Hand::DICE_NUM {
                            self.cursor_pos = CursorPos::Dust(new_pos);
                        } else {
                            self.move_cursor_pos_to_table();
                        }
                    }
                    _ => (),
                }
                AppReturn::Continue
            }

            Actions::Up => {
                match self.cursor_pos {
                    CursorPos::Hand(..) => {
                        let play = self.get_play_data().unwrap();
                        if !play.is_held.iter().all(|&x| x) {
                            self.cursor_pos = CursorPos::Role;
                        }
                    }
                    CursorPos::Dust(pos) => {
                        self.cursor_pos = CursorPos::Hand(pos);
                    }
                    CursorPos::Table(..) => {
                        self.up_action_in_score_table();
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
                    CursorPos::Table(..) => {
                        self.down_action_in_score_table();
                    }
                    _ => (),
                }
                AppReturn::Continue
            }

            _ => AppReturn::Continue,
        }
    }

    fn do_action_in_select(&mut self, input_event: InputEvent) -> AppReturn {
        match input_event.action() {
            Actions::Exit => AppReturn::Exit,

            Actions::Select => {
                self.confirm_score_action();
                AppReturn::Continue
            }

            Actions::Up => {
                self.up_action_in_score_table();
                AppReturn::Continue
            }

            Actions::Down => {
                self.down_action_in_score_table();
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
