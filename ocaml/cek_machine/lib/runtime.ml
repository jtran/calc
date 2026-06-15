type env = (Ast.identifier, value) Hashtbl.t

and value = Number of float | Closure of Ast.func * env

(* What the machine is doing right now. *)
and control =
  | EvalExpr of Ast.expr
  | EvalFactor of Ast.factor
  | Return of value

(* Continuation frames — the explicit evaluation stack that replaces native
   recursion. Each frame records what to do once the current sub-evaluation
   produces a value. *)
and kont =
  | KTermRight of Ast.term_binary_op * Ast.expr (* lhs pending; eval rhs next *)
  | KTermApply of Ast.term_binary_op * value (* lhs done; combine with rhs *)
  | KFactorRight of Ast.factor_binary_op * Ast.factor
  | KFactorApply of Ast.factor_binary_op * value
  | KCallFunc of Ast.expr list (* func done; start evaluating args *)
  | KCallArgs of value * Ast.expr list * value list
    (* closure, remaining args, accumulated arg values (reversed) *)
  | KReturnTo of env (* restore caller scope after a function body *)

(* The runtime machine state. The explicit stack lives here, separate from the
   variable [env] that closures capture. *)
and state =
  {mutable control: control; mutable scope: env; mutable stack: kont list}

let show_value (v : value) : string =
  match v with
  | Number n ->
      Printf.sprintf "%f" n
  | Closure (func, _) ->
      Printf.sprintf "(Closure %s)" (Ast.show_func func)

exception RuntimeError of string
