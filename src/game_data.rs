use crate::score_table::ScoreTable;

pub struct GameData {
    num_players: usize,
    scores: Vec<ScoreTable>,
}

impl GameData {
    pub fn new(num_players: usize) -> Self {
        Self {
            num_players,
            scores: (0..num_players).map(|_| ScoreTable::new()).collect(),
        }
    }

    pub fn get_score_table(&self, player_id: usize) -> &ScoreTable {
        if player_id >= self.num_players {
            panic!(
                "Unexpected player_id: {} (total players: {})",
                player_id, self.num_players
            );
        };

        &self.scores[player_id]
    }

    pub fn get_mut_score_table(&mut self, player_id: usize) -> &mut ScoreTable {
        if player_id >= self.num_players {
            panic!(
                "Unexpected player_id: {} (total players: {})",
                player_id, self.num_players
            );
        };

        &mut self.scores[player_id]
    }

    pub fn get_num_players(&self) -> usize {
        self.num_players
    }

    pub fn current_player_id(&self) -> usize {
        let pid_to_filled_scores: Vec<_> = self
            .scores
            .iter()
            .map(|e| e.get_num_filled_scores())
            .collect();
        (1..self.get_num_players())
            .find(|&i| pid_to_filled_scores[i - 1] > pid_to_filled_scores[i])
            .unwrap_or(0)
    }
}
