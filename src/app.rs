use crate::events::{Actions, InputEvent};
use crate::game_data::GameData;
use crate::hand::{Hand, HandOpError};
use crate::play::{Play, PlayPhase};
use crate::scoring::{scoring, Boxes};
use anyhow::{anyhow, Result};
use enum_iterator::{all, first, last, Sequence};
use std::fmt;
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

#[derive(PartialEq, Eq)]
pub enum StartMenuSelection {
    Play,
    Exit,
}

impl fmt::Display for StartMenuSelection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StartMenuSelection::Play => f.pad("Play"),
            StartMenuSelection::Exit => f.pad("Exit"),
        }
    }
}

pub const LOWEST_PLAYER_ID: usize = 1;
pub const HIGHEST_PLAYER_ID: usize = 4;

#[derive(PartialEq, Eq)]
pub enum NumPlayersSelection {
    NumPlayers(usize),
    Back,
}

impl fmt::Display for NumPlayersSelection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NumPlayersSelection::NumPlayers(n) => f.pad(&format!("{} Players", n)),
            NumPlayersSelection::Back => f.pad("Back"),
        }
    }
}

pub enum AppState {
    StartMenu(StartMenuSelection),
    SelectNumPlayers(NumPlayersSelection),
    Play(Option<Play>, PlayCursorPos),
    Result,
}

impl AppState {
    fn initialized_start_menu_state() -> Self {
        Self::StartMenu(StartMenuSelection::Play)
    }

    fn initialized_select_num_players_state() -> Self {
        Self::SelectNumPlayers(NumPlayersSelection::NumPlayers(LOWEST_PLAYER_ID))
    }

    fn initialized_play_state() -> Self {
        Self::Play(None, PlayCursorPos::Disappear)
    }

    pub fn get_play_data(&self) -> Result<&Play> {
        match self {
            Self::Play(play, ..) => play
                .as_ref()
                .ok_or_else(|| anyhow!(AppStateError::NoPlayData)),
            _ => Err(anyhow!(AppStateError::UnexpectedState)),
        }
    }

    fn get_mut_play_data(&mut self) -> Result<&mut Play> {
        match self {
            Self::Play(play, ..) => play
                .as_mut()
                .ok_or_else(|| anyhow!(AppStateError::NoPlayData)),
            _ => Err(anyhow!(AppStateError::UnexpectedState)),
        }
    }

    fn initialize_play_data(&mut self, player_id: usize) -> Result<()> {
        match self {
            Self::Play(play, ..) => {
                if play.is_none() {
                    *play = Some(Play::new(player_id));
                    Ok(())
                } else {
                    Err(anyhow!(AppStateError::ExistPlayData))
                }
            }
            _ => Err(anyhow!(AppStateError::UnexpectedState)),
        }
    }

    fn cleanup_play_data(&mut self) -> Result<()> {
        match self {
            Self::Play(..) => {
                *self = Self::initialized_play_state();
                Ok(())
            }
            _ => Err(anyhow!(AppStateError::UnexpectedState)),
        }
    }

    pub fn get_play_cursor_pos(&self) -> Result<&PlayCursorPos> {
        match self {
            AppState::Play(.., pos) => Ok(pos),
            _ => Err(anyhow!(AppStateError::UnexpectedState)),
        }
    }

    fn set_play_cursor_pos(&mut self, new_pos: PlayCursorPos) -> Result<()> {
        match self {
            AppState::Play(.., pos) => {
                *pos = new_pos;
                Ok(())
            }
            _ => Err(anyhow!(AppStateError::UnexpectedState)),
        }
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

pub struct App {
    state: AppState,
    game_data: Option<GameData>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::StartMenu(StartMenuSelection::Play),
            game_data: None,
        }
    }

    /* helper functions */
    pub fn get_state(&self) -> &AppState {
        &self.state
    }

    pub fn get_game_data(&self) -> &GameData {
        self.game_data.as_ref().unwrap()
    }

