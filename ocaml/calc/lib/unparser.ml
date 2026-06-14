type ctx = {buf: Buffer.t; indent: int}

let rec unparse_ty (ctx : ctx) (ty : Ast.typ) : unit =
  match ty with
  | Ast.Number ->
      Buffer.add_string ctx.buf "number"
  | Ast.Arrow (params, return_ty) ->
      List.iter
        (fun param ->
          unparse_ty ctx param ;
          Buffer.add_string ctx.buf ", " )
        params ;
      Buffer.add_string ctx.buf " -> " ;
      unparse_ty ctx return_ty

let rec unparse_expr (ctx : ctx) (expr : Ast.expr) : unit =
  match expr with
  | Ast.Factor factor ->
      unparse_factor ctx factor
  | Ast.BinaryOp {op; lhs; rhs} ->
      unparse_expr ctx lhs ;
      let op_str = match op with Ast.Add -> " + " | Ast.Sub -> " - " in
      Buffer.add_string ctx.buf op_str ;
      unparse_expr ctx rhs

and unparse_factor (ctx : ctx) (factor : Ast.factor) : unit =
  match factor with
  | Ast.Literal n ->
      Printf.bprintf ctx.buf "%g" n
  | Ast.Variable id ->
      Buffer.add_string ctx.buf id
  | Ast.Group inner_expr ->
      Buffer.add_string ctx.buf "(" ;
      unparse_expr ctx inner_expr ;
      Buffer.add_string ctx.buf ")"
  | Ast.BinaryOp {op; lhs; rhs} ->
      Buffer.add_string ctx.buf "(" ;
      unparse_factor ctx lhs ;
      let op_str = match op with Ast.Mul -> " * " | Ast.Div -> " / " in
      Buffer.add_string ctx.buf op_str ;
      unparse_factor ctx rhs ;
      Buffer.add_string ctx.buf ")"
  | Ast.Call {func; args} ->
      unparse_factor ctx func ;
      Buffer.add_string ctx.buf "(" ;
      let arg_strs =
        List.map
          (fun arg ->
            let buf = Buffer.create 16 in
            let arg_ctx = {buf; indent= 0} in
            unparse_expr arg_ctx arg ; Buffer.contents buf )
          args
      in
      Buffer.add_string ctx.buf (String.concat ", " arg_strs) ;
      Buffer.add_string ctx.buf ")"

let unparse_param (ctx : ctx) (param : Ast.param) : unit =
  Buffer.add_string ctx.buf param.name ;
  Buffer.add_string ctx.buf ": " ;
  unparse_ty ctx param.ty

let unparse_stmt (ctx : ctx) (stmt : Ast.stmt) : unit =
  match stmt with
  | Ast.Let (id, ty, expr) ->
      Printf.bprintf ctx.buf "%slet %s: " (String.make ctx.indent ' ') id ;
      unparse_ty ctx ty ;
      Buffer.add_string ctx.buf " = " ;
      unparse_expr ctx expr ;
      Buffer.add_string ctx.buf "\n"
  | Ast.Fun (id, func) ->
      Printf.bprintf ctx.buf "%sfun %s(" (String.make ctx.indent ' ') id ;
      let num_params : int = List.length func.params in
      List.iteri
        (fun i param ->
          unparse_param ctx param ;
          if i < num_params - 1 then Buffer.add_string ctx.buf ", " else () )
        func.params ;
      Buffer.add_string ctx.buf "): " ;
      unparse_ty ctx func.return_ty ;
      Buffer.add_string ctx.buf " = " ;
      unparse_expr ctx func.body ;
      Buffer.add_string ctx.buf "\n"

let unparse_stmts (stmts : Ast.stmt list) : string =
  let ctx = {buf= Buffer.create 1024; indent= 0} in
  List.iter (unparse_stmt ctx) stmts ;
  Buffer.contents ctx.buf
