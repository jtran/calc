module A = Calc.Ast
module R = Calc.Runtime

(* --- small AST constructors to keep the test cases readable --- *)
let lit (n : float) : A.expr = A.Factor (A.Literal n)

let evar (id : A.identifier) : A.expr = A.Factor (A.Variable id)

let add (l : A.expr) (r : A.expr) : A.expr =
  A.BinaryOp {op= A.Add; lhs= l; rhs= r}

let sub (l : A.expr) (r : A.expr) : A.expr =
  A.BinaryOp {op= A.Sub; lhs= l; rhs= r}

let call (f : A.identifier) (args : A.expr list) : A.expr =
  A.Factor (A.Call {func= A.Variable f; args})

(* --- assertions --- *)
let expect_number (label : string) (stmts : A.stmt list) (expected : float) :
    unit =
  match Calc.Evaluator.eval_stmts stmts with
  | Ok (R.Number n) ->
      if Float.abs (n -. expected) > 1e-9 then (
        Printf.eprintf "FAIL %s: expected %f, got %f\n" label expected n ;
        exit 1 )
  | Ok (R.Closure _) ->
      Printf.eprintf "FAIL %s: expected a number, got a closure\n" label ;
      exit 1
  | Error msg ->
      Printf.eprintf "FAIL %s: expected %f, got error %S\n" label expected msg ;
      exit 1

let expect_error (label : string) (stmts : A.stmt list) : unit =
  match Calc.Evaluator.eval_stmts stmts with
  | Error _ ->
      ()
  | Ok v ->
      Printf.eprintf "FAIL %s: expected an error, got %s\n" label
        (R.show_value v) ;
      exit 1

(* g(a) = a + 1 ; used by the call-related cases below *)
let g_func : A.func =
  { params= [{A.name= "a"; ty= A.Number}]
  ; return_ty= A.Number
  ; body= add (evar "a") (lit 1.0) }

(* f(b) = g(b) + 10 ; nested call exercising KReturnTo at two levels *)
let f_func : A.func =
  { params= [{A.name= "b"; ty= A.Number}]
  ; return_ty= A.Number
  ; body= add (call "g" [evar "b"]) (lit 10.0) }

(* Build a left-nested chain of [n] additions: (((0 + 1) + 1) + ... ) + 1.
   The recursive evaluator overflows the native stack here; the iterative one
   uses a heap-allocated continuation stack and returns [n]. *)
let deep_expr (n : int) : A.expr =
  let rec build i acc =
    if i = 0 then acc else build (i - 1) (add acc (lit 1.0))
  in
  build n (lit 0.0)

let () =
  (* arithmetic + operator precedence encoded in the AST: 1 + 2 * 3 = 7 *)
  expect_number "literal" [A.Let ("r", A.Number, lit 42.0)] 42.0 ;
  expect_number "precedence (1 + 2*3)"
    [ A.Let
        ( "r"
        , A.Number
        , add (lit 1.0)
            (A.Factor
               (A.BinaryOp {op= A.Mul; lhs= A.Literal 2.0; rhs= A.Literal 3.0})
            ) ) ]
    7.0 ;
  expect_number "subtraction (10 - 4)"
    [A.Let ("r", A.Number, sub (lit 10.0) (lit 4.0))]
    6.0 ;
  expect_number "division (8 / 2)"
    [ A.Let
        ( "r"
        , A.Number
        , A.Factor
            (A.BinaryOp {op= A.Div; lhs= A.Literal 8.0; rhs= A.Literal 2.0}) )
    ]
    4.0 ;
  (* grouping changes precedence: (1 + 2) * 3 = 9 *)
  expect_number "grouping ((1 + 2) * 3)"
    [ A.Let
        ( "r"
        , A.Number
        , A.Factor
            (A.BinaryOp
               { op= A.Mul
               ; lhs= A.Group (add (lit 1.0) (lit 2.0))
               ; rhs= A.Literal 3.0 } ) ) ]
    9.0 ;
  (* variable binding then reference across statements: x = 5; y = x + 2 *)
  expect_number "variable reference"
    [ A.Let ("x", A.Number, lit 5.0)
    ; A.Let ("y", A.Number, add (evar "x") (lit 2.0)) ]
    7.0 ;
  (* function definition + nested call: f(5) = g(5) + 10 = 16 *)
  expect_number "nested call"
    [ A.Fun ("g", g_func)
    ; A.Fun ("f", f_func)
    ; A.Let ("result", A.Number, call "f" [lit 5.0]) ]
    16.0 ;
  (* error cases *)
  expect_error "unbound identifier" [A.Let ("x", A.Number, evar "missing")] ;
  expect_error "call non-function"
    [A.Let ("x", A.Number, lit 5.0); A.Let ("y", A.Number, call "x" [])] ;
  expect_error "arity mismatch"
    [A.Fun ("g", g_func); A.Let ("r", A.Number, call "g" [lit 1.0; lit 2.0])] ;
  (* the payoff: a depth that overflows native recursion must succeed here *)
  expect_number "deep nesting (1,000,000)"
    [A.Let ("r", A.Number, deep_expr 1_000_000)]
    1_000_000.0 ;
  print_endline "All evaluator tests passed."
