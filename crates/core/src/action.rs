use serde::{Deserialize, Serialize};

use crate::input::Key;
use crate::layout::{Direction, PaneId, SessionAction, Step};
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

impl SessionCommand {
    pub fn on_pane(self, pane: PaneId) -> Option<SessionAction> {
        Some(match self {
            SessionCommand::OpenPane => return None,
            SessionCommand::ClosePane => SessionAction::ClosePane(pane),
            SessionCommand::ConsumeOrExpel(direction) => {
                SessionAction::ConsumeOrExpel { pane, direction }
            }
            SessionCommand::CycleWidth => SessionAction::CycleWidth(pane),
            SessionCommand::ToggleFullWidth => SessionAction::ToggleFullWidth(pane),
            SessionCommand::StepWidth(step) => SessionAction::StepWidth { pane, step },
            SessionCommand::StepHeight(step) => SessionAction::StepHeight { pane, step },
            SessionCommand::ResetHeight => SessionAction::ResetHeight(pane),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientAction {
    Detach,
    SendPrefix,
    SendKey(Key),
}