    fn get_mut_game_data(&mut self) -> &mut GameData {
        self.game_data.as_mut().unwrap()
    }

    /* action handlers */
    pub fn do_action(&mut self, input_event: InputEvent) -> AppReturn {
        match self.state {
            AppState::StartMenu(..) => self.do_action_in_start_menu(input_event),
            AppState::SelectNumPlayers(..) => self.do_action_in_select_num_players(input_event),
            AppState::Play(..) => self.do_action_in_play(input_event),
            AppState::Result => self.do_action_in_result(input_event),
        }
    }

    fn do_action_in_start_menu(&mut self, input_event: InputEvent) -> AppReturn {
        match input_event.action() {
            Actions::Exit => AppReturn::Exit,

            Actions::Select => {
                let AppState::StartMenu(pos) = &self.state else {
                    panic!("Unexpected state")
                };
                match pos {
                    StartMenuSelection::Play => {
                        self.state = AppState::initialized_select_num_players_state();
                        AppReturn::Continue
                    }
                    StartMenuSelection::Exit => AppReturn::Exit,
                }
            }

            Actions::Up => {
                let AppState::StartMenu(pos) = &mut self.state else {
                    panic!("Unexpected state")
                };
                match pos {
                    StartMenuSelection::Play => {
                        *pos = StartMenuSelection::Exit;
                    }
                    StartMenuSelection::Exit => {
                        *pos = StartMenuSelection::Play;
                    }
                }
                AppReturn::Continue
            }

            Actions::Down => {
                let AppState::StartMenu(pos) = &mut self.state else {
                    panic!("Unexpected state")
                };
                match pos {
                    StartMenuSelection::Play => {
                        *pos = StartMenuSelection::Exit;
                    }
                    StartMenuSelection::Exit => {
                        *pos = StartMenuSelection::Play;
                    }
                }
                AppReturn::Continue
            }

            _ => AppReturn::Continue,
        }
    }

    fn do_action_in_select_num_players(&mut self, input_event: InputEvent) -> AppReturn {
        match input_event.action() {
            Actions::Exit => AppReturn::Exit,

            Actions::Select => {
                let AppState::SelectNumPlayers(pos) = &self.state else {
                    panic!("Unexpected state")
                };
                match pos {
                    &NumPlayersSelection::NumPlayers(num_players) => {
                        self.state = AppState::initialized_play_state();
                        self.game_data = Some(GameData::new(num_players));
                    }
                    NumPlayersSelection::Back => {
                        self.state = AppState::initialized_start_menu_state();
                    }
                }
                AppReturn::Continue
            }

            Actions::Up => {
                let AppState::SelectNumPlayers(pos) = &mut self.state else {
                    panic!("Unexpected state")
                };
                *pos = match pos {
                    &mut NumPlayersSelection::NumPlayers(num_players) => {
                        if num_players == LOWEST_PLAYER_ID {
                            NumPlayersSelection::Back
                        } else if LOWEST_PLAYER_ID < num_players && num_players <= HIGHEST_PLAYER_ID
                        {
                            NumPlayersSelection::NumPlayers(num_players - 1)
                        } else {
                            panic!("Unexpected NumPlayers value");
                        }
                    }
                    NumPlayersSelection::Back => NumPlayersSelection::NumPlayers(HIGHEST_PLAYER_ID),
                };
                AppReturn::Continue
            }

            Actions::Down => {
                let AppState::SelectNumPlayers(pos) = &mut self.state else {
                    panic!("Unexpected state")
                };
                *pos = match pos {
                    &mut NumPlayersSelection::NumPlayers(num_players) => {
                        if (LOWEST_PLAYER_ID..HIGHEST_PLAYER_ID).contains(&num_players) {
                            NumPlayersSelection::NumPlayers(num_players + 1)
                        } else if num_players == HIGHEST_PLAYER_ID {
                            NumPlayersSelection::Back
                        } else {
                            panic!("Unexpected NumPlayers value");
                        }
                    }
                    NumPlayersSelection::Back => NumPlayersSelection::NumPlayers(LOWEST_PLAYER_ID),
                };
                AppReturn::Continue
            }

            _ => AppReturn::Continue,
        }
    }

