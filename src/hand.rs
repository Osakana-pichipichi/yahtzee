use anyhow::{anyhow, bail, ensure, Result};
use rand::Rng;
use thiserror::Error;

pub struct Die {
    pip: u32,
    is_held: bool,
}

impl Die {
    pub const PIPS: [u32; 6] = [1, 2, 3, 4, 5, 6];

    fn new() -> Self {
        Self {
            pip: Self::PIPS[rand::thread_rng().gen_range(0..Self::PIPS.len())],
            is_held: false,
        }
    }

    fn gen_n_dice(num: usize) -> Vec<Die> {
        (0..num).map(|_| Die::new()).collect()
    }

    fn pip(&self) -> u32 {
        self.pip
    }

    fn hold(&mut self, hold: bool) {
        self.is_held = hold;
    }

    fn is_held(&self) -> bool {
        self.is_held
    }
}

#[derive(Debug, Error)]
pub enum HandOpError {
    #[error("There is no die in pos {0}")]
    NoDie(usize),
    #[error("The number of dice must be {} or less", Hand::DICE_NUM)]
    OutOfPossibleRange,
    #[error("No dice to roll")]
    NoDiceToRoll,
    #[error("No dice to reroll")]
    NoDiceToReroll,
    #[error("Hand is too big")]
    TooBigHand,
    #[error("Hand is not fully filled")]
    NotFullyFilled,
    #[error("Return not fully filled hand")]
    ReturnShortHand(Vec<u32>),
}

impl HandOpError {
    pub fn unwrap_pips(pips: Result<Vec<u32>>) -> Vec<u32> {
        match pips {
            Ok(p) => p,
            Err(e) => match e.downcast_ref::<Self>() {
                Some(Self::ReturnShortHand(p)) => p.to_vec(),
                _ => panic!("{:?}", e),
            },
        }
    }
}

pub struct Hand {
    dice: Vec<Die>,
}

impl Default for Hand {
    fn default() -> Self {
        Self::new()
    }
}

impl Hand {
    pub const DICE_NUM: usize = 5;

    pub fn new() -> Self {
        Hand { dice: vec![] }
    }

    pub fn get_dice(&self) -> &Vec<Die> {
        &self.dice
    }

    pub fn get_pips(&self) -> Result<Vec<u32>> {
        ensure!(self.dice.len() <= Self::DICE_NUM, HandOpError::TooBigHand);

        let pips: Vec<_> = self.dice.iter().map(|d| d.pip()).collect();
        if pips.len() == Self::DICE_NUM {
            Ok(pips)
        } else {
            Err(anyhow!(HandOpError::ReturnShortHand(pips)))
        }
    }

    pub fn is_held(&self, pos: usize) -> Result<bool> {
        ensure!(pos < Self::DICE_NUM, HandOpError::OutOfPossibleRange);

        if let Some(d) = self.dice.get(pos) {
            Ok(d.is_held())
        } else {
            Ok(false)
        }
    }

    pub fn is_held_all(&self) -> Result<bool> {
        ensure!(self.dice.len() <= Self::DICE_NUM, HandOpError::TooBigHand);

        Ok(self.dice.len() == Self::DICE_NUM && self.dice.iter().all(|d| d.is_held))
    }

    pub fn hold(&mut self, pos: usize, hold: bool) -> Result<()> {
        ensure!(pos < Self::DICE_NUM, HandOpError::OutOfPossibleRange);

        if let Some(d) = self.dice.get_mut(pos) {
            d.hold(hold);
            Ok(())
        } else {
            bail!(HandOpError::NoDie(pos))
        }
    }

    pub fn hold_all(&mut self) -> Result<()> {
        ensure!(self.dice.len() <= Self::DICE_NUM, HandOpError::TooBigHand);

        self.dice.iter_mut().for_each(|d| d.hold(true));
        if self.is_held_all()? {
            Ok(())
        } else {
            bail!(HandOpError::NotFullyFilled)
        }
    }

    fn fill_dice(&mut self) -> Result<()> {
        ensure!(self.dice.len() < Self::DICE_NUM, HandOpError::NoDiceToRoll);

        let num = Self::DICE_NUM - self.dice.len();
        self.dice.extend(Die::gen_n_dice(num));
        Ok(())
    }

    fn remove_dice(&mut self) {
        let to_be_removed: Vec<_> = self
            .dice
            .iter()
            .enumerate()
            .rev()
            .filter_map(|(i, d)| if !d.is_held() { Some(i) } else { None })
            .collect();
        for i in to_be_removed {
            self.dice.remove(i);
        }
    }

    pub fn reroll_dice(&mut self) -> Result<()> {
        ensure!(!self.is_held_all()?, HandOpError::NoDiceToReroll);

        self.remove_dice();
        self.fill_dice()
    }
}

#[cfg(test)]
mod tests {
    use crate::hand::*;

    fn gen_hand_with_n_dice(num: usize) -> Hand {
        if num > Hand::DICE_NUM {
            panic!("num is too big")
        }
        Hand {
            dice: Die::gen_n_dice(num),
        }
    }

    #[test]
    fn new_test() {
        for num in 0..=Hand::DICE_NUM {
            let h = gen_hand_with_n_dice(num);
            assert_eq!(h.dice.len(), num);
            for d in h.dice.iter() {
                assert!(Die::PIPS.contains(&d.pip));
                assert!(!d.is_held);
                assert_eq!(d.is_held, d.is_held());
            }
        }
    }

