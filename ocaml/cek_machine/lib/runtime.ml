type env = (Ast.identifier, value) Hashtbl.t

and value = Number of float | Closure of Ast.func * env

let show_value (v : value) : string =
  match v with
  | Number n ->
      Printf.sprintf "%f" n
  | Closure (func, _) ->
      Printf.sprintf "(Closure %s)" (Ast.show_func func)

exception RuntimeError of string
