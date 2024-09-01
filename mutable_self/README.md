# Calc

This implementation splits static analyses like unparsing from dynamic analyses like logging.

Each concern has its own module that is oblivious to other modules.

For static analyses, we use the visitor pattern.  But crucially, state is owned by fields of `self`, and we run everything under `&mut self`.

For dynamic analyses, the evaluator drives all recursion.  Additional analyses can be activated by adding visitors.  Visitors implement a trait with pre- and post-visit methods.

The evaluator's `eval_expr()` calls `pre_visit()` on all visitors, does `inner_eval_expr()`, and finally calls `post_visit()` on all visitors.  This allows us to separate evaluation logic from visitor dispatch.

By having two methods on the visitor trait, pre and post, we allow wrapping, which is needed by some analyses like performance timing.  This also prevents additional call stack usage that an around-method of a traditional decorator would use.
