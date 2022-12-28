use crate::hand::Hand;
use array_macro::array;
use enum_iterator::Sequence;
use std::collections::HashSet;
use std::fmt;
use std::iter::FromIterator;

const FULL_HOUSE_SCORE: u32 = 25;
const SMALL_STRAIGHT_SCORE: u32 = 30;
const LARGE_STRAIGHT_SCORE: u32 = 40;
const YAHTZEE_SCORE: u32 = 50;

#[derive(PartialEq, Eq, Hash, Sequence, Clone, Copy)]
pub enum Boxes {
    Aces,
    Twos,
    Threes,
    Fours,
    Fives,
    Sixes,
    ThreeOfaAKind,
    FourOfaAKind,
    FullHouse,
    SmallStraight,
    LargeStraight,
    Yahtzee,
    Chance,
}

impl fmt::Display for Boxes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Boxes::Aces => f.pad("Aces"),
            Boxes::Twos => f.pad("Twos"),
            Boxes::Threes => f.pad("Threes"),
            Boxes::Fours => f.pad("Fours"),
            Boxes::Fives => f.pad("Fives"),
            Boxes::Sixes => f.pad("Sixes"),
            Boxes::ThreeOfaAKind => f.pad("Three of a kind"),
            Boxes::FourOfaAKind => f.pad("Four of a kind"),
            Boxes::FullHouse => f.pad("Full house"),
            Boxes::SmallStraight => f.pad("Small straight"),
            Boxes::LargeStraight => f.pad("Large straight"),
            Boxes::Yahtzee => f.pad("Yahtzee"),
            Boxes::Chance => f.pad("Chance"),
        }
    }
}

pub fn scoring(b: Boxes, dice: &[u32]) -> u32 {
    if dice.len() != Hand::DICE_NUM {
        panic!(
            "A hand needs {} dice, but your hand has only {} dice.",
            Hand::DICE_NUM,
            dice.len()
        );
    }
    match b {
        Boxes::Aces => aces(dice),
        Boxes::Twos => twos(dice),
        Boxes::Threes => threes(dice),
        Boxes::Fours => fours(dice),
        Boxes::Fives => fives(dice),
        Boxes::Sixes => sixes(dice),
        Boxes::ThreeOfaAKind => three_of_a_kind(dice),
        Boxes::FourOfaAKind => four_of_a_kind(dice),
        Boxes::FullHouse => full_house(dice),
        Boxes::SmallStraight => small_straight(dice),
        Boxes::LargeStraight => large_straight(dice),
        Boxes::Yahtzee => yahtzee(dice),
        Boxes::Chance => chance(dice),
    }
}

fn aces(dice: &[u32]) -> u32 {
    upper_section_scoring(dice, 1)
}

fn twos(dice: &[u32]) -> u32 {
    upper_section_scoring(dice, 2)
}

fn threes(dice: &[u32]) -> u32 {
    upper_section_scoring(dice, 3)
}

fn fours(dice: &[u32]) -> u32 {
    upper_section_scoring(dice, 4)
}

fn fives(dice: &[u32]) -> u32 {
    upper_section_scoring(dice, 5)
}

fn sixes(dice: &[u32]) -> u32 {
    upper_section_scoring(dice, 6)
}

fn n_of_a_kind(dice: &[u32], n: u32) -> u32 {
    let n: usize = n as usize;
    for d in dice[0..=(Hand::DICE_NUM - n)].iter() {
        if dice.iter().filter(|&x| x == d).count() >= n {
            return dice.iter().sum();
        }
    }

    0
}

fn three_of_a_kind(dice: &[u32]) -> u32 {
    n_of_a_kind(dice, 3)
}

fn four_of_a_kind(dice: &[u32]) -> u32 {
    n_of_a_kind(dice, 4)
}

fn full_house(dice: &[u32]) -> u32 {
    let mut sorted_dice: Vec<u32> = dice.to_vec();
    sorted_dice.sort_unstable();
    let init = sorted_dice[0];
    let end = sorted_dice[Hand::DICE_NUM - 1];
    let case0 = [init, init, init, end, end];
    let case1 = [init, init, end, end, end];

    if (init != end) && (sorted_dice == case0 || sorted_dice == case1) {
        FULL_HOUSE_SCORE
    } else {
        0
    }
}

