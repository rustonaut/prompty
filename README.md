
# Prompy

Is _my_ bash prompt written in rust.

It's also a playground I used to learn a bit more
about a certain architectural design which is in
a certain way total over engineering for a bash
prompt command.

As this is solely meant for personal use this only
runs on linux, might not be compatible with all terminals
and might not work with a non-bash shell.

I will response to issues but if the issue doesn't affect
me the response is not unlikely something on the line
of "I apologize but I won't fix this/add this feature".

# Usage

For trying it out use:

`> eval $(cargo run --release -- --bash-setup)`

For a more permanent setup:

1. build the binary `cargo build --release`
2. get binary from `./target/release/prompty`
3. add following to `.bashrc`: `eval $(prompty --bash-setup)`
   Where `prompty` should be a path to the `prompty` binary.

The `--bash-setup` option makes `prompty` a bash snipped consisting of:

1. A assignment to `PS1` in a form similar to `PS1='$(prompty $COLUMN)'.
   Note that instead of `prompty` a absolute path the the `prompty` binary will be
   used determined by rusts `std::env::current_exec()` function.
2. Add a function called `g` which works like `cd` but will set the `__PS_PATH_TOP`
   environment variable, which prompty uses to trim the displayed current working
   dir (if possible, if not it will try the value of `$HOME` if not it just displays
   the full path).
   - note that the `g` function is likely to be extended/changed in the future and
     might collide with your own aliases/programs. A non hard-coded config is possible
     but currently not given as I simply don't need it.





