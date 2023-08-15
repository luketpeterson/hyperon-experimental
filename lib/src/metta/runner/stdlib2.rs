use crate::*;
use crate::matcher::MatchResultIter;
use crate::space::*;
use crate::metta::*;
use crate::metta::text::Tokenizer;
use crate::metta::runner::Metta;
use crate::metta::types::{get_atom_types, get_meta_type};

use std::fmt::Display;
use std::path::PathBuf;
use regex::Regex;

use super::arithmetics::*;

#[derive(Clone, PartialEq, Debug)]
pub struct GetTypeOp {}

impl Display for GetTypeOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "get-type")
    }
}

impl Grounded for GetTypeOp {
    fn type_(&self) -> Atom {
        Atom::expr([ARROW_SYMBOL, ATOM_TYPE_ATOM, ATOM_TYPE_ATOM, ATOM_TYPE_ATOM])
    }

    fn execute(&self, args: &[Atom]) -> Result<Vec<Atom>, ExecError> {
        let arg_error = || ExecError::from("get-type expects single atom as an argument");
        let atom = args.get(0).ok_or_else(arg_error)?;
        let space = args.get(1).ok_or_else(arg_error)?;
        let space = Atom::as_gnd::<DynSpace>(space).ok_or("match expects a space as the first argument")?;
        Ok(get_atom_types(space, atom))
    }

    fn match_(&self, other: &Atom) -> MatchResultIter {
        match_by_equality(self, other)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct GetMetaTypeOp { }

impl Display for GetMetaTypeOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "get-metatype")
    }
}

impl Grounded for GetMetaTypeOp {
    fn type_(&self) -> Atom {
        Atom::expr([ARROW_SYMBOL, ATOM_TYPE_ATOM, ATOM_TYPE_ATOM])
    }

    fn execute(&self, args: &[Atom]) -> Result<Vec<Atom>, ExecError> {
        let arg_error = || ExecError::from("get-metatype expects single atom as an argument");
        let atom = args.get(0).ok_or_else(arg_error)?;

        Ok(vec![get_meta_type(&atom)])
    }

    fn match_(&self, other: &Atom) -> MatchResultIter {
        match_by_equality(self, other)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct IfEqualOp { }

impl Display for IfEqualOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "if-equal")
    }
}

impl Grounded for IfEqualOp {
    fn type_(&self) -> Atom {
        Atom::expr([ARROW_SYMBOL, ATOM_TYPE_ATOM, ATOM_TYPE_ATOM, ATOM_TYPE_ATOM, ATOM_TYPE_ATOM, ATOM_TYPE_ATOM])
    }

    fn execute(&self, args: &[Atom]) -> Result<Vec<Atom>, ExecError> {
        let arg_error = || ExecError::from("if-equal expects <atom> <pattern> <then> <else> as an argument");
        let atom = args.get(0).ok_or_else(arg_error)?;
        let pattern = args.get(1).ok_or_else(arg_error)?;
        let then = args.get(2).ok_or_else(arg_error)?;
        let else_ = args.get(3).ok_or_else(arg_error)?;

        if crate::matcher::atoms_are_equivalent(atom, pattern) {
            Ok(vec![then.clone()])
        } else {
            Ok(vec![else_.clone()])
        }
    }

    fn match_(&self, other: &Atom) -> MatchResultIter {
        match_by_equality(self, other)
    }
}


fn regex(regex: &str) -> Regex {
    Regex::new(regex).unwrap()
}

pub fn register_common_tokens(metta: &Metta) {
    let tokenizer = &metta.tokenizer;
    let mut tref = tokenizer.borrow_mut();

    let get_type_op = Atom::gnd(GetTypeOp{});
    tref.register_token(regex(r"get-type"), move |_| { get_type_op.clone() });
    let get_meta_type_op = Atom::gnd(GetMetaTypeOp{});
    tref.register_token(regex(r"get-metatype"), move |_| { get_meta_type_op.clone() });
    let is_equivalent = Atom::gnd(IfEqualOp{});
    tref.register_token(regex(r"if-equal"), move |_| { is_equivalent.clone() });
}

