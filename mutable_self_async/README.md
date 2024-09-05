# Calc

## Mutable self async

This is like `mutable_self` but with async.

To exercise async, we add two expressions at the lowest level, timeout and yield.  I also added an async sleep to the plus and minus operations to more reliably trigger timeouts.

Everything works the same as the sync version.  In addition to `tokio`, we need the `async-recursion` crate.  Visitors on the evaluator need an extra `Send` bound.  That's it!
