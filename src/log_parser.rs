use crate::Result;

pub struct LogParser {
    total_win: u64,
    total_round: u64,

    curser: usize,
}

impl LogParser {
    pub fn parse_single_line(&mut self, line: &[u8]) -> Result<()> {
        self.total_round += 1; // increment the round
        self.curser = 0;

        let line = line.trim_ascii();

        let round_no = parse_round_number(line);

        Ok(())
    }

    pub fn new() -> Self {
        Self {
            total_win: 0,
            total_round: 0,
        }
    }
}
