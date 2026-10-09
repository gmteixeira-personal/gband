use std::collections::BTreeMap;

use gband_core::action::{Action, ClientAction, SessionCommand, Steps};
use gband_core::layout::{
    BandId, Direction, Proportion, SessionAction, Step, Vertical, WindowContent, WindowId,
};
use gband_core::view::ViewAction;
use mlua::{Lua, MetaMethod, Table, UserData, UserDataMethods, Value};

use crate::Side;
use crate::api::{self, Dispatch, require_loading};
use crate::callbacks::{self, CallbackId};
use crate::error::ConfigError;
use crate::owner;

pub struct BuiltinAction {
    pub name: &'static str,
    pub action: Action,
    pub desc: &'static str,
}

const fn builtin(name: &'static str, action: Action, desc: &'static str) -> BuiltinAction {
    BuiltinAction { name, action, desc }
}

pub const ACTIONS: [BuiltinAction; 31] = [
    builtin(
        "focus_column_left",
        Action::View(ViewAction::FocusLeft),
        "focus the column to the left",
    ),
    builtin(
        "focus_column_right",
        Action::View(ViewAction::FocusRight),
        "focus the column to the right",
    ),
    builtin(
        "focus_window_down",
        Action::View(ViewAction::FocusDown),
        "focus the window below",
    ),
    builtin(
        "focus_window_up",
        Action::View(ViewAction::FocusUp),
        "focus the window above",
    ),
    builtin(
        "focus_band_down",
        Action::View(ViewAction::BandDown),
        "view the band below",
    ),
    builtin(
        "focus_band_up",
        Action::View(ViewAction::BandUp),
        "view the band above",
    ),
    builtin(
        "center_column",
        Action::View(ViewAction::CenterColumn),
        "center the focused column",
    ),
    builtin(
        "switch_focus_floating_tiled",
        Action::View(ViewAction::SwitchLayer),
        "switch focus between floating and tiled windows",
    ),
    builtin(
        "minimize_window",
        Action::View(ViewAction::Minimize(None)),
        "minimize the focused floating window",
    ),
    builtin(
        "open_window",
        Action::Session(SessionCommand::OpenWindow),
        "open a window running the user's shell",
    ),
    builtin(
        "close_window",
        Action::Session(SessionCommand::CloseWindow),
        "close the window",
    ),
    builtin(
        "consume_or_expel_left",
        Action::Session(SessionCommand::ConsumeOrExpel(Direction::Left)),
        "consume or expel the window to the left",
    ),
    builtin(
        "consume_or_expel_right",
        Action::Session(SessionCommand::ConsumeOrExpel(Direction::Right)),
        "consume or expel the window to the right",
    ),
    builtin(
        "move_column_left",
        Action::Session(SessionCommand::MoveColumn(Direction::Left)),
        "move the column or floating window to the left",
    ),
    builtin(
        "move_column_right",
        Action::Session(SessionCommand::MoveColumn(Direction::Right)),
        "move the column or floating window to the right",
    ),
    builtin(
        "move_window_down",
        Action::Session(SessionCommand::MoveWindow(Vertical::Down)),
        "move the window down",
    ),
    builtin(
        "move_window_up",
        Action::Session(SessionCommand::MoveWindow(Vertical::Up)),
        "move the window up",
    ),
    builtin(
        "toggle_window_floating",
        Action::Session(SessionCommand::ToggleFloating),
        "float or tile the window",
    ),
    builtin(
        "cycle_column_width",
        Action::Session(SessionCommand::CycleWidth),
        "cycle the width of the window's column",
    ),
    builtin(
        "toggle_full_width",
        Action::Session(SessionCommand::ToggleFullWidth),
        "toggle full width of the window's column",
    ),
    builtin(
        "grow_column_width",
        Action::Session(SessionCommand::StepWidth {
            step: Step::Grow,
            by: Proportion::TENTH,
        }),
        "grow the width of the window's column",
    ),
    builtin(
        "shrink_column_width",
        Action::Session(SessionCommand::StepWidth {
            step: Step::Shrink,
            by: Proportion::TENTH,
        }),
        "shrink the width of the window's column",
    ),
    builtin(
        "grow_window_height",
        Action::Session(SessionCommand::StepHeight {
            step: Step::Grow,
            by: Proportion::TENTH,
        }),
        "grow the height of the window",
    ),
    builtin(
        "shrink_window_height",
        Action::Session(SessionCommand::StepHeight {
            step: Step::Shrink,
            by: Proportion::TENTH,
        }),
        "shrink the height of the window",
    ),
    builtin(
        "reset_window_height",
        Action::Session(SessionCommand::ResetHeight),
        "reset the height of the window",
    ),
    builtin("detach", Action::Client(ClientAction::Detach), "detach"),
    builtin(
        "send_prefix",
        Action::Client(ClientAction::SendPrefix),
        "send the prefix key to the focused window",
    ),
    builtin(
        "reload",
        Action::Client(ClientAction::Reload),
        "reload the configuration",
    ),
    builtin(
        "drag_window",
        Action::Client(ClientAction::DragWindow),
        "move the window with the mouse",
    ),
    builtin(
        "drag_resize_window",
        Action::Client(ClientAction::DragResize(None)),
        "resize the window with the mouse",
    ),
    builtin(
        "drag_band",
        Action::Client(ClientAction::DragBand),
        "slide the band or switch bands with the mouse",
    ),
];

