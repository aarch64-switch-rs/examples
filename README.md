# examples

Homebrew examples of Nintendo Switch homebrew written with this organization's tools

## Example list

- `applet`: library applets

  - `libapplet-launch`: example of manually launching the `PlayerSelect` library applet (will be removed/replaced with libapplet support gets properly implemented in `nx`)

- `fs-api`: filesystem API

  - `file-rw`: example of reading/writing files

  - `dir-list`: example of listing files/dirs

- `graphics`:

  - `gpu-simple`, `gpu-simple2`: examples of simple graphics operations

  - `simple-window`: example of a sysmodule rendering over everything

  - `console-interactive`: simple CLI that prints any (external) keyboard input

  - `console-write`: tries to read contents from the file `sdmc:/lorem_ipsum` and prints them using a `ScrollbackConsole`

  - `console-write-random`: tries to read contents from the file `sdmc:/lorem_ipsum` and prints them in randomly sized chunks using a `Console<TextOnGraphic<PersistentBufferedCanvas>>`


- `input`: example of input API

- `os`:

  - `threads`: example of thread support

  - `thread-panic`: example of gracefully catching a panicking thread, requires a build like `RUSTFLAGS="-C panic=unwind" cargo +nightly nx build -p thread_panic` to ensure unwinding occurs.

- `net`:

  - `chat`

  - `echo`

  - `net-log`

- `server-ipc`:

  - `lm`: simple replacement of `LogManager` sysmodule

  - `prepo-mitm`: simple MitM of `prepo` (Play Report) services

  - `simple-mitm-service`: example of how a IPC service MitM works

    - `client`: client-side example

    - `server`: server-side example

  - `simple-service`: example of how a regular IPC service works

    - `client`: client-side example

    - `server`: server-side example