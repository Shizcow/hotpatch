### State of this project
Currently waiting for [inherent traits](https://github.com/rust-lang/rfcs/pull/2375). There has been some action here recently, and the RFC looks like it's going to be merged soon. Given that I'm busy with school anyway, I'll be holding off on continuing development until inherent traits are in nightly.

# hotpatch
[![crates.io](https://img.shields.io/crates/v/hotpatch.svg)](https://crates.io/crates/hotpatch)
[![docs.rs](https://docs.rs/hotpatch/badge.svg)](https://docs.rs/hotpatch)

This crate is primarily used to load new function definitions from shared
object files in an exceedingly easy way.

Key features:
- Thread safe
- Type safe
- Works for functions of any signature
- Namespace aware

## Nightly Requirement
This crate is nightly only. A list of features it uses are as follows:
- `unboxed_closures`
- `fn_traits`
- `const_fn`
- `const_fn_fn_ptr_basics`
- `proc_macro_diagnostic`

Most of the above features are critical to function. As such, this crate will remain nightly only until more of the above are finished.

## Short Example
The following shows how
dead-simple this crate is to use:
```rust
// main.rs
use hotpatch::*;

#[patchable]
fn foo() { }

fn main() -> Result<(), Box<dyn std::error::Error>> {
  foo(); // does nothing
  foo.hotpatch_lib("libsomething.so")?;
  foo(); // does something totally different!
  foo.hotpatch_fn(|| println!("Dynamic!"))?;
  foo(); // even more modification!
  Ok(())
}
```

## Warning
Don't hotpatch the function you're currently in, or any of its parents.

Because `hotpatch` doesn't allow multiple function definitions to be in
affect at the same time, this will cause a deadlock. `try` variants
exist which will return an error if a deadlock would occur.

It is possible to do this with the `force` functions, however they are
`unsafe`, as in a multithreaded enironment this could cause multiple
function definitions to be in effect at once.

## Docs
For more information, see the [docs](https://docs.rs/hotpatch).


## TODO
This crate is still has a long way to go before being "finished". Below are some items left to do. Submit an issue or PR to this section for feature requests!  
- `no_std` and use features to give the widest possible functionality
  - probably will need to move back to `lazy_static`
- wasm support
- methods (in progress)
- `#[patchable] ||()` to generate from a closure (is this even possible?)
- lower compile times
  - include only necessary features for sub-dependencies
