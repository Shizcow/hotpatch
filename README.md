# hotpatch

Chaning function definitions at runtime.

Key features:
- Thread safe
- Type safe
- Works for functions of any signature
- Namespace aware

## Directory
- `examples`
  - `hello_world`: A basic example on how to get things up and running
    - `src_bin`: Includes main and source definintion
    - `src_obj`: Includes alternative definitions for `src_bin`
- `hotpatch`: Main library. End users include this crate
- `hotpatch_macros`: Proc macro stuff. End users can ignore

## TODO
- Figure out how to hotpatch `main` via `#[start]` or `#[main]`
- Figure out local hotpatching with functions and closures.
  - Can you overload `=` in Rust?
  - Can `Patchable::hotpatch` take arguements of different types that `.into` into a funciton pointer?
- GATs were added to nightly. Does this allow anything particularly useful?
- Raise an issue for the root cause of `libloading::Library`'s memory leak
- Optional macro arguements to override automatic module handling (on both ends?)
- Seperate nightly vs non-nightly features and use features to enable
- Docs
  - Does anything need `#[doc(hidden)]` from `proc_macro`s?
- See how far out variadic template parameters are so the extra layer of indirection
  required by the tuple Fn args can go away
- More examples
