type typ = Number | Arrow of typ list * typ [@@deriving show]

type identifier = string [@@deriving show]

type stmt = Let of identifier * typ * expr | Fun of identifier * func

and func = {params: params; return_ty: typ; body: expr} [@@deriving show]

and params = param list

and param = {name: identifier; ty: typ} [@@deriving show]

and expr =
  | Factor of factor
  | BinaryOp of {op: term_binary_op; lhs: expr; rhs: expr}

and term_binary_op = Add | Sub

and factor =
  | Literal of float
  | Variable of identifier
  | Group of expr
  | BinaryOp of {op: factor_binary_op; lhs: factor; rhs: factor}
  | Call of {func: factor; args: expr list}

and factor_binary_op = Mul | Div
