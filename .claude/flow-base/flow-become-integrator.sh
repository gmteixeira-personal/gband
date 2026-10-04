#!/usr/bin/env bash
# Installed by /flow:install from ~/.claude/hooks/flow-become-integrator.sh. Project edits are kept: /flow:update merges source changes into this file against .claude/flow-base/flow-become-integrator.sh.
# UserPromptSubmit: rename the session to the project's integrator name when the prompt runs
# /become-integrator.
# The model has no tool that renames a session; a hook's sessionTitle is the supported way.
# Renames only from the main worktree on origin's default branch, and only when no other live
# session holds the exact name: Claude Code would give a duplicate a suffix /integrate refuses.
# The command itself confirms the outcome with ListAgents.
set -euo pipefail

input=$(cat)
prompt=$(jq -r '.prompt // empty' <<<"$input")
[[ $prompt =~ ^[[:space:]]*/become-integrator([[:space:]]|$) ]] || exit 0
cwd=$(jq -r '.cwd // empty' <<<"$input")
session=$(jq -r '.session_id // empty' <<<"$input")
[[ -n $cwd && -n $session ]] || exit 0

git_dir=$(git -C "$cwd" rev-parse --path-format=absolute --git-dir 2>/dev/null) || exit 0
common_dir=$(git -C "$cwd" rev-parse --path-format=absolute --git-common-dir) || exit 0
[[ $git_dir == "$common_dir" ]] || exit 0
base=$(git -C "$cwd" symbolic-ref --short refs/remotes/origin/HEAD 2>/dev/null) || exit 0
[[ $(git -C "$cwd" branch --show-current) == "${base#origin/}" ]] || exit 0

main=$(git -C "$cwd" worktree list --porcelain | sed -n '1s/^worktree //p')
[[ -n $main ]] || exit 0
name="${main##*/}-integrator"

shopt -s nullglob
for record in "${CLAUDE_CONFIG_DIR:-$HOME/.claude}"/sessions/*.json; do
	pid=$(jq -r --arg name "$name" --arg session "$session" \
		'select(.name == $name and .sessionId != $session) | .pid' "$record" 2>/dev/null) || continue
	if [[ -n $pid ]] && kill -0 "$pid" 2>/dev/null; then
		exit 0
	fi
done

jq -n --arg title "$name" '{hookSpecificOutput: {hookEventName: "UserPromptSubmit", sessionTitle: $title}}'
