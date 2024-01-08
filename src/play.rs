use crate::hand::Hand;
use anyhow::{bail, Result};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlayPhaseError {
    #[error("Unexpected roll count")]
    UnexpectedRollCount,
    #[error("Current Play has already rinished")]
    FinishedPlay,
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
    phase: PlayPhase,
}

impl Play {
    pub const MAX_ROLL_COUNT: usize = 3;

    pub fn new(player_id: usize) -> Self {
        Self {
            player_id,
            hand: Hand::new(),
            phase: PlayPhase::Init,
        }
    }

    pub fn progress(&mut self) -> Result<()> {
        self.phase = match self.phase {
            PlayPhase::Init => {
                self.hand.reroll_dice()?;
                PlayPhase::Roll(PlayPhase::INIT_ROLL_COUNT)
            }
            PlayPhase::Roll(count) => {
                self.hand.hold_all()?;
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
                    self.hand.reroll_dice()?;
                    PlayPhase::Roll(count + 1)
                } else {
                    bail!(PlayPhaseError::UnexpectedRollCount)
                }
            }
            PlayPhase::Select => bail!(PlayPhaseError::FinishedPlay),
        };
        Ok(())
    }

    /* helper functions */
    pub fn get_player_id(&self) -> usize {
        self.player_id
    }

    pub fn get_hand(&self) -> &Hand {
        &self.hand
    }

    pub fn get_mut_hand(&mut self) -> &mut Hand {
        &mut self.hand
    }

    pub fn get_phase(&self) -> &PlayPhase {
        &self.phase
    }
}
