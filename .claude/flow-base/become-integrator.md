---
description: Name this session as the project's integrator, the one session that integrates and receives ready announcements
model: opus
effort: low
allowed-tools: Bash(git:*), ListAgents
---
<!-- Installed by /flow:install from ~/.claude/commands/flow/become-integrator.md. Project edits are kept: /flow:update merges source changes into this file against .claude/flow-base/become-integrator.md. -->

Read `.claude/commands/flow-conventions.md` and follow it. The terms below — `<base>`, the main worktree, `<integrator>` — mean what it defines.

Give this session the name `<integrator>`, so `/ready` announces changes to it and `/integrate` accepts it.

**Input**: none.

**A hook renames the session before this command runs.** The model has no tool that renames a session, but a `UserPromptSubmit` hook can. The `flow-become-integrator.sh` hook sees this command's prompt and returns `<integrator>` as its `sessionTitle`, so the session already holds the name when step 1 begins. The hook renames only from the main worktree on `<base>`, and only when no other live session holds the name. This command checks the same conditions and confirms the name with `ListAgents`. It hands the rename to the operator only when the hook did not run.

## Steps

### 1. Guard the position

```sh
git rev-parse --git-dir
git rev-parse --git-common-dir
git branch --show-current
```

Refuse unless the two paths are equal and the branch is `<base>`. `/integrate` runs only there, so a session named from anywhere else holds the name and cannot use it. Name the worktree and branch found.

### 2. Derive the name

Derive `<integrator>` by the conventions' rule.

### 3. Read the sessions

```
ListAgents
```

**Only an exact match is a collision.** A session holds the name only when its name equals `<integrator>` as exact text. A bare `integrator`, another project's `<project>-integrator` and every other name are not collisions. Never stop or warn because of one.

- **Another session named `<integrator>` is anything but offline.** Refuse and name it. Two integrators collide on `.git/index.lock` mid-archive. The hook declined the rename for the same reason. The operator retires the peer, then runs this command again.
- **This session is `<integrator>`.** Report that it holds the name, so `/ready` announces to it and `/integrate` accepts it. Stop.
- **This session is `<integrator>` followed by a suffix.** Claude Code gives a name already held a two-word suffix, which `/integrate` refuses. Name the session `ListAgents` lists as `<integrator>`, and say to retire it and run this command again.
- **Any other name.** The hook did not run. Continue at step 4.

### 4. Hand the rename to the operator

The hook is absent from this operator's configuration. Print the line for the operator to type, alone in a code block:

```
/rename <integrator>
```

Then say how to confirm it: run this command again, and step 3 reports the name. To start a new integrator session instead, launch `claude --name <integrator>` from the main worktree.

## Guardrails

- Refuse from any position other than the main worktree on `<base>`.
- Refuse while another live session holds `<integrator>` as exact text. Never stop for any other name.
- Report the session as named only when `ListAgents` reports it.
- Write nothing: no file, tag, branch or commit.
