use log::info;
use rand::Rng;

pub struct Hand {
    dice: Vec<u32>,
}

impl Hand {
    pub const DICE_NUM: usize = 5;
    pub const PIPS: [u32; 6] = [1, 2, 3, 4, 5, 6];

    pub fn new_with_random_n_dice(num: usize) -> Self {
        if num > Hand::DICE_NUM {
            panic!("Invalud value: please input an integer between 0-5.");
        }

        Hand {
            dice: (0..num).map(|_| Hand::random_die()).collect(),
        }
    }

    pub fn get_dice(&self) -> &[u32] {
        &self.dice
    }

    pub fn add_dice(&mut self, elements: &Hand) {
        if self.dice.len() + elements.dice.len() > Hand::DICE_NUM {
            panic!("Too many dice: the number of dice is greater than five.");
        }
        self.dice.extend(elements.get_dice());
    }

    pub fn remove_dice(&mut self, elements: &[u32]) {
        for rm_val in elements.iter() {
            if let Some(index) = self.dice.iter().rposition(|x| x == rm_val) {
                self.dice.remove(index);
            } else if Hand::PIPS.contains(rm_val) {
                info!("There is no \"{}\" to be able to remove.", rm_val);
            } else {
                info!("Invalid pip value: {}", rm_val);
            }
        }
    }

    fn random_die() -> u32 {
        Hand::PIPS[rand::thread_rng().gen_range(0..Hand::PIPS.len())]
    }
}

#[cfg(test)]
mod tests {
    use crate::hand::*;

    #[test]
    fn new_with_random_n_dice_test() {
        for num in 0..=Hand::DICE_NUM {
            let h = Hand::new_with_random_n_dice(num);
            assert_eq!(h.dice.len(), num);
        }
    }

    #[test]
    fn add_dice_test() {
        let num: usize = 3;
        let mut h0 = Hand::new_with_random_n_dice(num);
        let h1 = Hand::new_with_random_n_dice(Hand::DICE_NUM - num);
        h0.add_dice(&h1);
        assert_eq!(h0.dice.len(), Hand::DICE_NUM);
    }

    #[test]
    fn remove_dice_test() {
        let mut h0 = Hand {
            dice: vec![1, 3, 2, 3, 5],
        };
        h0.remove_dice(&[3, 1]);
        assert_eq!(h0.get_dice(), [3, 2, 5]);
        let mut h0 = Hand::new_with_random_n_dice(4);
        let d = h0.get_dice();
        let d = [d[0], d[2]];
        h0.remove_dice(&d);
        assert_eq!(h0.dice.len(), 2);
        let d = h0.get_dice();
        let d = [8, d[1]];
        h0.remove_dice(&d);
        assert_eq!(h0.dice.len(), 1);
    }
}
