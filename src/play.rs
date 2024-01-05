use crate::hand::Hand;
use anyhow::{anyhow, bail, Result};
use array_macro::array;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlayPhaseError {
    #[error("Unexpected PlayPhase")]
    UnexpectedPlayPhase,
    #[error("Unexpected roll count")]
    UnexpectedRollCount,
    #[error("Current Play has already rinished")]
    FinishedPlay,
    #[error("Try to roll while not in the roll phase")]
    DisAllowedRoll,
    #[error("No dice to roll")]
    NoDiceToRoll,
}

pub enum PlayPhase {
    Init,
    Roll(usize),
    SelectOrReroll(usize),
    Select,
}

impl PlayPhase {
    pub const INIT_ROLL_COUNT: usize = 1;
    pub const MAX_ROLL_COUNT: usize = 3;
}

pub struct Play {
    player_id: usize,
    hand: Hand,
    is_held: [bool; Hand::DICE_NUM],
    phase: PlayPhase,
}

impl Play {
    pub const MAX_ROLL_COUNT: usize = 3;

    pub fn new(player_id: usize) -> Self {
        Self {
            player_id,
            hand: Hand::new_with_random_n_dice(0),
            is_held: [false; Hand::DICE_NUM],
            phase: PlayPhase::Init,
        }
    }

    fn start_first_roll(&mut self) {
        self.hand = Hand::new_with_random_n_dice(Hand::DICE_NUM);
    }

    pub fn reroll_dice(&mut self) -> Result<()> {
        if let PlayPhase::Roll(..) = self.phase {
            let removes: Vec<_> = self.is_held.iter().map(|&is_heled| !is_heled).collect();
            self.hand.remove_dice(&removes);

            let reroll_dices = removes.iter().filter(|&&e| e).count();
            let rests = Hand::DICE_NUM - reroll_dices;
            self.is_held = array![i => i < rests; Hand::DICE_NUM];
            self.hand
                .add_dice(&Hand::new_with_random_n_dice(reroll_dices));
            Ok(())
        } else {
            Err(anyhow!(PlayPhaseError::DisAllowedRoll))
        }
    }

    fn transition_phase(&self) -> Result<PlayPhase> {
        Ok(match self.phase {
            PlayPhase::Init => PlayPhase::Roll(PlayPhase::INIT_ROLL_COUNT),
            PlayPhase::Roll(count) => {
                if (PlayPhase::INIT_ROLL_COUNT..PlayPhase::MAX_ROLL_COUNT).contains(&count) {
                    PlayPhase::SelectOrReroll(count)
                } else if count == Self::MAX_ROLL_COUNT {
                    PlayPhase::Select
                } else {
                    bail!(PlayPhaseError::UnexpectedRollCount)
                }
            }
            PlayPhase::SelectOrReroll(count) => {
                if (PlayPhase::INIT_ROLL_COUNT..PlayPhase::MAX_ROLL_COUNT).contains(&count) {
                    if !self.get_is_held_all() {
                        PlayPhase::Roll(count + 1)
                    } else {
                        bail!(PlayPhaseError::NoDiceToRoll)
                    }
                } else {
                    bail!(PlayPhaseError::UnexpectedRollCount)
                }
            }
            PlayPhase::Select => bail!(PlayPhaseError::FinishedPlay),
        })
    }

    pub fn progress(&mut self) -> Result<()> {
        self.phase = self.transition_phase()?;
        match self.phase {
            PlayPhase::Roll(PlayPhase::INIT_ROLL_COUNT) => self.start_first_roll(),
            PlayPhase::Roll(..) => self.reroll_dice()?,
            PlayPhase::SelectOrReroll(..) | PlayPhase::Select => self.hold_all_dice(),
            _ => bail!(PlayPhaseError::UnexpectedPlayPhase),
        };
        Ok(())
    }

    /* helper functions */
    pub fn get_player_id(&self) -> usize {
        self.player_id
    }

    pub fn get_hand(&self) -> &[u32] {
        self.hand.get_dice()
    }

    pub fn get_mut_hand(&mut self) -> &mut Hand {
        &mut self.hand
    }

    pub fn get_is_held(&self, pos: usize) -> bool {
        if pos > self.is_held.len() {
            panic!("pos is out of range");
        };

        self.is_held[pos]
    }

    pub fn get_is_held_all(&self) -> bool {
        self.is_held.iter().all(|&x| x)
    }

    pub fn set_is_held(&mut self, pos: usize, is_held: bool) {
        if pos > self.is_held.len() {
            panic!("pos is out of range");
        };

        self.is_held[pos] = is_held;
    }

    pub fn hold_all_dice(&mut self) {
        self.is_held = [true; Hand::DICE_NUM];
    }

    pub fn get_phase(&self) -> &PlayPhase {
        &self.phase
    }
}
