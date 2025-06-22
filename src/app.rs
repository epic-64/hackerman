use strum_macros::{Display, EnumIter};

#[derive(EnumIter, Display)]
pub enum Game {
    Home,
    Counter,
    DinoJump,
}