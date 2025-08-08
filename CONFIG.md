# config

The canonical documentation of shpool's config is the comments
on the `Config` struct defined in `libshpool/src/config.rs`, but
this document aims to provide some high level explanations of
some common configuration options.

You can specify the path to your config file by passing a
`-c /path/to/config.toml` flag, or by creating and
editing `~/.config/shpool/config.toml`.

## Prompt Prefix

By default, `shpool` will detect when you are using a shell it knows
how to inject a prompt into. Currently, those shells include `bash`,
`zsh` and `fish`, but more may be added in the future. If it noticed
you are using one such shell, it will inject the prompt prefix
`shpool:$SHPOOL_SESSION_NAME` at the beginning of your prompt
in order to hint to you when you are inside of a `shpool` session.

You can customize this prompt prefix by setting a new value in
your config. For example, to show the `shpool` session name
inside square brackets, you can put

```
prompt_prefix = "[$SHPOOL_SESSION_NAME]"
```

in your config file. If you want to instead completely suppress
the prompt injection, you can just set a blank `prompt_prefix`
with

```
prompt_prefix = ""
```

this allows you to write a custom prompt hook in your .rc files
that examines the `$SHPOOL_SESSION_NAME` environment variable
directly, or eschew a `shpool` prompt customization entirely.

## Session Restore

`shpool` preserves shell output that occurs while you're disconnected and can
restore it when you reattach to a session. You can control this behavior using
the `session_restore` configuration option, which specifies how much output to
cache in memory.

### Default Behavior (5MB cache)

By default, `shpool` maintains a 5MB cache of terminal output per session.
When you reconnect, it will restore the cached output to give you context
about what happened while you were away. This works well for most use cases
without consuming excessive memory.

### Customizing Cache Size

You can specify a custom cache size using memory units:

```toml
session_restore = "10MB"  # 10 megabytes
session_restore = "512KB" # 512 kilobytes  
session_restore = "1GB"   # 1 gigabyte (for heavy logging)
```

### Disabling Session Restore

To disable output caching completely (SIGWINCH signals only):

```toml
session_restore = "0"
```

This mode will only send SIGWINCH signals to help full-screen applications
like vim or emacs redraw, but won't restore any shell output.

### Per-Session Override

You can override the configured cache size for individual sessions using
the `--restore` command line parameter:

```bash
shpool attach --restore 1MB session_name    # Use 1MB for this session
shpool attach --restore 0 session_name      # No caching for this session
shpool attach session_name                   # Use default config
```

This is useful when you know a session will generate lots of output or when
you want to minimize memory usage for specific sessions.

## Detach Keybinding

You may wish to configure your detach keybinding.
By default, `shpool` will detach from the current user session when you
press the sequence `Ctrl-Space Ctrl-q` (press `Ctrl-Space` then release
it and press `Ctrl-q`, don't try to hold down all three keys at once),
but you can configure a different binding by adding an entry
like

```
[[keybinding]]
binding = "Ctrl-a d"
action = "detach"
```

to your `~/.config/shpool/config.toml`.

For the moment, control is the only modifier key supported, but the keybinding
engine is designed to be able to handle more, so if you want a different one,
you can file a bug with your feature request.

## motd

`shpool` has support for displaying the message of the day (the message `sshd`
shows you when you first log into a system). This is most relevant to users
in institutional settings where important information gets communicated
via the message of the day.

### never mode

```
motd = "never"
```

currently, this is the default mode. In this mode, the message of the day will
not be shown by `shpool`.

### dump mode

```
motd = "dump"
```

in dump mode, `shpool` will dump out the motd inline the first time you
start a new session, but you will not see it when you re-attach to an
existing session.

### pager mode

```
[motd.pager]
bin = "less"
```

in pager mode, `shpool` will display the message of the day in a configurable
pager program. The pager must accept a file name to display as its first argument.
`shpool` will launch the pager in a pty and wait until it exits before moving
on to the actual terminal session. Pager mode is more disruptive than
dump mode, but it allows shpool to show you the motd even if you have a single
long running session you keep around for months and continually reattach to.

## Command Aliases

`shpool` supports command aliases to create shortcuts for commonly used commands.
You can define aliases in your config file using a simple mapping syntax:

```toml
[aliases]
dt = "detach"
at = "attach" 
sw = "switch"
ls = "list"
```

With these aliases configured, you can use the short forms instead of the full commands:

- `shpool dt session1` instead of `shpool detach session1`
- `shpool at session1` instead of `shpool attach session1`
- `shpool sw session2` instead of `shpool switch session2`
- `shpool ls` instead of `shpool list`

### Alias Features

- **Dynamic Reloading**: Aliases are reloaded automatically when you modify your config file, no need to restart the daemon
- **Full Argument Support**: All arguments and flags work with aliases - `shpool at -f session1` becomes `shpool attach -f session1`
- **Configuration Merging**: Aliases defined in system-level config can be overridden by user-level config
- **Error Handling**: Invalid alias configurations will be ignored and won't prevent shpool from working

### Alias Naming

- Alias names can contain letters, numbers, and common symbols
- Avoid using existing command names as aliases (e.g., don't alias "list" to something else)
- Keep aliases short and memorable for the best user experience
