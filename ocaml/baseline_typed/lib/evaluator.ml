let rec eval_factor (env : Runtime.env) (factor : Ast.factor) : Runtime.value =
  match factor with
  | Ast.Literal n ->
      Runtime.Number n
  | Ast.Variable id -> (
    match Hashtbl.find_opt env id with
    | Some value ->
        value
    | None ->
        raise (Runtime.RuntimeError (Printf.sprintf "Unbound identifier: %s" id))
    )
  | Ast.BinaryOp {op; lhs; rhs} -> (
      let left_val = eval_factor env lhs in
      let right_val = eval_factor env rhs in
      match op with
      | Ast.Mul -> (
        match (left_val, right_val) with
        | Runtime.Number l, Runtime.Number r ->
            Runtime.Number (l *. r)
        | _ ->
            raise
              (Runtime.RuntimeError "Multiplication only supported for numbers")
        )
      | Ast.Div -> (
        match (left_val, right_val) with
        | Runtime.Number l, Runtime.Number r ->
            Runtime.Number (l /. r)
        | _ ->
            raise (Runtime.RuntimeError "Division only supported for numbers") )
      )
  | Ast.Group expr ->
      eval_expr env expr
  | Ast.Call {func; args} -> (
      let func_val = eval_factor env func in
      match func_val with
      | Runtime.Closure (func, closure_env) ->
          let arg_vals = List.map (eval_expr env) args in
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
          eval_expr body_env func.body
      | _ ->
          raise (Runtime.RuntimeError "Attempted to call a non-function value")
      )

and eval_expr (env : Runtime.env) (expr : Ast.expr) : Runtime.value =
  match expr with
  | Ast.Factor factor ->
      eval_factor env factor
  | Ast.BinaryOp {op; lhs; rhs} -> (
      let left_val = eval_expr env lhs in
      let right_val = eval_expr env rhs in
      match op with
      | Ast.Add -> (
        match (left_val, right_val) with
        | Runtime.Number l, Runtime.Number r ->
            Runtime.Number (l +. r)
        | _ ->
            raise (Runtime.RuntimeError "Addition only supported for numbers") )
      | Ast.Sub -> (
        match (left_val, right_val) with
        | Runtime.Number l, Runtime.Number r ->
            Runtime.Number (l -. r)
        | _ ->
            raise (Runtime.RuntimeError "Subtraction only supported for numbers")
        ) )

let eval_stmt (env : Runtime.env) (stmt : Ast.stmt) : Runtime.value =
  match stmt with
  | Ast.Let (id, _ty, expr) ->
      let value = eval_expr env expr in
      Hashtbl.add env id value ; value
  | Ast.Fun (id, func) ->
      let closure = Runtime.Closure (func, Hashtbl.copy env) in
      Hashtbl.add env id closure ; closure

let eval_stmts (stmts : Ast.stmt list) : (Runtime.value, string) result =
  let env = Hashtbl.create 100 in
  let initial_value = Runtime.Number 0.0 in
  try Ok (List.fold_left (fun _ stmt -> eval_stmt env stmt) initial_value stmts)
  with Runtime.RuntimeError msg -> Error msg