    #[test]
    fn hold_test() {
        let pos = 3;
        let mut h = gen_hand_with_n_dice(pos + 1);

        let is_held = true;
        h.hold(pos, is_held).unwrap();
        assert_eq!(h.is_held(pos).unwrap(), is_held);
    }

    #[test]
    fn fill_dice_test() {
        let mut h0 = gen_hand_with_n_dice(0);
        h0.fill_dice().unwrap();
        assert_eq!(h0.dice.len(), Hand::DICE_NUM);

        const NUM: usize = 3;
        let mut h0 = gen_hand_with_n_dice(NUM);
        let org = HandOpError::unwrap_pips(h0.get_pips());
        let holds = [true, false, true];
        holds
            .iter()
            .enumerate()
            .for_each(|(p, &h)| h0.hold(p, h).unwrap());
        h0.fill_dice().unwrap();
        assert_eq!(h0.dice.len(), Hand::DICE_NUM);
        assert_eq!(HandOpError::unwrap_pips(h0.get_pips())[0..NUM], org);

        let mut h0 = gen_hand_with_n_dice(Hand::DICE_NUM);
        let holds = [false; Hand::DICE_NUM];
        holds
            .iter()
            .enumerate()
            .for_each(|(p, &h)| h0.hold(p, h).unwrap());
        match h0.fill_dice() {
            Ok(..) => panic!("Must not return Ok"),
            Err(e) => match e.downcast_ref::<HandOpError>() {
                Some(HandOpError::NoDiceToRoll) => (),
                _ => panic!("Shuld return HandOpError::NoDiceToRoll"),
            },
        }
    }

    #[test]
    fn reroll_dice_test() {
        let mut h0 = gen_hand_with_n_dice(0);
        h0.reroll_dice().unwrap();
        assert_eq!(h0.dice.len(), Hand::DICE_NUM);

        const NUM: usize = 3;
        let mut h0 = gen_hand_with_n_dice(NUM);
        let org = HandOpError::unwrap_pips(h0.get_pips());
        let holds = [true, false, true];
        holds
            .iter()
            .enumerate()
            .for_each(|(p, &h)| h0.hold(p, h).unwrap());
        h0.reroll_dice().unwrap();
        assert_eq!(h0.dice.len(), Hand::DICE_NUM);
        let hold_cnt = holds.iter().filter(|&&h| h).count();
        assert_eq!(
            HandOpError::unwrap_pips(h0.get_pips())[0..hold_cnt],
            [org[0], org[2]]
        );

        let mut h0 = gen_hand_with_n_dice(Hand::DICE_NUM);
        let org = HandOpError::unwrap_pips(h0.get_pips());
        let holds = [true, false, true, true, false];
        holds
            .iter()
            .enumerate()
            .for_each(|(p, &h)| h0.hold(p, h).unwrap());
        h0.reroll_dice().unwrap();
        assert_eq!(h0.dice.len(), Hand::DICE_NUM);
        let hold_cnt = holds.iter().filter(|&&h| h).count();
        assert_eq!(
            HandOpError::unwrap_pips(h0.get_pips())[0..hold_cnt],
            [org[0], org[2], org[3]]
        );

        let mut h0 = gen_hand_with_n_dice(Hand::DICE_NUM);
        let holds = [true; Hand::DICE_NUM];
        holds
            .iter()
            .enumerate()
            .for_each(|(p, &h)| h0.hold(p, h).unwrap());
        match h0.reroll_dice() {
            Ok(..) => panic!("Must not return Ok"),
            Err(e) => match e.downcast_ref::<HandOpError>() {
                Some(HandOpError::NoDiceToReroll) => (),
                _ => panic!("Shuld return HandOpError::NoDiceToReroll"),
            },
        }
    }

    #[test]
    fn remove_dice_test() {
        let mut h0 = gen_hand_with_n_dice(5);
        let org = HandOpError::unwrap_pips(h0.get_pips());
        let rms = [true, true, false, false, false];
        rms.iter()
            .enumerate()
            .for_each(|(p, &rm)| h0.hold(p, !rm).unwrap());
        h0.remove_dice();
        assert_eq!(
            HandOpError::unwrap_pips(h0.get_pips()),
            [org[2], org[3], org[4]]
        );

        let mut h1 = gen_hand_with_n_dice(5);
        let org = HandOpError::unwrap_pips(h1.get_pips());
        let rms = [true, false, true, true, false];
        rms.iter()
            .enumerate()
            .for_each(|(p, &rm)| h1.hold(p, !rm).unwrap());
        h1.remove_dice();
        assert_eq!(HandOpError::unwrap_pips(h1.get_pips()), [org[1], org[4]]);

        let mut h2 = gen_hand_with_n_dice(4);
        let org = HandOpError::unwrap_pips(h2.get_pips());
        let rms = [true, false, true, false];
        rms.iter()
            .enumerate()
            .for_each(|(p, &rm)| h2.hold(p, !rm).unwrap());
        h2.remove_dice();
        assert_eq!(HandOpError::unwrap_pips(h2.get_pips()), [org[1], org[3]]);
        let rms = [false, true];
        rms.iter()
            .enumerate()
            .for_each(|(p, &rm)| h2.hold(p, !rm).unwrap());
        h2.remove_dice();
        assert_eq!(HandOpError::unwrap_pips(h2.get_pips()), [org[1]]);
    }
}