fn small_straight(dice: &[u32]) -> u32 {
    let unique_dice: HashSet<u32> = dice.iter().copied().collect();
    let mut sorted_dice: Vec<u32> = Vec::from_iter(unique_dice);
    sorted_dice.sort_unstable();

    if sorted_dice.len() < 4 {
        return 0;
    }

    for (i, d) in sorted_dice[0..=(sorted_dice.len() - 4)].iter().enumerate() {
        if sorted_dice[i..(i + 4)] == array![i => d + (i as u32); 4] {
            return SMALL_STRAIGHT_SCORE;
        }
    }

    0
}

fn large_straight(dice: &[u32]) -> u32 {
    let mut sorted_dice: Vec<u32> = dice.to_vec();
    sorted_dice.sort_unstable();
    let init = sorted_dice[0];

    if sorted_dice == array![i => init + (i as u32); 5] {
        LARGE_STRAIGHT_SCORE
    } else {
        0
    }
}

fn yahtzee(dice: &[u32]) -> u32 {
    if dice == [dice[0]; Hand::DICE_NUM] {
        YAHTZEE_SCORE
    } else {
        0
    }
}

fn chance(dice: &[u32]) -> u32 {
    dice.iter().sum()
}

fn upper_section_scoring(dice: &[u32], spots: u32) -> u32 {
    (dice.iter().filter(|&d| *d == spots).count() as u32) * spots
}

#[cfg(test)]
mod tests {
    use crate::scoring::*;

    #[test]
    fn upper_section_scoring_test() {
        let dice: [u32; 5] = [1, 3, 3, 3, 6];
        assert_eq!(scoring(Boxes::Threes, &dice), 9);
    }

    #[test]
    fn three_of_a_kind_test() {
        let dice: [u32; 5] = [1, 3, 3, 3, 6];
        assert_eq!(scoring(Boxes::ThreeOfaAKind, &dice), 16);

        let dice: [u32; 5] = [1, 3, 3, 3, 3];
        assert_eq!(scoring(Boxes::ThreeOfaAKind, &dice), 13);

        let dice: [u32; 5] = [1, 2, 3, 3, 6];
        assert_eq!(scoring(Boxes::ThreeOfaAKind, &dice), 0);
    }

    #[test]
    fn four_of_a_kind_test() {
        let dice: [u32; 5] = [1, 3, 3, 3, 3];
        assert_eq!(scoring(Boxes::FourOfaAKind, &dice), 13);

        let dice: [u32; 5] = [3, 3, 3, 3, 3];
        assert_eq!(scoring(Boxes::FourOfaAKind, &dice), 15);

        let dice: [u32; 5] = [1, 2, 3, 3, 6];
        assert_eq!(scoring(Boxes::FourOfaAKind, &dice), 0);
    }

    #[test]
    fn full_house_test() {
        let dice: [u32; 5] = [4, 3, 3, 4, 3];
        assert_eq!(scoring(Boxes::FullHouse, &dice), FULL_HOUSE_SCORE);

        let dice: [u32; 5] = [5, 5, 5, 5, 5];
        assert_eq!(scoring(Boxes::FullHouse, &dice), 0);

        let dice: [u32; 5] = [4, 5, 1, 4, 5];
        assert_eq!(scoring(Boxes::FullHouse, &dice), 0);
    }

    #[test]
    fn small_straight_test() {
        let dice: [u32; 5] = [5, 3, 4, 4, 2];
        assert_eq!(scoring(Boxes::SmallStraight, &dice), SMALL_STRAIGHT_SCORE);

        let dice: [u32; 5] = [2, 4, 4, 6, 4];
        assert_eq!(scoring(Boxes::SmallStraight, &dice), 0);
    }

    #[test]
    fn large_straight_test() {
        let dice: [u32; 5] = [5, 3, 1, 4, 2];
        assert_eq!(scoring(Boxes::LargeStraight, &dice), LARGE_STRAIGHT_SCORE);

        let dice: [u32; 5] = [2, 3, 4, 1, 4];
        assert_eq!(scoring(Boxes::LargeStraight, &dice), 0);
    }

    #[test]
    fn yahtzee_test() {
        let dice: [u32; 5] = [4, 4, 4, 4, 4];
        assert_eq!(scoring(Boxes::Yahtzee, &dice), YAHTZEE_SCORE);

        let dice: [u32; 5] = [2, 3, 5, 1, 4];
        assert_eq!(scoring(Boxes::Yahtzee, &dice), 0);
    }

    #[test]
    fn chance_test() {
        let dice: [u32; 5] = [1, 3, 5, 2, 6];
        assert_eq!(scoring(Boxes::Chance, &dice), 17);
    }
}
