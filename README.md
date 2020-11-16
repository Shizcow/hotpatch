# hotpatch

Chaning function definitions at runtime.

Key features:
- Thread safe
- Type safe
- Works for functions of any signature
- Namespace aware

## Directory
- `src_bin`: Example, includes main and source definintion
- `src_obj`: Example, includes alternative definitions for `src_bin`
- `patchable`: main library
- `patch_proc`: proc macro stuff

## TODO
- Fix project structure
  - Workspaces
  - Better naming
  - Proper .gitignore
- Investigate if a linker object can have `::` in its name, and if so how to mangle that in
- Figure out a way to make `src_obj` not need a single export static (for proc_macro reasons)
- Library/proc\_macros for alternative definitions (like `src_obj`)
- Finalize module-aware functionality
- Embed type information in exports for increased saftey
  (can't hotpatch with an incorrect function signature)
- Find a more efficient way of storing `libloading::Library` objects to remove duplicates
  - Can we just keep the `libloading::Symbol` and drop the library?
  - maybe a global static with the libs and track live references?
  - Are duplicates magically optimized away?
- Optional macro arguements to override automatic module handling (on both ends?)
- Seperate nightly vs non-nightly features and use features to enable
- Docs
- Tests for thread-saftey, how `RWLock` rejects function calls, and how to handle (`try_call`?)
- What traits to `Patchable` structs need? `Sync`? `Copy`/`Clone`? Should some of these traits be under an `extra_traits` feature option?
- More examples
