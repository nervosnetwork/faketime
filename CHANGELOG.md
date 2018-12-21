# [0.2.0](https://github.com/nervosnetwork/faketime/compare/v0.1.0...v0.2.0) (2018-12-21)

### Features

- new API `unix_time_as_millis`. ([d6e2bb1](https://github.com/nervosnetwork/faketime/commit/d6e2bb1))
- use env FAKETIME to sepcify file directly. ([35709ad](https://github.com/nervosnetwork/faketime/commit/35709ad))
- Windows support ([e5fdc02](https://github.com/nervosnetwork/faketime/commit/e5fdc02))

### BREAKING CHANGES

- `FAKETIME_DIR` is obsoleted. Set environment variable `FAKETIME` to enable faketime and use the specified file for all threads in the process.

# [0.1.0](https://github.com/nervosnetwork/faketime/commit/4f01cf37563460c3c5ab15698b61b9af7bd713aa) (2018-12-20)

### Features

- release faketime ([4f01cf3](https://github.com/nervosnetwork/faketime/commit/4f01cf3))
