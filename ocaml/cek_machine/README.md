# calc

This is based on `baseline_typed`. The evaluator is changed to not recurse.
Instead, it returns the next expression to evaluate and how to continue. This is
called a CEK machine.

# Usage

Run `bin/main.ml`.

```shell
dune exec calc
```

Format all code.

```shell
dune fmt
```