    fn do_action_in_play(&mut self, input_event: InputEvent) -> AppReturn {
        match self.state.get_play_data() {
            Ok(play) => match play.get_phase() {
                PlayPhase::Init => self.do_action_in_init(input_event),
                PlayPhase::Roll(..) => self.do_action_in_roll(input_event),
                PlayPhase::SelectOrReroll(..) => self.do_action_in_select_or_reroll(input_event),
                PlayPhase::Select => self.do_action_in_select(input_event),
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
                self.state.initialize_play_data(pid).unwrap();
                self.state.set_play_cursor_pos(PlayCursorPos::Roll).unwrap();
                AppReturn::Continue
            }
        }
    }

    fn do_action_in_init(&mut self, input_event: InputEvent) -> AppReturn {
        match input_event.action() {
            Actions::Exit => AppReturn::Exit,

            Actions::Select => {
                let play = self.state.get_mut_play_data().unwrap();
                play.progress().unwrap();
                AppReturn::Continue
            }

            _ => AppReturn::Continue,
        }
    }

    fn move_cursor_pos_to_table(&mut self) {
        let pid = self.state.get_mut_play_data().unwrap().get_player_id();
        for pos in all::<Boxes>() {
            if !self.get_game_data().get_score_table(pid).has_score_in(pos) {
                self.state
                    .set_play_cursor_pos(PlayCursorPos::Table(pos))
                    .unwrap();
                break;
            }
        }
    }

    fn do_action_in_roll(&mut self, input_event: InputEvent) -> AppReturn {
        match input_event.action() {
            Actions::Exit => AppReturn::Exit,

            Actions::Select => {
                let play = self.state.get_mut_play_data().unwrap();

                play.progress().unwrap();
                match play.get_phase() {
                    PlayPhase::SelectOrReroll(..) => {
                        self.state
                            .set_play_cursor_pos(PlayCursorPos::Hand(0))
                            .unwrap();
                    }
                    PlayPhase::Select => {
                        self.move_cursor_pos_to_table();
                    }
                    _ => panic!("Unexpected PlayPhase"),
                }

                AppReturn::Continue
            }

            _ => {
                self.state
                    .get_mut_play_data()
                    .unwrap()
                    .get_mut_hand()
                    .reroll_dice()
                    .unwrap();
                AppReturn::Continue
            }
        }
    }

