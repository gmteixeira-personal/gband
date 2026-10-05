use serde::{Deserialize, Serialize};

use crate::input::Key;
use crate::layout::{Direction, Step};
use crate::view::ViewAction;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    View(ViewAction),
    Session(SessionCommand),
    Client(ClientAction),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionCommand {
    OpenPane,
    ClosePane,
    ConsumeOrExpel(Direction),
    CycleWidth,
    ToggleFullWidth,
    StepWidth(Step),
    StepHeight(Step),
    ResetHeight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientAction {
    Detach,
    SendPrefix,
    SendKey(Key),
}
