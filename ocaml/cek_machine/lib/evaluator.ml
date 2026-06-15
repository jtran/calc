(* Combine two values with a term-level operator (+ / -). *)
let apply_term (op : Ast.term_binary_op) (l : Runtime.value) (r : Runtime.value)
    : Runtime.value =
  match (op, l, r) with
  | Ast.Add, Runtime.Number a, Runtime.Number b ->
      Runtime.Number (a +. b)
  | Ast.Sub, Runtime.Number a, Runtime.Number b ->
      Runtime.Number (a -. b)
  | Ast.Add, _, _ ->
      raise (Runtime.RuntimeError "Addition only supported for numbers")
  | Ast.Sub, _, _ ->
      raise (Runtime.RuntimeError "Subtraction only supported for numbers")

(* Combine two values with a factor-level operator: multiplication or division. *)
let apply_factor (op : Ast.factor_binary_op) (l : Runtime.value)
    (r : Runtime.value) : Runtime.value =
  match (op, l, r) with
  | Ast.Mul, Runtime.Number a, Runtime.Number b ->
      Runtime.Number (a *. b)
  | Ast.Div, Runtime.Number a, Runtime.Number b ->
      Runtime.Number (a /. b)
  | Ast.Mul, _, _ ->
      raise (Runtime.RuntimeError "Multiplication only supported for numbers")
  | Ast.Div, _, _ ->
      raise (Runtime.RuntimeError "Division only supported for numbers")

(* Enter a function body: build its environment, save the caller's scope on the
   stack so it can be restored on return, and jump into the body. Mutates [st];
   never calls back into the loop, so the native stack stays flat. *)
let apply_call (st : Runtime.state) (fv : Runtime.value)
    (arg_vals : Runtime.value list) : unit =
  match fv with
  | Runtime.Closure (func, closure_env) ->
      if List.length arg_vals <> List.length func.params then
        raise
          (Runtime.RuntimeError
             (Printf.sprintf "Expected %d arguments but got %d"
                (List.length func.params) (List.length arg_vals) ) ) ;
      let body_env = Hashtbl.copy closure_env in
      List.iter2
        (fun (param : Ast.param) (arg_val : Runtime.value) ->
          Hashtbl.add body_env param.name arg_val )
        func.params arg_vals ;
      st.Runtime.stack <- Runtime.KReturnTo st.Runtime.scope :: st.Runtime.stack ;
      st.Runtime.scope <- body_env ;
      st.Runtime.control <- Runtime.EvalExpr func.body
  | _ ->
      raise (Runtime.RuntimeError "Attempted to call a non-function value")

(* Once the callee value is known, start evaluating arguments left-to-right. *)
let begin_args (st : Runtime.state) (fv : Runtime.value) (args : Ast.expr list)
    : unit =
  match args with
  | [] ->
      apply_call st fv []
  | first :: rest ->
      st.Runtime.stack <- Runtime.KCallArgs (fv, rest, []) :: st.Runtime.stack ;
      st.Runtime.control <- Runtime.EvalExpr first

(* Iteratively evaluate an expression using an explicit continuation stack
   (a CEK-style machine) instead of native recursion. *)
