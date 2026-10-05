# agent-status

A sample gband plugin with both sides. It notices when a coding agent in a pane stops to ask a question, and tells you wherever your client runs.

- `server.lua` runs in the server. It keeps the last 512 bytes each pane printed. When they hold a prompt such as `Do you want to proceed?`, it sets the pane's state key `agent` to `"waiting"` and emits the event `agent.waiting` to the clients of the session. A key or a paste into the pane clears the key. It registers the command `agent-status.next_waiting`, which focuses the next waiting pane in the calling client.
- `client.lua` runs in each client. It raises a desktop notification on `agent.waiting`, adds a status line component that counts the waiting panes, and binds `prefix a` to the command.
- `plugin.lua` is the manifest. Its `client = ">= 0.1"` asks every client that attaches to load `agent-status` 0.1 or later; a client without it reports an error naming the plugin.

The server keeps the state and queues the event while no client is attached, so an agent that stops while you are detached notifies you when you attach again.

Read [the plugin guide](../../../docs/plugins.md) for the API it uses.

## Install

Link this directory into the plugins directory of the machine the server runs on and of each machine a client runs on. When both run on one machine, one link serves both:

```sh
mkdir -p ~/.local/share/gband/plugins
ln -s "$PWD/examples/plugins/agent-status" ~/.local/share/gband/plugins/agent-status
```

Start a new server with `gband kill-server` and attach again, then run `printf 'Do you want to proceed?\n'` in a pane.
The status line shows `agents waiting: 1`, your terminal shows a notification, and Ctrl+Space then `a` focuses that pane.
