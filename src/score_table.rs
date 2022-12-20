use crate::hand::Hand;
use crate::scoring::Boxes;
use std::collections::HashMap;

struct Record {
    score: Option<u32>,
}

impl Record {
    fn new() -> Self {
        Record { score: None }
    }

    fn fill(&mut self, score: u32) {
        if !self.is_filled() {
            self.score = Some(score);
        } else {
            panic!("The record to be filled is already filled.");
        }
    }

    fn get_score(&self) -> &Option<u32> {
        &self.score
    }

    fn is_filled(&self) -> bool {
        matches!(self.score, Some(..))
    }
}

pub struct ScoreTable {
    table: HashMap<Boxes, Record>,
}

impl Default for ScoreTable {
    fn default() -> Self {
        Self::new()
    }
}

impl ScoreTable {
    const BONUS_TARGETS: [(Boxes, u32); 6] = {
        [
            (Boxes::Aces, 1),
            (Boxes::Twos, 2),
            (Boxes::Threes, 3),
            (Boxes::Fours, 4),
            (Boxes::Fives, 5),
            (Boxes::Sixes, 6),
        ]
    };
    const BONUS_THRESHOLD: u32 = {
        /* This means (1 + 2 + 3 + 4 + 5 + 6) * 3. */
        let mut sum = 0;
        let mut i = 1;
        while i <= Hand::PIPS.len() {
            sum += i;
            i += 1;
        }
        (sum * 3) as u32
    };
    const BONUS_POINT: u32 = 35;

    pub fn new() -> Self {
        ScoreTable {
            table: HashMap::from_iter(enum_iterator::all::<Boxes>().map(|b| (b, Record::new()))),
        }
    }

    pub fn get_score(&self, b: Boxes) -> &Option<u32> {
        self.table.get(&b).unwrap().get_score()
    }

    pub fn has_score_in(&self, b: Boxes) -> bool {
        self.table.get(&b).unwrap().is_filled()
    }

    pub fn has_all_scores(&self) -> bool {
        self.table
            .iter()
            .map(|(.., row)| row.is_filled())
            .all(|x| x)
    }

    pub fn confirm_score(&mut self, b: Boxes, score: u32) {
        self.table.get_mut(&b).unwrap().fill(score);
    }

    pub fn remaining_boxes(&self) -> Vec<Boxes> {
        enum_iterator::all::<Boxes>()
            .filter(|&b| self.has_score_in(b))
            .collect()
    }

    pub fn calculate_bonus(&self) -> Option<u32> {
        let current: u32 = ScoreTable::BONUS_TARGETS
            .iter()
            .map(|&(b, ..)| match self.get_score(b) {
                Some(score) => score,
                None => &0,
            })
            .sum();
        let max: u32 = ScoreTable::BONUS_TARGETS
            .iter()
            .map(|&(b, p)| {
                if self.has_score_in(b) {
                    self.get_score(b).unwrap()
                } else {
                    p * (Hand::DICE_NUM as u32)
                }
            })
            .sum();

        if current >= ScoreTable::BONUS_THRESHOLD {
            Some(ScoreTable::BONUS_POINT)
        } else if max >= ScoreTable::BONUS_THRESHOLD {
            None
        } else {
            Some(0)
        }
    }

    pub fn get_total_score(&self) -> u32 {
        let sum: u32 = self
            .table
            .keys()
            .map(|&b| match self.get_score(b) {
                Some(score) => score,
                None => &0,
            })
            .sum();

        sum + if let Some(x) = self.calculate_bonus() {
            x
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::score_table::*;

    #[test]
    fn test_record() {
        let mut record = Record::new();
        assert!(!record.is_filled());

        let score: u32 = 32;
        record.fill(score);
        assert_eq!(record.get_score(), &Some(score));
        assert!(record.is_filled());
    }

    #[test]
    fn test_score_table() {
        let mut score_table = ScoreTable::new();
        let b = Boxes::Chance;
        let score: u32 = 21;

        assert!(!score_table.has_score_in(b));

        score_table.confirm_score(b, score);
        assert!(score_table.has_score_in(b));
        assert_eq!(score_table.get_score(b), &Some(score));
    }

    #[test]
    fn test_calculate_bonus() {
        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS.iter() {
            score_table.confirm_score(b, p * 3);
        }

        assert_eq!(score_table.calculate_bonus(), Some(ScoreTable::BONUS_POINT));

        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS.iter() {
            score_table.confirm_score(b, p * 2);
        }

        assert_eq!(score_table.calculate_bonus(), Some(0));

        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS[1..].iter() {
            score_table.confirm_score(b, p * 3);
        }

        assert_eq!(score_table.calculate_bonus(), None);

        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS[1..].iter() {
            score_table.confirm_score(b, p * 2);
        }

        assert_eq!(score_table.calculate_bonus(), Some(0));
    }

    #[test]
    fn test_get_total_score() {
        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS.iter() {
            score_table.confirm_score(b, p * 3);
        }

        assert_eq!(
            score_table.get_total_score(),
            ScoreTable::BONUM_POINT + ScoreTable::BONUS_THRESHOLD
        );

        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS.iter() {
            score_table.confirm_score(b, p * 2);
        }

        assert_eq!(
            score_table.get_total_score(),
            Hand::PIPS.iter().sum::<u32>() * 2
        );

        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS[1..].iter() {
            score_table.confirm_score(b, p * 3);
        }

        assert_eq!(
            score_table.get_total_score(),
            Hand::PIPS[1..].iter().sum::<u32>() * 3
        );

        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS[1..].iter() {
            score_table.confirm_score(b, p * 2);
        }

        assert_eq!(
            score_table.get_total_score(),
            Hand::PIPS[1..].iter().sum::<u32>() * 2
        );
    }
}
