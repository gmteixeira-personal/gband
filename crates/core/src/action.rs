use serde::{Deserialize, Serialize};

use crate::input::Key;
use crate::layout::{Direction, Proportion, SessionAction, Step, Vertical, WindowId};
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
    StepWidth { step: Step, by: Proportion },
    StepHeight { step: Step, by: Proportion },
    ResetHeight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Steps {
    pub width: Proportion,
    pub height: Proportion,
}

impl Default for Steps {
    fn default() -> Self {
        Self {
            width: Proportion::TENTH,
            height: Proportion::TENTH,
        }
    }
}

impl Action {
    pub fn stepped(self, steps: Steps) -> Self {
        match self {
            Action::Session(command) => Action::Session(command.stepped(steps)),
            action => action,
        }
    }
}

impl SessionCommand {
    pub fn stepped(self, steps: Steps) -> Self {
        match self {
            SessionCommand::StepWidth { step, .. } => SessionCommand::StepWidth {
                step,
                by: steps.width,
            },
            SessionCommand::StepHeight { step, .. } => SessionCommand::StepHeight {
                step,
                by: steps.height,
            },
            command => command,
        }
    }

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
            SessionCommand::StepWidth { step, by } => SessionAction::StepWidth { window, step, by },
            SessionCommand::StepHeight { step, by } => {
                SessionAction::StepHeight { window, step, by }
            }
            SessionCommand::ResetHeight => SessionAction::ResetHeight(window),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientAction {
    Detach,
    SendPrefix,
    SendKey(Key),
    DragWindow,
    DragResize,
    DragBand,
}
