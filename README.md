# hotpatch

Chaning function definitions at runtime.

Key features:
- Thread safe
- Type safe
- Works for functions of any signature
- Namespace aware

## Warnings
- Don't hotpatch the function you're currently in, or any of its parents
  - Because `hotpatch` doesn't allow multiple function definitions to be in
	affect at the same time, this would cause a deadlock

## TODO
- Raise an issue for the root cause of `libloading::Library`'s memory leak
- Optional macro arguements to override automatic module handling (on both ends?)
- Seperate nightly vs non-nightly features and use features to enable
- Docs
- More examples
