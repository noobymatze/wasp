use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize, Serializer};

pub type Line = usize;

pub type Col = usize;

#[derive(Debug, Eq, Ord, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub line: Line,
    pub col: Col,
}

#[derive(Debug, Eq, Ord, PartialOrd, PartialEq, Deserialize)]
pub struct Region {
    pub start: Position,
    pub end: Position,
}

impl Region {
    /// Returns a new `Region`.
    pub fn new(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self {
        Region {
            start: Position {
                line: start_line,
                col: start_col,
            },
            end: Position {
                line: end_line,
                col: end_col,
            },
        }
    }
}

impl From<(usize, usize, usize, usize)> for Region {
    fn from((start_line, start_col, end_line, end_col): (usize, usize, usize, usize)) -> Self {
        Region::new(start_line, start_col, end_line, end_col)
    }
}

impl From<(usize, usize, usize)> for Region {
    fn from((line, start_col, end_col): (usize, usize, usize)) -> Self {
        Region::new(line, start_col, line, end_col)
    }
}

impl From<(usize, usize)> for Region {
    fn from((line, col): (usize, usize)) -> Self {
        Region::new(line, col, line, col)
    }
}

impl Serialize for Region {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(4))?;
        seq.serialize_element(&self.start.line)?;
        seq.serialize_element(&self.start.col)?;
        seq.serialize_element(&self.end.line)?;
        seq.serialize_element(&self.end.col)?;
        seq.end()
    }
}
