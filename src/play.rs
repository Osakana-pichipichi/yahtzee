use crate::hand::Hand;
use array_macro::array;

pub enum PlayPhase {
    Init,
    Roll(usize),
    SelectOrReroll(usize),
    Select,
}

pub struct Play {
    player_id: usize,
    hand: Hand,
    is_held: [bool; Hand::DICE_NUM],
    phase: PlayPhase,
}

impl Play {
    pub fn new(player_id: usize) -> Self {
        Self {
            player_id,
            hand: Hand::new_with_random_n_dice(0),
            is_held: [false; Hand::DICE_NUM],
            phase: PlayPhase::Init,
        }
    }

    pub fn start_first_roll(&mut self) {
        self.hand = Hand::new_with_random_n_dice(Hand::DICE_NUM);
        self.phase = PlayPhase::Roll(0);
    }

    pub fn reroll_dice(&mut self) -> usize {
        let removes: Vec<_> = self.is_held.iter().map(|&is_heled| !is_heled).collect();
        self.hand.remove_dice(&removes);

        let reroll_dices = removes.iter().filter(|&&e| e).count();
        let rests = Hand::DICE_NUM - reroll_dices;
        self.is_held = array![i => i < rests; Hand::DICE_NUM];
        self.hand
            .add_dice(&Hand::new_with_random_n_dice(reroll_dices));

        reroll_dices
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

    pub fn set_phase(&mut self, phase: PlayPhase) {
        self.phase = phase;
    }
}
