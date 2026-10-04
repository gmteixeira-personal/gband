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

**The model cannot rename a session.** Claude Code gives it no tool for that. Only the operator sets the name `ListAgents` reports: with `/rename <name>` in the session, or `claude --name <name>` at launch. This command derives the name, checks that no live session holds it, and gives the operator the exact line to type.

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

- **This session is already `<integrator>`.** Report it and stop. Nothing is left to do.
- **Another session named `<integrator>` is anything but offline.** Refuse and name it. Two integrators collide on `.git/index.lock` mid-archive. Claude Code also gives a duplicate name a two-word suffix, so the rename would produce a name `/integrate` refuses. The operator retires the peer first.
- **This session is named `integrator`, without the project prefix.** Say that the bare name receives no announcement, then continue.

### 4. Hand the rename to the operator

Print the line for the operator to type, alone in a code block:

```
/rename <integrator>
```

Then say how to confirm it: run this command again, and step 3 reports the name. To start a new integrator session instead, launch `claude --name <integrator>` from the main worktree.

## Guardrails

- Refuse from any position other than the main worktree on `<base>`.
- Refuse while another live session holds `<integrator>`.
- Never report the session as renamed. Only the operator's `/rename` renames it, and only `ListAgents` confirms it.
- Write nothing: no file, tag, branch or commit.
