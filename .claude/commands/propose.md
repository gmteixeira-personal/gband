---
description: Propose an OpenSpec change on the integration branch and publish its coordination declaration
model: opus
effort: high
allowed-tools: Bash(git:*), Bash(openspec:*), Bash(id:*), Read, Write, Edit, Glob, Grep, Skill
argument-hint: "[change-id]"
---
<!-- Installed by /flow:install from ~/.claude/commands/flow/propose.md. Edit the source and reinstall: the next install overwrites a local edit. -->

Read `.claude/commands/flow-conventions.md` and follow it. The terms below — `<base>`, the main worktree, **Synchronise**, the archive record, the coordination block, the login — mean what it defines.

Propose a new change, declare what it depends on and which files it expects to touch, and publish it on `<base>` so every agent can read it.

**Input**: `$ARGUMENTS` is optionally a change id, for example `add-contact-filters`. If omitted, infer it from the conversation or ask.

**Planning boundary**: this command writes planning artifacts only. Write nothing outside `openspec/changes/<id>/` and do not start implementing, even when the request asks for it. Implementation starts with `/implement` in a later session.

## Steps

### 1. Guard the position

```sh
git rev-parse --git-dir
git rev-parse --git-common-dir
git branch --show-current
```

Refuse unless the two paths are equal and the branch is `<base>`. Name the worktree and branch found, and say the command runs in the main worktree on `<base>`. Change nothing.

### 2. Synchronise

Run **Synchronise** in full, pull included, and follow its refusal on a diverged `<base>`.

Then refuse an id that is already taken: one with an open directory under `openspec/changes/`, or one with an archive record on `origin/<base>`. A reused archived id makes every later command read the new change as already finished.

### 3. Draft the change

Invoke the `openspec-propose` skill with the change id, passing no `--store`. Let it write `proposal.md`, the delta specs, `design.md` and `tasks.md` under `openspec/changes/<id>/`. Then confirm the artifacts are valid:

```sh
openspec validate <id> --strict
```

A validation failure is fixed in the artifacts before going on.

### 4. Declare coordination

Append the coordination block to the end of `openspec/changes/<id>/proposal.md`, in the order the conventions give: `### Author`, `### Depends On`, then `### Expected Files` last.

**Author** — exactly one entry: the login of the account running this command. Write it without asking.

**Depends On** — ask the user which open or archived changes this one builds on. Default to the single entry `none` when they name none. A dependency means this change cannot start until that change is archived, so declare only a real one.

**Expected Files** — derive from the drafted `tasks.md` and `design.md`: every file the tasks create or modify, plus the directory prefix of any area the design restructures. Entries are repository-relative, never absolute, and never globs.

- **Each entry is the narrowest path or prefix that contains the change's work.** A change touching one file declares that file. A directory is declared only when the change writes across it. Never write the enclosing project or the parent directory in place of the narrower entry. The overlap check is a prefix test, so a widened entry contests every path beneath it, and the refusal that follows names a path neither change would have written.
- **Always include `openspec/changes/<id>/`.** Implementing a change ticks its own `tasks.md`, so every change touches its own planning directory. Leaving it out makes `/ready` report drift on every change. The entry never overlaps another change's.
- **A narrow declaration is evidence, not a guarantee.** A collision over a path neither change declared surfaces as a merge conflict at integration, which `flow/conflict/<id>` exists for. The trade is deliberate: a certain refusal over a path nobody will write is exchanged for a possible conflict over one somebody did.

Both lists carry at least one entry. A change that touches no file is not a change.

Run `openspec validate <id> --strict` again after appending the block.

### 5. Validate the dependencies

**Run Synchronise again, in full, before checking anything.** Drafting takes minutes, and another machine may have proposed or archived a change meanwhile. A change proposed since step 2 is otherwise invisible, so naming it in `Depends On` would be refused as unknown.

Check every entry of `Depends On` other than `none` against the open changes, `ls openspec/changes/`, and the archive records on `origin/<base>`:

- **Unknown dependency** — an entry matching neither an open change nor an archive record. Refuse, naming it.
- **Cycle** — read the `### Depends On` list of every open change and walk the graph from this change. If the walk reaches this change again, refuse and report the chain that closes it, `a -> b -> c -> a`. Every change in a cycle could never start.

On either refusal, delete the drafted `openspec/changes/<id>/` directory, which this command created, and stop. Delete nothing else.

### 6. Commit and push

Stage every file under the change directory by name, then commit with an explicit pathspec:

```sh
git status --porcelain --untracked-files=all -- "openspec/changes/<id>/"
git add -- <each listed path>
git commit -m "<message>" -- "openspec/changes/<id>/"
git push origin <base>
```

Write the message by the conventions' commit-message rule, naming the change proposed. The `git add` is required: `git commit -- <pathspec>` commits only paths git already tracks, and the change directory is untracked. The pathspec on `git commit` still commits only this change's paths and ignores anything another session has staged.

If the push is rejected as non-fast-forward, run `git pull --ff-only origin <base>` and push again. The change directory is unique to this change, so it cannot conflict.

Keep write, commit and push close together. An uncommitted file in the shared main worktree can abort the integrator's merge with `error: Untracked working tree file ... would be overwritten by merge`.

### 7. Report

State the change id, its author, its dependencies, its declared files, and whether it can start now — every dependency archived — or waits on a named one. Confirm that `git status --porcelain -- openspec/changes/<id>/` is empty.

## Guardrails

- Refuse from any position other than the main worktree on `<base>`.
- Synchronise before reading shared state, and again after drafting, before the dependency checks.
- Never implement. Write planning artifacts only, and only under `openspec/changes/<id>/`.
- Refuse an id that is open or has an archive record.
- Always write the coordination block with `### Author` first and `### Expected Files` last. Write the login without asking, and never another login.
- Declare the narrowest path or prefix containing the work, never the project or parent directory around it.
- Refuse on an unknown dependency or a cycle, and leave no drafted directory behind.
- Stage by name and commit with the explicit pathspec. Never commit the whole index.
- Never pass `--store`. Never delete `.git/index.lock`.
