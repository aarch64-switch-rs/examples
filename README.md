# examples

Homebrew examples of Nintendo Switch homebrew written with this organization's tools

## Example list

- `applet`:

  - `libapplet-launch`: example of manually launching the `PlayerSelect` library applet (will be removed/replaced with libapplet support gets properly implemented in `nx`)

- `fs-api`: example of the filesystem API

- `graphics`:

  - `gpu-simple`: example of simple gfx support

  - `simple-window`: example of a sysmodule rendering over everything

  - `ui2d`: simple UI libraries used by the examples above:

- `os`:

  - `threads`: example of thread support

- `server-ipc`:

  - `lm`: simple replacement of `LogManager` sysmodule

  - `prepo-mitm`: simple MitM of `prepo` (Play Report) services

  - `simple-mitm-service`: example of how a IPC service MitM works

    - `client`: client-side example

    - `server`: server-side example

  - `simple-service`: example of how a regular IPC service works

    - `client`: client-side example

    - `server`: server-side example