const RESERVED: [&str; 2] = ["register", "list"];

pub(crate) enum LuaAction {
    Builtin {
        name: &'static str,
        action: Action,
    },
    Registered {
        name: String,
        callback: CallbackId,
    },
    Targeted {
        name: &'static str,
        command: SessionCommand,
    },
}

impl UserData for LuaAction {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(MetaMethod::Call, |lua, this, target: Value| match this {
            LuaAction::Builtin { action, .. } if target.is_nil() => {
                let action = action.stepped(crate::options::current(lua).steps);
                api::queue(lua, Dispatch::Action(action), "an action")
            }
            LuaAction::Builtin { name, action } => {
                if !api::in_callback(lua) {
                    return Err(api::outside_callback(lua, "an action"));
                }
                let entry = crate::control::targeted(lua, name, *action, &target)
                    .map_err(|message| ConfigError::raise(lua, message))?;
                api::queue(lua, entry, "an action")
            }
            LuaAction::Registered { callback, .. } => {
                if !api::in_callback(lua) {
                    return Err(api::outside_callback(lua, "an action"));
                }
                callbacks::run::<()>(lua, *callback, ())?;
                Ok(())
            }
            LuaAction::Targeted { name, command } => {
                if !api::in_callback(lua) {
                    return Err(api::outside_callback(lua, "an action"));
                }
                let entry = session_target(name, *command, &target)
                    .map_err(|message| ConfigError::raise(lua, message))?;
                api::queue(lua, entry, "an action")
            }
        });
    }
}

#[derive(Default)]
struct Registered(BTreeMap<String, Option<String>>);

fn integer(value: &Value) -> Option<u32> {
    match *value {
        Value::Integer(number) => u32::try_from(number).ok(),
        Value::Number(number) if number.fract() == 0.0 && number >= 0.0 => {
            u32::try_from(number as i64).ok()
        }
        _ => None,
    }
}

fn session_target(name: &str, command: SessionCommand, target: &Value) -> Result<Dispatch, String> {
    let Value::Table(target) = target else {
        return Err(format!(
            "`{name}` expects a target table naming `session`, found {}",
            target.type_name()
        ));
    };
    let opening = command == SessionCommand::OpenWindow;
    let allowed: &[&str] = match command {
        SessionCommand::OpenWindow => &["session", "band", "after", "program", "floating"],
        SessionCommand::ToggleFloating => &["session", "window", "after", "floating"],
        SessionCommand::StepWidth { .. } | SessionCommand::StepHeight { .. } => {
            &["session", "window", "step"]
        }
        _ => &["session", "window"],
    };
    for pair in target.pairs::<Value, Value>() {
        let (field, _) = pair.map_err(|error| error.to_string())?;
        let known = matches!(&field, Value::String(text) if allowed.iter().any(|allowed| *text == *allowed));
        if !known {
            let field = crate::control::field_name(&field);
            return Err(crate::removed::message(&field)
                .unwrap_or_else(|| format!("the target of `{name}` takes no field `{field}`")));
        }
    }
    let get = |field: &str| {
        target
            .get::<Value>(field)
            .map_err(|error| error.to_string())
    };
    let session = match get("session")? {
        Value::String(session) if !session.as_bytes().is_empty() => session.to_string_lossy(),
        Value::Nil => return Err(format!("the target of `{name}` must name a `session`")),
        other => {
            return Err(format!(
                "the `session` of `{name}` must be a session name, found {}",
                other.type_name()
            ));
        }
    };
    let number = |field: &str, required: bool| -> Result<Option<u32>, String> {
        match get(field)? {
            Value::Nil if required => Err(format!("the target of `{name}` must name a `{field}`")),
            Value::Nil => Ok(None),
            value => integer(&value).map(Some).ok_or_else(|| {
                format!(
                    "the `{field}` of `{name}` must be a number, found {}",
                    crate::control::field_name(&value)
                )
            }),
        }
    };
    let layer = |after: Option<WindowId>| -> Result<Option<bool>, String> {
        let floating = match get("floating")? {
            Value::Nil => None,
            Value::Boolean(floating) => Some(floating),
            other => {
                return Err(format!(
                    "the `floating` of `{name}` must be a boolean, found {}",
                    other.type_name()
                ));
            }
        };
        if floating == Some(true) && after.is_some() {
            return Err(format!(
                "the target of `{name}` cannot hold `after` with `floating = true`"
            ));
        }
        Ok(floating)
    };
    let action = if opening {
        let band = BandId(number("band", true)?.expect("required"));
        let after = number("after", false)?.map(WindowId);
        let floating = layer(after)?.unwrap_or(false);
        let program = match get("program")? {
            Value::Nil => None,
            Value::String(line) => Some(gband_core::layout::Program::CommandLine(
                line.to_string_lossy(),
            )),
            Value::Table(list) => match api::list_of_strings(&list) {
                Some(argv) if !argv.is_empty() => Some(gband_core::layout::Program::Argv(argv)),
                _ => {
                    return Err(format!(
                        "the `program` of `{name}` must be a non-empty list of strings"
                    ));
                }
            },
            other => {
                return Err(format!(
                    "the `program` of `{name}` must be a string or a list of strings, found {}",
                    other.type_name()
                ));
            }
        };
        SessionAction::OpenWindow {
            band,
            after,
            width: None,
            floating,
            focus: false,
            content: WindowContent::Program(program),
        }
    } else if command == SessionCommand::ToggleFloating {
        let window = WindowId(number("window", true)?.expect("required"));
        let after = number("after", false)?.map(WindowId);
        let floating = layer(after)?;
        SessionAction::ToggleFloating {
            window,
            after,
            floating,
        }
    } else {
        let window = WindowId(number("window", true)?.expect("required"));
        let by =
            crate::control::read_step(&get("step")?, command, name)?.unwrap_or(Proportion::TENTH);
        command
            .stepped(Steps {
                width: by,
                height: by,
            })
            .on_window(window)
            .expect("every command but open window names a window")
    };
    Ok(Dispatch::Targeted { session, action })
}

