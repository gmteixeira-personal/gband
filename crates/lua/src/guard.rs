use std::path::PathBuf;

use mlua::debug::Debug;
use mlua::{HookTriggers, Lua, Thread, VmState};

use crate::error::ConfigError;
use crate::owner;
use crate::runtime::is_loading;

pub(crate) const LIMIT: &str = "instruction limit exceeded";
const STEP: u32 = 10_000;

pub(crate) struct Budget {
    limit: u64,
    left: u64,
    depth: usize,
    exhausted: Option<ConfigError>,
    tightened: Vec<Thread>,
}

fn every(instructions: u32) -> HookTriggers {
    HookTriggers::new().every_nth_instruction(instructions)
}

#[derive(Default)]
pub(crate) struct Sources(pub Vec<PathBuf>);

#[derive(Default)]
pub(crate) struct Reported(Vec<ConfigError>);

#[derive(Debug)]
pub(crate) struct Failure {
    pub error: ConfigError,
    pub limit: bool,
}

pub(crate) fn install(lua: &Lua, limit: u64) -> mlua::Result<()> {
    lua.set_app_data(Budget {
        limit,
        left: limit,
        depth: 0,
        exhausted: None,
        tightened: Vec::new(),
    });
    lua.set_app_data(Sources::default());
    lua.set_app_data(Reported::default());
    lua.set_global_hook(every(STEP), tick)
}

fn budget(lua: &Lua) -> mlua::AppDataRefMut<'_, Budget> {
    lua.app_data_mut::<Budget>()
        .expect("the budget is installed with the runtime")
}

fn tick(lua: &Lua, debug: &Debug) -> mlua::Result<VmState> {
    let mut budget = budget(lua);
    if budget.depth == 0 {
        return Ok(VmState::Continue);
    }
    budget.left = budget.left.saturating_sub(u64::from(STEP));
    if budget.left > 0 {
        return Ok(VmState::Continue);
    }
    let error = budget.exhausted.get_or_insert_with(|| {
        let location = debug.source().source.and_then(|source| {
            let path = source.strip_prefix('@')?;
            let line = u32::try_from(debug.current_line()?).ok()?;
            Some((PathBuf::from(path), line))
        });
        ConfigError {
            plugin: owner::current(lua),
            location,
            message: LIMIT.to_owned(),
        }
    });
    let error = mlua::Error::external(error.clone());
    let thread = lua.current_thread();
    let tighten = !budget
        .tightened
        .iter()
        .any(|tightened| tightened.to_pointer() == thread.to_pointer());
    if tighten {
        budget.tightened.push(thread.clone());
    }
    drop(budget);
    if tighten {
        thread.set_hook(every(1), tick)?;
    }
    Err(error)
}

pub(crate) fn sources(lua: &Lua) -> Vec<PathBuf> {
    lua.app_data_ref::<Sources>()
        .map(|sources| sources.0.clone())
        .unwrap_or_default()
}

pub(crate) fn add_source(lua: &Lua, path: PathBuf) {
    if let Some(mut sources) = lua.app_data_mut::<Sources>() {
        sources.0.push(path);
    }
}

pub(crate) fn run<T>(
    lua: &Lua,
    owner: Option<String>,
    f: impl FnOnce() -> mlua::Result<T>,
) -> mlua::Result<Result<T, Failure>> {
    {
        let mut budget = budget(lua);
        if budget.depth == 0 {
            budget.left = budget.limit;
            budget.exhausted = None;
        }
        budget.depth += 1;
    }
    owner::push(lua, owner.clone());
    let result = f();
    owner::pop(lua);
    let exhausted = {
        let mut budget = budget(lua);
        budget.depth -= 1;
        if budget.depth > 0 {
            if let Some(error) = &budget.exhausted {
                return Err(mlua::Error::external(error.clone()));
            }
            None
        } else {
            budget
                .exhausted
                .take()
                .map(|error| (error, std::mem::take(&mut budget.tightened)))
        }
    };
    if let Some((error, tightened)) = exhausted {
        for thread in tightened {
            thread.set_hook(every(STEP), tick)?;
        }
        lua.set_global_hook(every(STEP), tick)?;
        return Ok(Err(Failure { error, limit: true }));
    }
    Ok(result.map_err(|error| {
        let mut error = ConfigError::from_lua(&error, &sources(lua));
        error.plugin = owner;
        Failure {
            error,
            limit: false,
        }
    }))
}

