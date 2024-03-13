use serde::{Deserialize, Serialize};
use crate::reporting::{Col, Line};

#[derive(Debug, Serialize, Deserialize)]
pub enum Error {
    BadChar(Line, Col, char),
    Number(Line, Col, String),
    BadEndOfInput(Line, Col),
}

