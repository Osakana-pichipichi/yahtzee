use crate::hand::{Die, Hand};
use crate::scoring::Boxes;
use anyhow::{bail, Result};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RecordError {
    #[error("Try to fill the filled record")]
    TryToFillFilledRecord,
}

#[derive(Clone)]
struct Record {
    score: Option<u32>,
}

impl Record {
    fn new() -> Self {
        Record { score: None }
    }

    fn new_with_score(score: u32) -> Self {
        Record { score: Some(score) }
    }

    fn fill(&mut self, score: u32) -> Result<()> {
        if self.is_filled() {
            bail!(RecordError::TryToFillFilledRecord);
        }

        self.score = Some(score);

        Ok(())
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
    pub const BONUS_THRESHOLD: u32 = {
        /* This means (1 + 2 + 3 + 4 + 5 + 6) * 3. */
        let mut sum = 0;
        let mut i = 1;
        while i <= Die::PIPS.len() {
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
        self.table[&b].get_score()
    }

    pub fn has_score_in(&self, b: Boxes) -> bool {
        self.table[&b].is_filled()
    }

    pub fn has_all_scores(&self) -> bool {
        self.table.iter().all(|(.., row)| row.is_filled())
    }

    pub fn get_num_filled_scores(&self) -> usize {
        self.table
            .iter()
            .filter(|(.., row)| row.is_filled())
            .count()
    }

    pub fn confirm_score(&mut self, b: Boxes, score: u32) -> Result<()> {
        self.table.get_mut(&b).unwrap().fill(score)
    }

    pub fn get_total_upper_score(&self) -> u32 {
        ScoreTable::BONUS_TARGETS
            .iter()
            .map(|&(b, ..)| self.get_score(b).unwrap_or(0))
            .sum()
    }

    pub fn get_total_upper_score_if_filled_by(&self, b: Boxes, score: u32) -> u32 {
        let mut dummy_table = HashMap::new();
        dummy_table.clone_from(&self.table);

        let mut dummy_score_table = ScoreTable { table: dummy_table };
        if !dummy_score_table.has_score_in(b) {
            dummy_score_table
                .table
                .insert(b, Record::new_with_score(score));
        }

        dummy_score_table.get_total_upper_score()
    }

    pub fn calculate_bonus(&self) -> Option<u32> {
        let current = self.get_total_upper_score();
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

    pub fn calculate_bonus_if_filled_by(&self, b: Boxes, score: u32) -> Option<u32> {
        let mut dummy_table = HashMap::new();
        dummy_table.clone_from(&self.table);

        let mut dummy_score_table = ScoreTable { table: dummy_table };
        if !dummy_score_table.has_score_in(b) {
            dummy_score_table
                .table
                .insert(b, Record::new_with_score(score));
        }

        dummy_score_table.calculate_bonus()
    }

    pub fn get_total_score(&self) -> u32 {
        let sum: u32 = self
            .table
            .keys()
            .map(|&b| self.get_score(b).unwrap_or(0))
            .sum();

        sum + self.calculate_bonus().unwrap_or(0)
    }

    pub fn get_total_score_if_filled_by(&self, b: Boxes, score: u32) -> u32 {
        let mut dummy_table = HashMap::new();
        dummy_table.clone_from(&self.table);

        let mut dummy_score_table = ScoreTable { table: dummy_table };
        if !dummy_score_table.has_score_in(b) {
            dummy_score_table
                .table
                .insert(b, Record::new_with_score(score));
        }

        dummy_score_table.get_total_score()
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
        record.fill(score).unwrap();
        assert_eq!(record.get_score(), &Some(score));
        assert!(record.is_filled());
    }

    #[test]
    fn test_score_table() {
        let mut score_table = ScoreTable::new();
        let b = Boxes::Chance;
        let score: u32 = 21;

        assert!(!score_table.has_score_in(b));

        score_table.confirm_score(b, score).unwrap();
        assert!(score_table.has_score_in(b));
        assert_eq!(score_table.get_score(b), &Some(score));
    }

    #[test]
    fn test_get_total_upper_score() {
        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS.iter() {
            score_table.confirm_score(b, p * 3).unwrap();
        }

        assert_eq!(
            score_table.get_total_upper_score(),
            ScoreTable::BONUS_THRESHOLD
        );

        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS.iter() {
            score_table.confirm_score(b, p * 2).unwrap();
        }

        assert_eq!(
            score_table.get_total_upper_score(),
            Die::PIPS.iter().sum::<u32>() * 2
        );

        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS[1..].iter() {
            score_table.confirm_score(b, p * 3).unwrap();
        }

        assert_eq!(
            score_table.get_total_upper_score(),
            Die::PIPS[1..].iter().sum::<u32>() * 3
        );

        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS[1..].iter() {
            score_table.confirm_score(b, p * 2).unwrap();
        }

        assert_eq!(
            score_table.get_total_upper_score(),
            Die::PIPS[1..].iter().sum::<u32>() * 2
        );
    }

    #[test]
    fn test_get_total_upper_score_if_filled_by() {
        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS.iter() {
            score_table.confirm_score(b, p * 3).unwrap();
        }

        assert_eq!(
            score_table.get_total_upper_score_if_filled_by(Boxes::Chance, 20),
            ScoreTable::BONUS_THRESHOLD
        );

        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS.iter() {
            score_table.confirm_score(b, p * 2).unwrap();
        }

        assert_eq!(
            score_table.get_total_upper_score_if_filled_by(Boxes::Chance, 20),
            Die::PIPS.iter().sum::<u32>() * 2
        );

        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS[1..].iter() {
            score_table.confirm_score(b, p * 3).unwrap();
        }

        let (b, p) = ScoreTable::BONUS_TARGETS[0];
        assert_eq!(
            score_table.get_total_upper_score_if_filled_by(b, p * 3),
            Die::PIPS.iter().sum::<u32>() * 3
        );
        assert_eq!(
            score_table.get_total_upper_score_if_filled_by(b, p * 2),
            Die::PIPS[1..].iter().sum::<u32>() * 3 + 2
        );
    }

    #[test]
    fn test_calculate_bonus() {
        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS.iter() {
            score_table.confirm_score(b, p * 3).unwrap();
        }

        assert_eq!(score_table.calculate_bonus(), Some(ScoreTable::BONUS_POINT));

        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS.iter() {
            score_table.confirm_score(b, p * 2).unwrap();
        }

        assert_eq!(score_table.calculate_bonus(), Some(0));

        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS[1..].iter() {
            score_table.confirm_score(b, p * 3).unwrap();
        }

        assert_eq!(score_table.calculate_bonus(), None);

        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS[1..].iter() {
            score_table.confirm_score(b, p * 2).unwrap();
        }

        assert_eq!(score_table.calculate_bonus(), Some(0));
    }

    #[test]
    fn test_calculate_bonus_if_filled_by() {
        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS.iter() {
            score_table.confirm_score(b, p * 3).unwrap();
        }

        let result = score_table.calculate_bonus_if_filled_by(Boxes::Chance, 20);
        assert_eq!(result, Some(ScoreTable::BONUS_POINT));

        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS.iter() {
            score_table.confirm_score(b, p * 2).unwrap();
        }

        let result = score_table.calculate_bonus_if_filled_by(Boxes::Chance, 20);
        assert_eq!(result, Some(0));

        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS[1..].iter() {
            score_table.confirm_score(b, p * 3).unwrap();
        }

        let (b, p) = ScoreTable::BONUS_TARGETS[0];
        let result = score_table.calculate_bonus_if_filled_by(b, p * 2);
        assert_eq!(result, Some(0));
        let result = score_table.calculate_bonus_if_filled_by(b, p * 3);
        assert_eq!(result, Some(ScoreTable::BONUS_POINT));
    }

    #[test]
    fn test_get_total_score() {
        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS.iter() {
            score_table.confirm_score(b, p * 3).unwrap();
        }

        assert_eq!(
            score_table.get_total_score(),
            ScoreTable::BONUS_POINT + ScoreTable::BONUS_THRESHOLD
        );

        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS.iter() {
            score_table.confirm_score(b, p * 2).unwrap();
        }

        assert_eq!(
            score_table.get_total_score(),
            Die::PIPS.iter().sum::<u32>() * 2
        );

        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS[1..].iter() {
            score_table.confirm_score(b, p * 3).unwrap();
        }

        assert_eq!(
            score_table.get_total_score(),
            Die::PIPS[1..].iter().sum::<u32>() * 3
        );

        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS[1..].iter() {
            score_table.confirm_score(b, p * 2).unwrap();
        }

        assert_eq!(
            score_table.get_total_score(),
            Die::PIPS[1..].iter().sum::<u32>() * 2
        );
    }

    #[test]
    fn test_get_total_score_if_filled_by() {
        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS.iter() {
            score_table.confirm_score(b, p * 3).unwrap();
        }

        assert_eq!(
            score_table.get_total_score_if_filled_by(Boxes::Chance, 20),
            ScoreTable::BONUS_POINT + ScoreTable::BONUS_THRESHOLD + 20
        );

        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS.iter() {
            score_table.confirm_score(b, p * 2).unwrap();
        }

        assert_eq!(
            score_table.get_total_score_if_filled_by(Boxes::Chance, 20),
            Die::PIPS.iter().sum::<u32>() * 2 + 20
        );

        let mut score_table = ScoreTable::new();
        for &(b, p) in ScoreTable::BONUS_TARGETS[1..].iter() {
            score_table.confirm_score(b, p * 3).unwrap();
        }

        let (b, p) = ScoreTable::BONUS_TARGETS[0];
        assert_eq!(
            score_table.get_total_score_if_filled_by(b, p * 3),
            Die::PIPS.iter().sum::<u32>() * 3 + ScoreTable::BONUS_POINT
        );
        assert_eq!(
            score_table.get_total_score_if_filled_by(b, p * 2),
            Die::PIPS[1..].iter().sum::<u32>() * 3 + 2
        );
    }
}
