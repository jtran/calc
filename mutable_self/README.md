# Calc

## Mutable self

In this design, each analysis has its own module that is oblivious to other modules.  The `ast` module has no code for evaluation or other analyses.  AST types are dumb structs.

Static analyses like unparsing are implemented in a different way from dynamic analyses like logging.  This is because static analyses visit each AST node exactly once.  On the other hand, dynamic analyses follow execution.  AST nodes may be visited zero or more times.

For static analyses, we use the visitor pattern.  But crucially, state is owned by fields of `self`, and we run everything under `&mut self`.  This is both more ergonomic than passing state as parameters and more performant than returning temporary state that needs to get combined.  State can be directly mutated.

For dynamic analyses, the evaluator drives all recursion.  Additional analyses can be activated by adding visitors to the evaluator.  Visitors implement a trait with pre- and post-visit methods.

The evaluator's `eval_expr()` calls `pre_visit()` on all visitors, does `inner_eval_expr()`, and finally calls `post_visit()` on all visitors.  This allows us to separate evaluation logic from visitor dispatch.  It also allows us to use `?` in inner functions, which is extremely convenient.

By having two methods on the visitor trait, pre and post, corresponding to pre-order and post-order depth-first search [traversal](https://en.wikipedia.org/wiki/Tree_traversal), we allow wrapping, which is needed by some analyses like performance timing.  This also prevents additional call stack usage that an around-method of a traditional decorator would use.

Some parts of the evaluator need to be accessible by other things.  These have been moved to another module called `runtime` to signify that they are part of the runtime system that's expected to be available.

## Future Work

As is, static analyses are done separately.  I.e. static analysis 1 is run on the entire AST, then static analysis 2 is run on the entire AST, etc.

In the future, as user programs get larger, it may be desirable to run different analyses concurrently on a single pass of the AST.  With an iterator with multiple map steps, it's desireable to run all the map functions on a single element before moving on to the next element.  Similarly, we could run multiple static analyses on an AST node before moving on to the next node.

Compared to iterators, AST traversals are complicated by the fact that they often need to run code both before and after recursing.  They may also need a return value from the recursive computation.  If there are multiple children, like with a binary operator expression, it means there are multiple return values, one for each subtree.

Additionally, multiple analyses are not composed with each other; they're completely separate.  We'd want each node to fan-out to each analysis.

I think that these issues make it non-trivial, and manual implementation of traversal is just fine for the time being.

However, that's a little unsatisfying.  If this were a functional language, the answer would probably be [CPS](https://en.wikipedia.org/wiki/Continuation-passing_style).  But even in languages where CPS is commonly used, it's easy to make code difficult to read.  There would need to be _significant_ benefit to justify breaking from idiomatic Rust.

One could theoretically do this with async Rust.  But now you run into all the problems with async, with lifetimes crossing async boundaries.
