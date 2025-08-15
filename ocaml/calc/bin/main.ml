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

let _ =
  match Calc.Tc.check_stmts stmts with
  | Ok env ->
      Printf.printf "Type checking succeeded.\n" ;
      Hashtbl.iter
        (fun id ty -> Printf.printf "  %s : %s\n" id (Calc.Ast.show_typ ty))
        env
  | Error msg ->
      Printf.eprintf "Type error: %s\n" msg
