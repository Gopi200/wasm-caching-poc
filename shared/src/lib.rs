use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Action {
    Update(Direction),
    Get,
}

#[derive(Serialize, Deserialize)]
pub enum Direction {
    Increment,
    Decrement,
}