pub(crate) fn report(lua: &Lua, failure: Failure) {
    if let Some(plugin) = &failure.error.plugin
        && (failure.limit || is_loading(lua))
    {
        owner::mark_failed(lua, plugin);
    }
    push(lua, failure.error);
}

pub(crate) fn push(lua: &Lua, error: ConfigError) {
    if let Some(mut reported) = lua.app_data_mut::<Reported>() {
        reported.0.push(error);
    }
}

pub(crate) fn drain(lua: &Lua) -> Vec<ConfigError> {
    lua.app_data_mut::<Reported>()
        .map(|mut reported| std::mem::take(&mut reported.0))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use mlua::Function;

    use super::*;
    use crate::owner::Owners;
    use crate::runtime::Phase;

    fn state(limit: u64) -> Lua {
        let lua = Lua::new();
        lua.set_app_data(Owners::default());
        lua.set_app_data(Phase { loading: false });
        install(&lua, limit).unwrap();
        lua
    }

    fn guarded(lua: &Lua, owner: Option<&str>, source: &str) -> Result<(), Failure> {
        run(lua, owner.map(str::to_owned), || {
            lua.load(source).set_name("@/p/x.lua").exec()
        })
        .unwrap()
    }

    #[test]
    fn endless_loop_stops_under_a_small_budget() {
        let lua = state(50_000);
        let failure = guarded(&lua, Some("hello"), "while true do end").unwrap_err();
        assert!(failure.limit);
        assert_eq!(failure.error.plugin.as_deref(), Some("hello"));
        assert_eq!(failure.error.message, LIMIT);
        assert_eq!(failure.error.location, Some((PathBuf::from("/p/x.lua"), 1)));
    }

    #[test]
    fn budget_resets_for_each_outermost_run() {
        let lua = state(50_000);
        let short = "for i = 1, 5000 do end";
        for _ in 0..20 {
            guarded(&lua, None, short).unwrap();
        }
    }

    #[test]
    fn pcall_does_not_swallow_the_stop() {
        let lua = state(50_000);
        let failure = guarded(
            &lua,
            Some("hello"),
            "while true do pcall(function() while true do end end) end",
        )
        .unwrap_err();
        assert!(failure.limit);
    }

    #[test]
    fn pcall_that_returns_is_still_stopped() {
        let lua = state(50_000);
        let failure = guarded(&lua, None, "pcall(function() while true do end end)").unwrap_err();
        assert!(failure.limit);
    }

    #[test]
    fn nested_stop_blames_the_inner_owner() {
        let lua = state(50_000);
        let inner = lua
            .create_function(|lua, function: Function| {
                match run(lua, Some("inner".to_owned()), || function.call::<()>(()))? {
                    Ok(()) => Ok(()),
                    Err(failure) => {
                        report(lua, failure);
                        Ok(())
                    }
                }
            })
            .unwrap();
        lua.globals().set("inner", inner).unwrap();
        let failure = guarded(
            &lua,
            Some("outer"),
            "inner(function() while true do end end)\nwhile true do end",
        )
        .unwrap_err();
        assert!(failure.limit);
        assert_eq!(failure.error.plugin.as_deref(), Some("inner"));
        assert!(drain(&lua).is_empty());
    }

    #[test]
    fn errors_name_the_owner() {
        let lua = state(50_000);
        let failure = guarded(&lua, Some("hello"), "\n\nerror('boom')").unwrap_err();
        assert!(!failure.limit);
        assert_eq!(failure.error.to_string(), "hello: /p/x.lua:3: boom");
    }

    #[test]
    fn code_outside_a_run_is_not_limited() {
        let lua = state(10_000);
        lua.load("for i = 1, 100000 do end").exec().unwrap();
    }
}