    fn up_action_in_score_table(&mut self) {
        if let &PlayCursorPos::Table(init_pos) = self.state.get_play_cursor_pos().unwrap() {
            let mut pos = init_pos;
            loop {
                let prev = pos.previous().unwrap_or_else(|| last::<Boxes>().unwrap());
                let pid = self.state.get_play_data().unwrap().get_player_id();
                let has_score = self.get_game_data().get_score_table(pid).has_score_in(prev);
                if !has_score || prev == init_pos {
                    self.state
                        .set_play_cursor_pos(PlayCursorPos::Table(prev))
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
        if let &PlayCursorPos::Table(init_pos) = self.state.get_play_cursor_pos().unwrap() {
            let mut pos = init_pos;
            loop {
                let next = pos.next().unwrap_or_else(|| first::<Boxes>().unwrap());
                let pid = self.state.get_play_data().unwrap().get_player_id();
                let has_score = self.get_game_data().get_score_table(pid).has_score_in(next);
                if !has_score || next == init_pos {
                    self.state
                        .set_play_cursor_pos(PlayCursorPos::Table(next))
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
        if let &PlayCursorPos::Table(pos) = self.state.get_play_cursor_pos().unwrap() {
            let play = self.state.get_play_data().unwrap();
            let pid = play.get_player_id();
            if !self.get_game_data().get_score_table(pid).has_score_in(pos) {
                let dice = HandOpError::unwrap_pips(play.get_hand().get_pips());
                let score_table = self.get_mut_game_data().get_mut_score_table(pid);
                score_table.confirm_score(pos, scoring(pos, &dice));
                let next_pid = (pid + 1) % self.get_game_data().get_num_players();
                self.state.cleanup_play_data().unwrap();

                let next_score_table = self.get_game_data().get_score_table(next_pid);
                if !next_score_table.has_all_scores() {
                    self.state.set_play_cursor_pos(PlayCursorPos::Roll).unwrap();
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
                match self.state.get_play_cursor_pos().unwrap() {
                    PlayCursorPos::Roll => {
                        let play = self.state.get_mut_play_data().unwrap();
                        match play.progress() {
                            Ok(..) => (),
                            Err(e) => match e.downcast_ref::<HandOpError>() {
                                Some(HandOpError::NoDiceToRoll) => (),
                                _ => panic!("{}", e),
                            },
                        }
                    }
                    &PlayCursorPos::Hand(pos) | &PlayCursorPos::Dust(pos) => {
                        let hand = self.state.get_mut_play_data().unwrap().get_mut_hand();
                        hand.hold(pos, !hand.is_held(pos).unwrap()).unwrap();
                    }
                    PlayCursorPos::Table(..) => {
                        self.confirm_score_action();
                    }
                    _ => (),
                }
                AppReturn::Continue
            }

            Actions::Left => {
                match self.state.get_play_cursor_pos().unwrap() {
                    &PlayCursorPos::Hand(pos) => {
                        if pos > 0 {
                            self.state
                                .set_play_cursor_pos(PlayCursorPos::Hand(pos - 1))
                                .unwrap();
                        }
                    }
                    &PlayCursorPos::Dust(pos) => {
                        if pos > 0 {
                            self.state
                                .set_play_cursor_pos(PlayCursorPos::Dust(pos - 1))
                                .unwrap();
                        }
                    }
                    PlayCursorPos::Table(..) => {
                        self.state
                            .set_play_cursor_pos(PlayCursorPos::Hand(Hand::DICE_NUM - 1))
                            .unwrap();
                    }
                    _ => (),
                }
                AppReturn::Continue
            }

            Actions::Right => {
                match self.state.get_play_cursor_pos().unwrap() {
                    PlayCursorPos::Roll => {
                        self.move_cursor_pos_to_table();
                    }
                    PlayCursorPos::Hand(pos) => {
                        let new_pos = pos + 1;
                        if new_pos < Hand::DICE_NUM {
                            self.state
                                .set_play_cursor_pos(PlayCursorPos::Hand(new_pos))
                                .unwrap();
                        } else {
                            self.move_cursor_pos_to_table();
                        }
                    }
                    PlayCursorPos::Dust(pos) => {
                        let new_pos = pos + 1;
                        if new_pos < Hand::DICE_NUM {
                            self.state
                                .set_play_cursor_pos(PlayCursorPos::Dust(new_pos))
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
                match self.state.get_play_cursor_pos().unwrap() {
                    PlayCursorPos::Hand(..) => {
                        let play = self.state.get_play_data().unwrap();
                        if !play.get_hand().is_held_all().unwrap() {
                            self.state.set_play_cursor_pos(PlayCursorPos::Roll).unwrap();
                        }
                    }
                    &PlayCursorPos::Dust(pos) => {
                        self.state
                            .set_play_cursor_pos(PlayCursorPos::Hand(pos))
                            .unwrap();
                    }
                    PlayCursorPos::Table(..) => {
                        self.up_action_in_score_table();
                    }
                    _ => (),
                }
                AppReturn::Continue
            }

            Actions::Down => {
                match self.state.get_play_cursor_pos().unwrap() {
                    PlayCursorPos::Roll => {
                        self.state
                            .set_play_cursor_pos(PlayCursorPos::Hand(0))
                            .unwrap();
                    }
                    &PlayCursorPos::Hand(pos) => {
                        self.state
                            .set_play_cursor_pos(PlayCursorPos::Dust(pos))
                            .unwrap();
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