fn on_side(action: &BuiltinAction, side: Side) -> bool {
    side == Side::Client || !crate::sides::is_client_action(action.action)
}

pub(crate) fn install(lua: &Lua, gband: &Table, side: Side) -> mlua::Result<()> {
    lua.set_app_data(Registered::default());
    let actions = lua.create_table()?;
    for BuiltinAction { name, action, .. } in ACTIONS {
        match (side, action) {
            (Side::Client, _) => actions.set(name, LuaAction::Builtin { name, action })?,
            (Side::Server, Action::Session(command)) => {
                actions.set(name, LuaAction::Targeted { name, command })?
            }
            (Side::Server | Side::Test, _) => {}
        }
    }
    actions.set("register", lua.create_function(register)?)?;
    actions.set("list", lua.create_function(list)?)?;
    gband.set("action", actions)
}

fn register(lua: &Lua, (name, function, opts): (Value, Value, Value)) -> mlua::Result<Value> {
    let Value::String(name) = name else {
        return Err(ConfigError::raise(
            lua,
            "gband.action.register expects a name as a non-empty string",
        ));
    };
    let name = name.to_str()?.to_owned();
    if name.is_empty() {
        return Err(ConfigError::raise(
            lua,
            "gband.action.register expects a name as a non-empty string",
        ));
    }
    let Value::Function(function) = function else {
        return Err(ConfigError::raise(
            lua,
            format!("the action `{name}` must be a function"),
        ));
    };
    let desc = description(lua, &opts)?;
    let full = owner::full_name(lua, &name)?;
    let taken = RESERVED.contains(&full.as_str())
        || ACTIONS.iter().any(|action| action.name == full)
        || registered(lua).0.contains_key(&full);
    if taken {
        return Err(ConfigError::raise(
            lua,
            format!("the action name `{full}` is taken"),
        ));
    }
    require_loading(lua, "gband.action.register")?;
    let callback = callbacks::register(lua, function)?;
    registered(lua).0.insert(full.clone(), desc);
    let actions: Table = lua.globals().get::<Table>("gband")?.get("action")?;
    actions.set(
        full.as_str(),
        LuaAction::Registered {
            name: full.clone(),
            callback,
        },
    )?;
    actions.get(full)
}

fn registered(lua: &Lua) -> mlua::AppDataRefMut<'_, Registered> {
    lua.app_data_mut::<Registered>()
        .expect("registered actions are installed with the runtime")
}

fn description(lua: &Lua, opts: &Value) -> mlua::Result<Option<String>> {
    let invalid = || ConfigError::raise(lua, "gband.action.register expects `desc` to be a string");
    match opts {
        Value::Nil => Ok(None),
        Value::Table(opts) => match opts.get::<Value>("desc")? {
            Value::Nil => Ok(None),
            Value::String(desc) => Ok(Some(desc.to_str()?.to_owned())),
            _ => Err(invalid()),
        },
        _ => Err(invalid()),
    }
}

fn list(lua: &Lua, (): ()) -> mlua::Result<Table> {
    let side = crate::runtime::side(lua);
    let mut entries: Vec<(String, Option<String>)> = ACTIONS
        .iter()
        .filter(|action| on_side(action, side))
        .map(|action| (action.name.to_owned(), Some(action.desc.to_owned())))
        .collect();
    entries.extend(
        registered(lua)
            .0
            .iter()
            .map(|(name, desc)| (name.clone(), desc.clone())),
    );
    entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    let list = lua.create_table()?;
    for (name, desc) in entries {
        let entry = lua.create_table()?;
        entry.set("name", name)?;
        entry.set("desc", desc)?;
        list.push(entry)?;
    }
    Ok(list)
}