let run_expr (scope : Runtime.env) (e : Ast.expr) : Runtime.value =
  let st = {Runtime.control= Runtime.EvalExpr e; scope; stack= []} in
  let result = ref (Runtime.Number 0.0) in
  let running = ref true in
  while !running do
    match st.Runtime.control with
    (* ---- Eval steps: decompose the focus, push a frame, descend ---- *)
    | Runtime.EvalExpr (Ast.Factor f) ->
        st.Runtime.control <- Runtime.EvalFactor f
    | Runtime.EvalExpr (Ast.BinaryOp {op; lhs; rhs}) ->
        st.Runtime.stack <- Runtime.KTermRight (op, rhs) :: st.Runtime.stack ;
        st.Runtime.control <- Runtime.EvalExpr lhs
    | Runtime.EvalFactor (Ast.Literal n) ->
        st.Runtime.control <- Runtime.Return (Runtime.Number n)
    | Runtime.EvalFactor (Ast.Variable id) -> (
      match Hashtbl.find_opt st.Runtime.scope id with
      | Some v ->
          st.Runtime.control <- Runtime.Return v
      | None ->
          raise
            (Runtime.RuntimeError (Printf.sprintf "Unbound identifier: %s" id))
      )
    | Runtime.EvalFactor (Ast.Group inner) ->
        st.Runtime.control <- Runtime.EvalExpr inner
    | Runtime.EvalFactor (Ast.BinaryOp {op; lhs; rhs}) ->
        st.Runtime.stack <- Runtime.KFactorRight (op, rhs) :: st.Runtime.stack ;
        st.Runtime.control <- Runtime.EvalFactor lhs
    | Runtime.EvalFactor (Ast.Call {func; args}) ->
        st.Runtime.stack <- Runtime.KCallFunc args :: st.Runtime.stack ;
        st.Runtime.control <- Runtime.EvalFactor func
    (* ---- Return steps: pop a frame, advance to the next sub-node or combine ---- *)
    | Runtime.Return v -> (
      match st.Runtime.stack with
      | [] ->
          result := v ;
          running := false
      | Runtime.KTermRight (op, rhs) :: rest ->
          st.Runtime.stack <- Runtime.KTermApply (op, v) :: rest ;
          st.Runtime.control <- Runtime.EvalExpr rhs
      | Runtime.KTermApply (op, lv) :: rest ->
          st.Runtime.stack <- rest ;
          st.Runtime.control <- Runtime.Return (apply_term op lv v)
      | Runtime.KFactorRight (op, rhs) :: rest ->
          st.Runtime.stack <- Runtime.KFactorApply (op, v) :: rest ;
          st.Runtime.control <- Runtime.EvalFactor rhs
      | Runtime.KFactorApply (op, lv) :: rest ->
          st.Runtime.stack <- rest ;
          st.Runtime.control <- Runtime.Return (apply_factor op lv v)
      | Runtime.KCallFunc args :: rest -> (
        (* [v] is the callee; reject non-functions before evaluating args. *)
        match v with
        | Runtime.Closure _ ->
            st.Runtime.stack <- rest ;
            begin_args st v args
        | _ ->
            raise (Runtime.RuntimeError "Attempted to call a non-function value")
        )
      | Runtime.KCallArgs (fv, remaining, acc) :: rest -> (
          let acc = v :: acc in
          match remaining with
          | [] ->
              st.Runtime.stack <- rest ;
              apply_call st fv (List.rev acc)
          | next :: more ->
              st.Runtime.stack <- Runtime.KCallArgs (fv, more, acc) :: rest ;
              st.Runtime.control <- Runtime.EvalExpr next )
      | Runtime.KReturnTo saved :: rest ->
          st.Runtime.scope <- saved ;
          st.Runtime.stack <- rest ;
          st.Runtime.control <- Runtime.Return v )
  done ;
  !result

let eval_stmt (scope : Runtime.env) (stmt : Ast.stmt) : Runtime.value =
  match stmt with
  | Ast.Let (id, _ty, expr) ->
      let value = run_expr scope expr in
      Hashtbl.add scope id value ; value
  | Ast.Fun (id, func) ->
      let closure = Runtime.Closure (func, Hashtbl.copy scope) in
      Hashtbl.add scope id closure ;
      closure

let eval_stmts (stmts : Ast.stmt list) : (Runtime.value, string) result =
  let env = Hashtbl.create 100 in
  let initial_value = Runtime.Number 0.0 in
  try Ok (List.fold_left (fun _ stmt -> eval_stmt env stmt) initial_value stmts)
  with Runtime.RuntimeError msg -> Error msg
