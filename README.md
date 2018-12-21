# faketime

[![Build Status](https://travis-ci.com/nervosnetwork/faketime.svg?branch=master)](https://travis-ci.com/nervosnetwork/faketime)
[![Build status](https://ci.appveyor.com/api/projects/status/h7t1uxnbag13rvv6?svg=true)](https://ci.appveyor.com/project/doitian/faketime)

Provides a method `unix_time` which returns elapsed time since _UNIX EPOCH_.
The returned time can be faked in each thread separately.

[Documentation](https://docs.rs/faketime)

## Quick Started

Add faketime as dependency and use `faketime::unix_time` or
`faketime::unix_time_as_millis` to get current time.

To fake time in test:

- Use `faketime::millis_tempfile` to create a temp timestamp file.
- Enable faketime via `faketime::enable` in current thread.

To fake time in child threads:

- Use `faketime::millis_tempfile` to create a temp timestamp file.
- Set child thread name to `FAKETIME=PATH`, where PATH is the path to the
timestamp file.

To fake time of the generated binary, set the environment variable

```
echo 123456 > /tmp/faketime
FAKETIME=/tmp/faketime path/to/binary
```
