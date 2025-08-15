type env = (Ast.identifier, Ast.typ) Hashtbl.t

exception TypeError of string

let rec check_factor env factor : Ast.typ =
  match factor with
  | Ast.Literal _ ->
      Ast.Number
  | Ast.Variable id -> (
    match Hashtbl.find_opt env id with
    | Some ty ->
        ty
    | None ->
        raise (TypeError (Printf.sprintf "Unbound variable: %s" id)) )
  | Ast.Group expr ->
      check_expr env expr
  | Ast.BinaryOp {op= _; lhs; rhs} ->
      let t1 = check_factor env lhs in
      let t2 = check_factor env rhs in
      (* All ops currently expect the same types. *)
      let expected_ty = Ast.Number in
      if t1 != expected_ty then
        raise
          (TypeError
             (Printf.sprintf "Type mismatch: expected %s, found %s"
                (Ast.show_typ expected_ty) (Ast.show_typ t1) ) )
      else if t2 != expected_ty then
        raise
          (TypeError
             (Printf.sprintf "Type mismatch: expected %s, found %s"
                (Ast.show_typ expected_ty) (Ast.show_typ t2) ) )
      else expected_ty
  | Ast.Call {func; args} -> (
      let func_ty = check_factor env func in
      match func_ty with
      | Ast.Arrow (param_tys, return_ty) ->
          if List.length param_tys != List.length args then
            raise
              (TypeError
                 (Printf.sprintf
                    "Number of function parameters differs from arguments: \
                     expected %d, found %d"
                    (List.length param_tys) (List.length args) ) ) ;
          List.iter2
            (fun expected_ty arg ->
              let arg_ty = check_expr env arg in
              if arg_ty != expected_ty then
                raise
                  (TypeError
                     (Printf.sprintf "Type mismatch: expected %s, found %s"
                        (Ast.show_typ expected_ty) (Ast.show_typ arg_ty) ) )
              else () )
            param_tys args ;
          return_ty
      | _ ->
          raise
            (TypeError
               (Printf.sprintf "Expected function, found %s"
                  (Ast.show_typ func_ty) ) ) )

and check_func env (rec_id : Ast.identifier option) (func : Ast.func) : Ast.typ
    =
  let func_ty =
    Ast.Arrow
      ( List.map (fun (p : Ast.param) : Ast.typ -> p.ty) func.params
      , func.return_ty )
  in
  (* Add the function name to the environment for recursion. *)
  ignore (Option.map (fun id -> Hashtbl.add env id func_ty) rec_id) ;
  (* Add the parameters. *)
  List.iter (fun (p : Ast.param) -> Hashtbl.add env p.name p.ty) func.params ;
  (* Check the function body with the parameters in scope. *)
  ignore (check_expr env func.body) ;
  (* Remove parameters. *)
  List.iter
    (fun (p : Ast.param) -> Hashtbl.remove env p.name)
    (List.rev func.params) ;
  ignore (Option.map (fun id -> Hashtbl.remove env id) rec_id) ;
  func_ty

and check_expr env expr : Ast.typ =
  match expr with
  | Ast.Factor e ->
      check_factor env e
  | Ast.BinaryOp {op= _; lhs; rhs} ->
      let t1 = check_expr env lhs in
      let t2 = check_expr env rhs in
      (* All ops currently expect the same types. *)
      let expected_ty = Ast.Number in
      if t1 != expected_ty then
        raise
          (TypeError
             (Printf.sprintf "Type mismatch: expected %s, found %s"
                (Ast.show_typ expected_ty) (Ast.show_typ t1) ) )
      else if t2 != expected_ty then
        raise
          (TypeError
             (Printf.sprintf "Type mismatch: expected %s, found %s"
                (Ast.show_typ expected_ty) (Ast.show_typ t2) ) )
      else expected_ty

let check_stmt env stmt =
  match stmt with
  | Ast.Let (id, ty, expr) ->
      let expr_ty = check_expr env expr in
      if ty != expr_ty then
        raise
          (TypeError
             (Printf.sprintf
                "Type mismatch in let binding: expected %s, found %s"
                (Ast.show_typ ty) (Ast.show_typ expr_ty) ) ) ;
      Hashtbl.add env id ty
  | Ast.Fun (id, func) ->
      let func_ty = check_func env (Some id) func in
      Hashtbl.add env id func_ty

let check_stmts (stmts : Ast.stmt list) : (env, string) result =
  let env = Hashtbl.create 100 in
  try
    List.iter (check_stmt env) stmts ;
    Ok env
  with TypeError msg -> Error msg
