# examples

Homebrew examples of Nintendo Switch homebrew written with this organization's tools

## Example list

- `applet`: library applets

  - `libapplet-launch`: example of manually launching the `PlayerSelect` library applet (will be removed/replaced with libapplet support gets properly implemented in `nx`)

- `fs-api`: filesystem API

  - `file-rw`: example of reading/writing files

  - `dir-list`: example of listing files/dirs

- `graphics`:

  - `gpu-simple`: example of simple gfx support

  - `simple-window`: example of a sysmodule rendering over everything

  - `ui2d`: simple UI libraries used by the examples above:

- `input`: example of input API

- `os`:

  - `threads`: example of thread support

- `server-ipc`:

  - `lm`: simple replacement of `LogManager` sysmodule
  
  - `ams-ecs`: simple usage of Atmosphere's external content source API to take over a game and redirect it to custom ExeFs/RomFs on the SD card

  - `prepo-mitm`: simple MitM of `prepo` (Play Report) services

  - `simple-mitm-service`: example of how a IPC service MitM works

    - `client`: client-side example

    - `server`: server-side example

  - `simple-service`: example of how a regular IPC service works

    - `client`: client-side example

    - `server`: server-side example