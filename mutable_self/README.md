# Calc

In this design, each concern has its own module that is oblivious to other modules.  The `ast` module has no code for evaluation or other analyses.  AST types are dumb structs.

Static analyses like unparsing are implemented in a different way from dynamic analyses like logging.  This is because static analyses visit each AST node exactly once.  On the other hand, dynamic analyses follow execution.  AST nodes may be visited zero or more times.

For static analyses, we use the visitor pattern.  But crucially, state is owned by fields of `self`, and we run everything under `&mut self`.  This is both more ergonomic than passing state as parameters and more efficient than returning temporary state that needs to get combined.  State can be directly mutated.

For dynamic analyses, the evaluator drives all recursion.  Additional analyses can be activated by adding visitors to the evaluator.  Visitors implement a trait with pre- and post-visit methods.

The evaluator's `eval_expr()` calls `pre_visit()` on all visitors, does `inner_eval_expr()`, and finally calls `post_visit()` on all visitors.  This allows us to separate evaluation logic from visitor dispatch.

By having two methods on the visitor trait, pre and post, we allow wrapping, which is needed by some analyses like performance timing.  This also prevents additional call stack usage that an around-method of a traditional decorator would use.