pub fn register_runner_tokens(metta: &Metta, _cwd: PathBuf) {
    let _space = &metta.space;
    let tokenizer = &metta.tokenizer;

    let mut tref = tokenizer.borrow_mut();

    // &self should be updated
    // TODO: adding &self might be done not by stdlib, but by MeTTa itself.
    // TODO: adding &self introduces self referencing and thus prevents space
    // from being freed. There are two options to eliminate this. (1) use weak
    // pointer and somehow use the same type to represent weak and strong
    // pointers to the atomspace. (2) resolve &self in GroundingSpace::query
    // method without adding it into container.
    let self_atom = Atom::gnd(metta.space.clone());
    tref.register_token(regex(r"&self"), move |_| { self_atom.clone() });
}

pub fn register_rust_tokens(metta: &Metta) {
    let mut rust_tokens = Tokenizer::new();
    let tref = &mut rust_tokens;

    tref.register_token(regex(r"\d+"),
        |token| { Atom::gnd(Number::from_int_str(token)) });
    tref.register_token(regex(r"\d+(.\d+)([eE][\-\+]?\d+)?"),
        |token| { Atom::gnd(Number::from_float_str(token)) });
    tref.register_token(regex(r"True|False"),
        |token| { Atom::gnd(Bool::from_str(token)) });
    let sum_op = Atom::gnd(SumOp{});
    tref.register_token(regex(r"\+"), move |_| { sum_op.clone() });
    let sub_op = Atom::gnd(SubOp{});
    tref.register_token(regex(r"\-"), move |_| { sub_op.clone() });
    let mul_op = Atom::gnd(MulOp{});
    tref.register_token(regex(r"\*"), move |_| { mul_op.clone() });
    let div_op = Atom::gnd(DivOp{});
    tref.register_token(regex(r"/"), move |_| { div_op.clone() });
    let mod_op = Atom::gnd(ModOp{});
    tref.register_token(regex(r"%"), move |_| { mod_op.clone() });

    metta.tokenizer.borrow_mut().move_front(&mut rust_tokens);
}

pub static METTA_CODE: &'static str = "

;`$then`, `$else` should be of `Atom` type to avoid evaluation
; and infinite cycle in inference
(: if (-> Bool Atom Atom $t))
(= (if True $then $else) $then)
(= (if False $then $else) $else)

(: Error (-> Atom Atom ErrorType))

(= (if-non-empty-expression $atom $then $else)
  (chain (eval (get-metatype $atom)) $type
    (eval (if-equal $type Expression
      (eval (if-equal $atom () $else $then))
      $else ))))

(= (if-decons $atom $head $tail $then $else)
  (eval (if-non-empty-expression $atom
    (chain (decons $atom) $list
      (match $list ($head $tail) $then $else) )
    $else )))

(= (if-empty $atom $then $else)
  (eval (if-equal $atom Empty $then $else)))

(= (if-error $atom $then $else)
  (eval (if-decons $atom $head $_
    (eval (if-equal $head Error $then $else))
    $else )))

(= (return-on-error $atom $then)
  (eval (if-empty $atom Empty
    (eval (if-error $atom $atom
      $then )))))

