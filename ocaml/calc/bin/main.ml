let stmts =
  [ Calc.Ast.Let ("x", Calc.Ast.Number, Calc.Ast.Factor (Calc.Ast.Literal 1.0))
  ; Calc.Ast.Let
      ( "y"
      , Calc.Ast.Number
      , Calc.Ast.BinaryOp
          { op= Calc.Ast.Add
          ; lhs= Calc.Ast.Factor (Calc.Ast.Variable "x")
          ; rhs= Calc.Ast.Factor (Calc.Ast.Literal 2.0) } )
  ; Calc.Ast.Fun
      ( "add"
      , { Calc.Ast.params=
            [{name= "a"; ty= Calc.Ast.Number}; {name= "b"; ty= Calc.Ast.Number}]
        ; return_ty=
            Calc.Ast.Number
            (* ; return_ty= Calc.Ast.Arrow ([Calc.Ast.Number], Calc.Ast.Number) *)
        ; body=
            Calc.Ast.BinaryOp
              { op= Calc.Ast.Add
              ; lhs= Calc.Ast.Factor (Calc.Ast.Variable "a")
              ; rhs= Calc.Ast.Factor (Calc.Ast.Variable "b") } } )
  ; Calc.Ast.Let
      ( "answer"
      , Calc.Ast.Number
      , Calc.Ast.Factor
          (Calc.Ast.Call
             { func= Calc.Ast.Variable "add"
             ; args=
                 [ Calc.Ast.Factor (Calc.Ast.Variable "x")
                 ; Calc.Ast.Factor (Calc.Ast.Variable "y") ] } ) ) ]

let type_check_result =
  match Calc.Tc.check_stmts stmts with
  | Ok env ->
      Printf.printf "Type checking succeeded.\n" ;
      Hashtbl.iter
        (fun id ty -> Printf.printf "  %s : %s\n" id (Calc.Ast.show_typ ty))
        env ;
      Some ()
  | Error msg ->
      Printf.eprintf "Type error: %s\n" msg ;
      None

let _ =
  match type_check_result with
  | Some () -> (
      let eval_result = Calc.Evaluator.eval_stmts stmts in
      match eval_result with
      | Ok value ->
          Printf.printf "Evaluation succeeded: %s\n"
            (Calc.Runtime.show_value value)
      | Error msg ->
          Printf.eprintf "Evaluation error: %s\n" msg )
  | None ->
      ()
