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

## Short Example
The following shows how
dead-simple this crate is to use:
```
// main.rs
use hotpatch::patchable;

#[patchable]
fn foo() { }

fn main() -> Result<(), Box<dyn std::error::Error>> {
  foo(); // does nothing
  foo.hotpatch_lib("libsomething.so")?;
  foo(); // does something totally different!
  foo.hotpatch_fn(|| println!("Dyamic!"))?;
  foo(); // even more modification!
  Ok(())
}
```

## Warning
Don't hotpatch the function you're currently in, or any of its parents.

Because `hotpatch` doesn't allow multiple function definitions to be in
affect at the same time, this will cause a deadlock.

It is possible to do this with the `force` functions, however they are
`unsafe`, as in a multithreaded enironment this could cause multiple
function definitions to be in effect at once.

## Docs
For more information, see the [docs](https://docs.rs/hotpatch).
