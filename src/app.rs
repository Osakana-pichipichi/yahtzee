use crate::events::{Actions, InputEvent};
use crate::hand::Hand;
use crate::score_table::ScoreTable;
use crate::scoring::{scoring, Boxes};
use anyhow::{anyhow, Result};
use array_macro::array;
use enum_iterator::{all, first, last, Sequence};
use thiserror::Error;

#[derive(PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

#[derive(PartialEq, Eq)]
pub enum PlayCursorPos {
    Roll,
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

    pub fn start_first_roll(&mut self) {
        self.hand = Hand::new_with_random_n_dice(Hand::DICE_NUM);
        self.game_phase = GamePhase::Roll(0);
    }

    pub fn reroll_dice(&mut self) -> usize {
        let removes = self
            .is_held
            .iter()
            .map(|&is_heled| !is_heled)
            .collect::<Vec<_>>();
        self.hand.remove_dice(&removes);

        let reroll_dices = removes.iter().filter(|&&e| e).count();
        let rests = Hand::DICE_NUM - reroll_dices;
        self.is_held = array![i => i < rests; Hand::DICE_NUM];
        self.hand
            .add_dice(&Hand::new_with_random_n_dice(reroll_dices));

        reroll_dices
    }
}

pub enum AppState {
    Play(Option<Play>, PlayCursorPos),
    Result,
}

impl AppState {
    fn initialized_play_state() -> Self {
        AppState::Play(None, PlayCursorPos::Disappear)
    }
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
    game_data: Option<GameData>,
}

impl App {
    const MAX_ROLL_COUNT: usize = 3;

    pub fn new(num_players: usize) -> Self {
        Self {
            state: AppState::initialized_play_state(),
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
            AppState::Play(p, ..) => p.as_ref().ok_or_else(|| anyhow!(AppStateError::NoPlayData)),
            _ => Err(anyhow!(AppStateError::UnexpectedState)),
        }
    }

    fn get_mut_play_data(&mut self) -> Result<&mut Play> {
        match &mut self.state {
            AppState::Play(p, ..) => p.as_mut().ok_or_else(|| anyhow!(AppStateError::NoPlayData)),
            _ => Err(anyhow!(AppStateError::UnexpectedState)),
        }
    }