(= (car $atom)
  (eval (if-decons $atom $head $_
    $head
    (Error (car $atom) \"car expects a non-empty expression as an argument\") )))

(= (switch $atom $cases)
  (chain (decons $cases) $list (eval (switch-internal $atom $list))))
(= (switch-internal $atom (($pattern $template) $tail))
  (match $atom $pattern $template (eval (switch $atom $tail))))

(= (subst $atom $var $templ)
  (match $atom $var $templ
    (Error (subst $atom $var $templ)
      \"subst expects a variable as a second argument\") ))

(= (reduce $atom $var $templ)
  (chain (eval $atom) $res
    (eval (if-error $res $res
      (eval (if-empty $res
        (eval (subst $atom $var $templ))
        (eval (reduce $res $var $templ)) ))))))

(= (type-cast $atom $type $space)
  (chain (eval (get-type $atom $space)) $actual-type
    (eval (switch ($actual-type $type)
      (
        ((%Undefined% $_) $atom)
        (($_ %Undefined%) $atom)
        (($type $_) $atom)
        ($_ (Error $atom BadType)) )))))

(= (is-function $type)
  (chain (eval (get-metatype $type)) $meta
    (eval (switch ($type $meta)
      (
        (($_ Expression)
          (chain (eval (car $type)) $head
            (match $head -> True False) ))
        ($_ False) )))))

(= (interpret $atom $type $space)
  (chain (eval (get-metatype $atom)) $meta
    (eval (switch ($type $meta)
      (
        ((Atom $_meta) $atom)
        (($meta $meta) $atom)
        (($_type Variable) $atom)

        (($_type Symbol) (eval (type-cast $atom $type $space)))
        (($_type Grounded) (eval (type-cast $atom $type $space)))
        (($_type Expression) (eval (interpret-expression $atom $type $space))) )))))

(= (interpret-expression $atom $type $space)
  (eval (if-decons $atom $op $args
    (chain (eval (get-type $op $space)) $op-type
      (chain (eval (is-function $op-type)) $is-func
        (match $is-func True
          (chain (eval (interpret-func $atom $op-type $space)) $reduced-atom
            (eval (call $reduced-atom $type $space)) )
          (chain (eval (interpret-tuple $atom $space)) $reduced-atom
            (eval (call $reduced-atom $type $space)) ))))
    (eval (type-cast $atom $type $space)) )))

(= (interpret-func $expr $type $space)
  (eval (if-decons $expr $op $args
    (chain (eval (interpret $op $type $space)) $reduced-op
      (eval (return-on-error $reduced-op
        (eval (if-decons $type $arrow $arg-types
          (chain (eval (interpret-args $expr $args $arg-types $space)) $reduced-args
            (eval (return-on-error $reduced-args
              (cons $reduced-op $reduced-args) )))
          (Error $type \"Function type expected\") )))))
    (Error $expr \"Non-empty expression atom is expected\") )))

(= (interpret-args $atom $args $arg-types $space)
  (match $args ()
    (match $arg-types ($ret) () (Error $atom BadType))
    (eval (if-decons $args $head $tail
      (eval (if-decons $arg-types $head-type $tail-types
        (chain (eval (interpret $head $head-type $space)) $reduced-head
          ; check that head was changed otherwise Error or Empty in the head
          ; can be just an argument which is passed by intention
          (eval (if-equal $reduced-head $head
            (eval (interpret-args-tail $atom $reduced-head $tail $tail-types $space))
            (eval (return-on-error $reduced-head
              (eval (interpret-args-tail $atom $reduced-head $tail $tail-types $space)) )))))
        (Error $atom BadType) ))
      (Error (interpret-atom $atom $args $arg-types $space)
        \"Non-empty expression atom is expected\") ))))

(= (interpret-args-tail $atom $head $args-tail $args-tail-types $space)
  (chain (eval (interpret-args $atom $args-tail $args-tail-types $space)) $reduced-tail
    (eval (return-on-error $reduced-tail
      (cons $head $reduced-tail) ))))

(= (interpret-tuple $atom $space)
  (match $atom ()
    $atom
    (eval (if-decons $atom $head $tail
      (chain (eval (interpret $head %Undefined% $space)) $rhead
        (chain (eval (interpret-tuple $tail $space)) $rtail
          (cons $rhead $rtail) ))
      (Error (interpret-tuple $atom $space) \"Non-empty expression atom is expected as an argument\") ))))

(= (call $atom $type $space)
  (chain (eval $atom) $result
    (eval (if-empty $result $atom
      (eval (if-error $result $result
        (eval (interpret $result $type $space)) ))))))

";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metta::runner::new_metta_rust;
    use crate::matcher::atoms_are_equivalent;

    fn run_program(program: &str) -> Result<Vec<Vec<Atom>>, String> {
        let metta = new_metta_rust();
        metta.run(&mut SExprParser::new(program))
    }

    #[test]
    fn get_type_op() {
        let space = DynSpace::new(metta_space("
            (: B Type)
            (: C Type)
            (: A B)
            (: A C)
        "));

        let get_type_op = GetTypeOp{};
        assert_eq_no_order!(get_type_op.execute(&mut vec![sym!("A"), expr!({space.clone()})]).unwrap(),
            vec![sym!("B"), sym!("C")]);
    }

    #[test]
    fn get_type_op_non_valid_atom() {
        let space = DynSpace::new(metta_space("
            (: f (-> Number String))
            (: 42 Number)
            (: \"test\" String)
        "));

        let get_type_op = GetTypeOp{};
        assert_eq_no_order!(get_type_op.execute(&mut vec![expr!("f" "42"), expr!({space.clone()})]).unwrap(),
            vec![sym!("String")]);
        assert_eq_no_order!(get_type_op.execute(&mut vec![expr!("f" "\"test\""), expr!({space.clone()})]).unwrap(),
            Vec::<Atom>::new());
    }


    #[test]
    fn metta_car() {
        let result = run_program("!(eval (car (A $b)))");
        assert_eq!(result, Ok(vec![vec![expr!("A")]]));
        let result = run_program("!(eval (car ($a B)))");
        //assert_eq!(result, Ok(vec![vec![expr!(a)]]));
        assert!(result.is_ok_and(|res| res.len() == 1 && res[0].len() == 1 &&
            atoms_are_equivalent(&res[0][0], &expr!(a))));
        let result = run_program("!(eval (car ()))");
        assert_eq!(result, Ok(vec![vec![expr!("Error" ("car" ()) "\"car expects a non-empty expression as an argument\"")]]));
        let result = run_program("!(eval (car A))");
        assert_eq!(result, Ok(vec![vec![expr!("Error" ("car" "A") "\"car expects a non-empty expression as an argument\"")]]));
    }

    #[test]
    fn metta_switch() {
        let result = run_program("!(eval (switch (A $b) ( (($a B) ($b $a)) ((B C) (C B)) )))");
        assert_eq!(result, Ok(vec![vec![expr!("B" "A")]]));
        let result = run_program("!(eval (switch (A $b) ( ((B C) (C B)) (($a B) ($b $a)) )))");
        assert_eq!(result, Ok(vec![vec![expr!("B" "A")]]));
        let result = run_program("!(eval (switch (A $b) ( ((B C) (C B)) ((D E) (E B)) )))");
        assert_eq!(result, Ok(vec![vec![]]));
    }

    #[test]
    fn metta_is_function() {
        let result = run_program("!(eval (is-function (-> $t)))");
        assert_eq!(result, Ok(vec![vec![expr!({Bool(true)})]]));
        let result = run_program("!(eval (is-function (A $t)))");
        assert_eq!(result, Ok(vec![vec![expr!({Bool(false)})]]));
        let result = run_program("!(eval (is-function %Undefined%))");
        assert_eq!(result, Ok(vec![vec![expr!({Bool(false)})]]));
    }

    #[test]
    fn metta_reduce_chain() {
        assert_eq!(run_program("
            (= (foo $x) (bar $x))
            (= (bar $x) (baz $x))
            (= (baz $x) $x)
            !(eval (reduce (foo A) $x (reduced $x)))
        "), Ok(vec![vec![expr!("reduced" "A")]]));
        assert_eq!(run_program("
            !(eval (reduce (foo A) $x (reduced $x)))
        "), Ok(vec![vec![expr!("reduced" ("foo" "A"))]]));
        assert_eq!(run_program("
            (= (foo A) (Error (foo A) \"Test error\"))
            !(eval (reduce (foo A) $x (reduced $x)))
        "), Ok(vec![vec![expr!("Error" ("foo" "A") "\"Test error\"")]]));
    }

    #[test]
    fn metta_reduce_reduce() {
        assert_eq!(run_program("
            (= (foo $x) (reduce (bar $x) $t $t))
            (= (bar $x) $x)
            !(eval (reduce (foo A) $x $x))
        "), Ok(vec![vec![expr!("A")]]));
    }

    #[test]
    fn metta_type_cast() {
        assert_eq!(run_program("(: a A) !(eval (type-cast a A &self))"), Ok(vec![vec![expr!("a")]]));
        assert_eq!(run_program("(: a A) !(eval (type-cast a B &self))"), Ok(vec![vec![expr!("Error" "a" "BadType")]]));
        assert_eq!(run_program("(: a A) !(eval (type-cast a %Undefined% &self))"), Ok(vec![vec![expr!("a")]]));
        assert_eq!(run_program("!(eval (type-cast a B &self))"), Ok(vec![vec![expr!("a")]]));
        assert_eq!(run_program("!(eval (type-cast 42 Number &self))"), Ok(vec![vec![expr!({Number::Integer(42)})]]));
        assert_eq!(run_program("!(eval (type-cast 42 %Undefined% &self))"), Ok(vec![vec![expr!({Number::Integer(42)})]]));
    }

    #[test]
    fn metta_interpret_single_atom_as_atom() {
        let result = run_program("!(eval (interpret A Atom &self))");
        assert_eq!(result, Ok(vec![vec![expr!("A")]]));
    }

    #[test]
    fn metta_interpret_single_atom_as_meta_type() {
        assert_eq!(run_program("!(eval (interpret A Symbol &self))"), Ok(vec![vec![expr!("A")]]));
        assert_eq!(run_program("!(eval (interpret $x Variable &self))"), Ok(vec![vec![expr!(x)]]));
        assert_eq!(run_program("!(eval (interpret (A B) Expression &self))"), Ok(vec![vec![expr!("A" "B")]]));
        assert_eq!(run_program("!(eval (interpret 42 Grounded &self))"), Ok(vec![vec![expr!({Number::Integer(42)})]]));
    }

    #[test]
    fn metta_interpret_symbol_or_grounded_value_as_type() {
        assert_eq!(run_program("(: a A) !(eval (interpret a A &self))"), Ok(vec![vec![expr!("a")]]));
        assert_eq!(run_program("(: a A) !(eval (interpret a B &self))"), Ok(vec![vec![expr!("Error" "a" "BadType")]]));
        assert_eq!(run_program("!(eval (interpret 42 Number &self))"), Ok(vec![vec![expr!({Number::Integer(42)})]]));
    }

    #[test]
    fn metta_interpret_variable_as_type() {
        assert_eq!(run_program("!(eval (interpret $x %Undefined% &self))"), Ok(vec![vec![expr!(x)]]));
        assert_eq!(run_program("!(eval (interpret $x SomeType &self))"), Ok(vec![vec![expr!(x)]]));
    }

    #[test]
    fn metta_interpret_empty_expression_as_type() {
        assert_eq!(run_program("!(eval (interpret () %Undefined% &self))"), Ok(vec![vec![expr!(())]]));
        assert_eq!(run_program("!(eval (interpret () SomeType &self))"), Ok(vec![vec![expr!(())]]));
    }

    #[test]
    fn metta_interpret_children() {
        assert_eq!(run_program("!(eval (interpret-tuple () &self))"), Ok(vec![vec![expr!(())]]));
        assert_eq!(run_program("!(eval (interpret-tuple (a) &self))"), Ok(vec![vec![expr!(("a"))]]));
        assert_eq!(run_program("!(eval (interpret-tuple (a b) &self))"), Ok(vec![vec![expr!(("a" "b"))]]));
        assert_eq!(run_program("
            (= (foo $x) (bar $x))
            (= (bar $x) (baz $x))
            (= (baz $x) $x)
            !(eval (interpret-tuple ((foo A) (foo B)) &self))
        "), Ok(vec![vec![expr!("A" "B")]]));
    }

    #[test]
    fn metta_interpret_expression_as_type() {
        assert_eq!(run_program("(= (foo $x) $x) !(eval (interpret (foo a) %Undefined% &self))"), Ok(vec![vec![expr!("a")]]));
        assert_eq!(run_program("!(eval (interpret (foo a) %Undefined% &self))"), Ok(vec![vec![expr!("foo" "a")]]));
        assert_eq!(run_program("!(eval (interpret () SomeType &self))"), Ok(vec![vec![expr!(())]]));
    }


    #[test]
    fn test_frog_reasoning() {
        let program = "
            (= (and True True) True)

            (= (is Fritz croaks) True)
            (= (is Fritz eats-flies) True)

            (= (is Tweety chirps) True)
            (= (is Tweety yellow) True)
            (= (is Tweety eats-flies) True)

            !(eval (interpret (if (and (is $x croaks) (is $x eats-flies)) (= (is $x frog) True) Empty) %Undefined% &self))
        ";

        assert_eq!(run_program(program),
            Ok(vec![vec![expr!("=" ("is" "Fritz" "frog") {Bool(true)})]]));
    }

    #[test]
    fn test_match_all() {
        let program = "
            (= (color) blue)
            (= (color) red)
            (= (color) green)

            !(eval (interpret (color) %Undefined% &self))
        ";

        assert_eq_metta_results!(run_program(program),
            Ok(vec![vec![expr!("blue"), expr!("red"), expr!("green")]]));
    }

    #[test]
    fn test_variable_keeps_value_in_different_sub_expressions() {
        let program = "
            (= (eq $x $x) True)
            (= (plus Z $y) $y)
            (= (plus (S $k) $y) (S (plus $k $y)))

            !(eval (interpret (eq (plus Z $n) $n) %Undefined% &self))
            !(eval (interpret (eq (plus (S Z) $n) $n) %Undefined% &self))
        ";

        assert_eq_metta_results!(run_program(program),
            Ok(vec![vec![expr!({Bool(true)})], vec![expr!("eq" ("S" n) n)]]));
    }

    #[test]
    fn test_variable_defined_via_variabe() {
        let program = "
            (= (myif T $y) $y)
            (= (not F) T)
            (= (a $z) (not (b $z)))
            (= (b d) F)

            !(eval (interpret (myif (a $x) $x) %Undefined% &self))
        ";

        assert_eq_metta_results!(run_program(program),
            Ok(vec![vec![expr!("d")]]));
    }

    #[test]
    fn test_variable_name_conflict() {
        let program = "
            (= (a ($W)) True)

            !(eval (interpret (a $W) %Undefined% &self))
        ";

        assert_eq_metta_results!(run_program(program),
            Ok(vec![vec![expr!({Bool(true)})]]));
    }

    #[test]
    fn test_variable_name_conflict_renaming() {
        let program = "
            (= (b ($x $y)) (c $x $y))

            !(eval (interpret (a (b $a) $x $y) %Undefined% &self))
        ";

        let result = run_program(program);
        assert!(result.is_ok_and(|res| res.len() == 1 && res[0].len() == 1 &&
            atoms_are_equivalent(&res[0][0], &expr!("a" ("c" a b) c d))));
    }

    #[test]
    fn test_operation_is_expression() {
        let program = "
            (: foo (-> (-> A A)))
            (: a A)
            (= (foo) bar)
            (= (bar $x) $x)

            !(eval (interpret ((foo) a) %Undefined% &self))
        ";

        assert_eq_metta_results!(run_program(program), Ok(vec![vec![expr!("a")]]));
    }

    #[test]
    fn test_return_bad_type_error() {
        let program = "
            (: myAtom myType)
            (: id_a (-> A A))
            (= (id_a $a) $a)

            !(eval (interpret (id_a myAtom) %Undefined% &self))
            ;!(id_num myatom)
        ";

        assert_eq!(run_program(program),
            Ok(vec![vec![expr!("Error" "myAtom" "BadType")]]));
        //assert_eq!(interpret(&space, &expr!({ID_NUM} "myAtom")),
            //Ok(vec![Atom::expr([ERROR_SYMBOL, sym!("myAtom"), BAD_TYPE_SYMBOL])]));
    }
}
