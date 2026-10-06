use serde::{Deserialize, Serialize};

use crate::input::Key;
use crate::layout::{Direction, SessionAction, Step, Vertical, WindowId};
use crate::view::ViewAction;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    View(ViewAction),
    Session(SessionCommand),
    Client(ClientAction),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionCommand {
    OpenWindow,
    CloseWindow,
    ConsumeOrExpel(Direction),
    MoveColumn(Direction),
    MoveWindow(Vertical),
    ToggleFloating,
    CycleWidth,
    ToggleFullWidth,
    StepWidth(Step),
    StepHeight(Step),
    ResetHeight,
}

impl SessionCommand {
    pub fn on_window(self, window: WindowId) -> Option<SessionAction> {
        Some(match self {
            SessionCommand::OpenWindow => return None,
            SessionCommand::ToggleFloating => SessionAction::ToggleFloating {
                window,
                after: None,
            },

            SessionCommand::CloseWindow => SessionAction::CloseWindow(window),
            SessionCommand::ConsumeOrExpel(direction) => {
                SessionAction::ConsumeOrExpel { window, direction }
            }
            SessionCommand::MoveColumn(direction) => {
                SessionAction::MoveColumn { window, direction }
            }
            SessionCommand::MoveWindow(direction) => {
                SessionAction::MoveWindow { window, direction }
            }

            SessionCommand::CycleWidth => SessionAction::CycleWidth(window),
            SessionCommand::ToggleFullWidth => SessionAction::ToggleFullWidth(window),
            SessionCommand::StepWidth(step) => SessionAction::StepWidth { window, step },
            SessionCommand::StepHeight(step) => SessionAction::StepHeight { window, step },
            SessionCommand::ResetHeight => SessionAction::ResetHeight(window),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientAction {
    Detach,
    SendPrefix,
    SendKey(Key),
}