    fn initialize_play_data(&mut self, player_id: usize) -> Result<&Play> {
        match &self.state {
            AppState::Play(p, ..) => {
                if p.is_none() {
                    self.state = AppState::Play(Some(Play::new(player_id)), PlayCursorPos::Roll);
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
                self.state = AppState::initialized_play_state();
                Ok(())
            }
            _ => Err(anyhow!(AppStateError::UnexpectedState)),
        }
    }

    pub fn get_play_cursor_pos(&self) -> Result<&PlayCursorPos> {
        match &self.state {
            AppState::Play(.., pos) => Ok(pos),
            _ => Err(anyhow!(AppStateError::UnexpectedState)),
        }
    }

    fn set_play_cursor_pos(&mut self, new_pos: PlayCursorPos) -> Result<()> {
        match &mut self.state {
            AppState::Play(_, pos) => {
                *pos = new_pos;
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
                GamePhase::Init => self.do_action_in_init(input_event),
                GamePhase::Roll(..) => self.do_action_in_roll(input_event),
                GamePhase::SelectOrReroll(..) => self.do_action_in_select_or_reroll(input_event),
                GamePhase::Select => self.do_action_in_select(input_event),
            },
            Err(e) => match e.downcast_ref::<AppStateError>().unwrap() {
                AppStateError::NoPlayData => self.do_action_in_no_play_data(input_event),
                _ => panic!("{}", e),
            },
        }
    }

    fn do_action_in_no_play_data(&mut self, input_event: InputEvent) -> AppReturn {
        match input_event.action() {
            Actions::Exit => AppReturn::Exit,

            _ => {
                let pid = self.get_game_data().current_player_id();
                self.initialize_play_data(pid).unwrap();
                self.set_play_cursor_pos(PlayCursorPos::Roll).unwrap();
                AppReturn::Continue
            }
        }
    }

    fn do_action_in_init(&mut self, input_event: InputEvent) -> AppReturn {
        match input_event.action() {
            Actions::Exit => AppReturn::Exit,

            Actions::Select => {
                let play = self.get_mut_play_data().unwrap();
                play.start_first_roll();
                AppReturn::Continue
            }

            _ => AppReturn::Continue,
        }
    }

    fn move_cursor_pos_to_table(&mut self) {
        let pid = self.get_mut_play_data().unwrap().player_id;
        for pos in all::<Boxes>() {
            if !self.get_game_data().get_score_table(pid).has_score_in(pos) {
                self.set_play_cursor_pos(PlayCursorPos::Table(pos)).unwrap();
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
                    self.set_play_cursor_pos(PlayCursorPos::Hand(0)).unwrap();
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
        if let &PlayCursorPos::Table(init_pos) = self.get_play_cursor_pos().unwrap() {
            let mut pos = init_pos;
            loop {
                let prev = pos.previous().unwrap_or_else(|| last::<Boxes>().unwrap());
                let pid = self.get_play_data().unwrap().player_id;
                let has_score = self.get_game_data().get_score_table(pid).has_score_in(prev);
                if !has_score || prev == init_pos {
                    self.set_play_cursor_pos(PlayCursorPos::Table(prev))
                        .unwrap();
                    break;
                }
                pos = prev;
            }
        } else {
            panic!("Unexpected curson position");
        }
    }

    fn down_action_in_score_table(&mut self) {
        if let &PlayCursorPos::Table(init_pos) = self.get_play_cursor_pos().unwrap() {
            let mut pos = init_pos;
            loop {
                let next = pos.next().unwrap_or_else(|| first::<Boxes>().unwrap());
                let pid = self.get_play_data().unwrap().player_id;
                let has_score = self.get_game_data().get_score_table(pid).has_score_in(next);
                if !has_score || next == init_pos {
                    self.set_play_cursor_pos(PlayCursorPos::Table(next))
                        .unwrap();
                    break;
                }
                pos = next;
            }
        } else {
            panic!("Unexpected curson position");
        }
    }

    fn confirm_score_action(&mut self) {
        if let &PlayCursorPos::Table(pos) = self.get_play_cursor_pos().unwrap() {
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
                    self.set_play_cursor_pos(PlayCursorPos::Roll).unwrap();
                } else {
                    self.state = AppState::Result;
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
                match self.get_play_cursor_pos().unwrap() {
                    PlayCursorPos::Roll => {
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
                    &PlayCursorPos::Hand(pos) | &PlayCursorPos::Dust(pos) => {
                        let play = self.get_mut_play_data().unwrap();
                        play.is_held[pos] = !play.is_held[pos]
                    }
                    PlayCursorPos::Table(..) => {
                        self.confirm_score_action();
                    }
                    _ => (),
                }
                AppReturn::Continue
            }

            Actions::Left => {
                match self.get_play_cursor_pos().unwrap() {
                    &PlayCursorPos::Hand(pos) => {
                        if pos > 0 {
                            self.set_play_cursor_pos(PlayCursorPos::Hand(pos - 1))
                                .unwrap();
                        }
                    }
                    &PlayCursorPos::Dust(pos) => {
                        if pos > 0 {
                            self.set_play_cursor_pos(PlayCursorPos::Dust(pos - 1))
                                .unwrap();
                        }
                    }
                    PlayCursorPos::Table(..) => {
                        self.set_play_cursor_pos(PlayCursorPos::Hand(Hand::DICE_NUM - 1))
                            .unwrap();
                    }
                    _ => (),
                }
                AppReturn::Continue
            }

            Actions::Right => {
                match self.get_play_cursor_pos().unwrap() {
                    PlayCursorPos::Roll => {
                        self.move_cursor_pos_to_table();
                    }
                    PlayCursorPos::Hand(pos) => {
                        let new_pos = pos + 1;
                        if new_pos < Hand::DICE_NUM {
                            self.set_play_cursor_pos(PlayCursorPos::Hand(new_pos))
                                .unwrap();
                        } else {
                            self.move_cursor_pos_to_table();
                        }
                    }
                    PlayCursorPos::Dust(pos) => {
                        let new_pos = pos + 1;
                        if new_pos < Hand::DICE_NUM {
                            self.set_play_cursor_pos(PlayCursorPos::Dust(new_pos))
                                .unwrap();
                        } else {
                            self.move_cursor_pos_to_table();
                        }
                    }
                    _ => (),
                }
                AppReturn::Continue
            }

            Actions::Up => {
                match self.get_play_cursor_pos().unwrap() {
                    PlayCursorPos::Hand(..) => {
                        let play = self.get_play_data().unwrap();
                        if !play.is_held.iter().all(|&x| x) {
                            self.set_play_cursor_pos(PlayCursorPos::Roll).unwrap();
                        }
                    }
                    &PlayCursorPos::Dust(pos) => {
                        self.set_play_cursor_pos(PlayCursorPos::Hand(pos)).unwrap();
                    }
                    PlayCursorPos::Table(..) => {
                        self.up_action_in_score_table();
                    }
                    _ => (),
                }
                AppReturn::Continue
            }

            Actions::Down => {
                match self.get_play_cursor_pos().unwrap() {
                    PlayCursorPos::Roll => {
                        self.set_play_cursor_pos(PlayCursorPos::Hand(0)).unwrap();
                    }
                    &PlayCursorPos::Hand(pos) => {
                        self.set_play_cursor_pos(PlayCursorPos::Dust(pos)).unwrap();
                    }
                    PlayCursorPos::Table(..) => {
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
