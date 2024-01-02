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

    pub fn remove_dice(&mut self, removes: &[bool]) {
        if removes.len() > self.dice.len() {
            panic!("array length for removal is larger than holding dice length");
        }

        for (i, _) in removes.iter().enumerate().rev().filter(|&(_, &rm)| rm) {
            self.dice.remove(i);
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
            dice: vec![5, 3, 2, 3, 5],
        };
        h0.remove_dice(&[true, true, false, false, false]);
        assert_eq!(h0.get_dice(), [2, 3, 5]);

        let mut h0 = Hand {
            dice: vec![1, 3, 2, 3, 5],
        };
        h0.remove_dice(&[true, false, false, true, false]);
        assert_eq!(h0.get_dice(), [3, 2, 5]);

        let mut h0 = Hand::new_with_random_n_dice(4);
        let org = h0.get_dice().to_vec();
        let d = [true, false, true];
        h0.remove_dice(&d);
        assert_eq!(h0.dice.len(), 2);
        assert_eq!(h0.dice, [org[1], org[3]]);
        let d = [false, true];
        h0.remove_dice(&d);
        assert_eq!(h0.dice.len(), 1);
        assert_eq!(h0.dice, [org[1]]);
    }
}
