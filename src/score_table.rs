use std::collections::HashMap;
use enum_iterator::*;

use crate::scoring::*;

struct Record {
	score: u32,
	filled: bool
}

impl Record {
	fn new() -> Self {
		return Record {
			score: 0,
			filled: false
		};
	}

	fn fill(&mut self, score: u32) {
		self.score = score;
		self.filled = true;
	}

	fn get_score(&self) -> u32 {
		return self.score;
	}

	fn is_filled(&self) -> bool {
		return self.filled;
	}
}

pub struct ScoreTable {
	table: HashMap<Boxes, Record>,
}

impl ScoreTable {
	pub fn new() -> Self {
		let mut table = HashMap::new();
		for b in all::<Boxes>() {
			table.insert(b, Record::new());
		}
		return ScoreTable {
			table: table
		}
	}

	pub fn get_score(&self, b: &Boxes) -> u32 {
		return self.table.get(b).unwrap().get_score();
	}

	pub fn is_filled(&self, b: &Boxes) -> bool {
		return self.table.get(b).unwrap().is_filled();
	}

	pub fn confirm_score(&mut self, b: &Boxes, score: u32) {
		self.table.get_mut(b).unwrap().fill(score);
	}

	pub fn remaining_boxes(&self) -> Vec<Boxes> {
		let mut rest: Vec<Boxes> = Vec::new();
		for b in all::<Boxes>() {
			if !self.table.get(&b).unwrap().is_filled() {
				rest.push(b);
			}
		}
		return rest;
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
		assert_eq!(record.get_score(), score);
		assert!(record.is_filled());
	}

	#[test]
	fn test_score_table() {
		let mut score_table = ScoreTable::new();
		let b = &Boxes::Chance;
		let score: u32 = 21;

		assert!(!score_table.is_filled(b));

		score_table.confirm_score(b, score);
		assert!(score_table.is_filled(b));
		assert_eq!(score_table.get_score(b), score);
	}
}