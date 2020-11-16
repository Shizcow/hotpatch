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
- Investigate if a linker object can have `::` in its name, and if so how to mangle that in
- Library/proc\_macros for alternative definitions (like `src_obj`)
- Find a more efficient way of storing `libloading::Library` objects to remove duplicates
  - Can we just keep the `libloading::Symbol` and drop the library?
    - Functions can call other functions, so this is impossible
  - maybe a global static with the libs and track live references?
  - Are duplicates magically optimized away?
- Optional macro arguements to override automatic module handling (on both ends?)
- Seperate nightly vs non-nightly features and use features to enable
- Docs
- Tests for thread-saftey, how `RWLock` rejects function calls, and how to handle (`try_call`?)
- What traits to `Patchable` structs need? `Sync`? `Copy`/`Clone`? Should some of these traits be under an `extra_traits` feature option?
- Methods!
- More examples
