//! Type theory: simply-typed lambda calculus types, type inference (Hindley-Milner).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Types in the simply-typed lambda calculus.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Type {
    /// Base type (Int, Bool, etc.)
    Base(String),
    /// Function type: T1 -> T2
    Arrow(Box<Type>, Box<Type>),
    /// Type variable (for inference)
    Var(usize),
    /// Universal quantification: ∀α.τ
    Forall(usize, Box<Type>),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Base(s) => write!(f, "{}", s),
            Type::Arrow(a, b) => {
                match a.as_ref() {
                    Type::Arrow(_, _) => write!(f, "({}) -> {}", a, b),
                    _ => write!(f, "{} -> {}", a, b),
                }
            }
            Type::Var(i) => write!(f, "t{}", i),
            Type::Forall(i, t) => write!(f, "∀t{}.{}", i, t),
        }
    }
}

impl Type {
    pub fn int() -> Self { Type::Base("Int".into()) }
    pub fn bool() -> Self { Type::Base("Bool".into()) }
    pub fn arrow(a: Type, b: Type) -> Self {
        Type::Arrow(Box::new(a), Box::new(b))
    }
    pub fn forall(i: usize, t: Type) -> Self {
        Type::Forall(i, Box::new(t))
    }
}

/// Typed expressions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Var(String),
    Abs(String, Type, Box<Expr>),
    App(Box<Expr>, Box<Expr>),
    Let(String, Box<Expr>, Box<Expr>),
    Lit(Literal),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Int(i64),
    Bool(bool),
}

/// Type checking/inference context.
#[derive(Clone, Debug)]
pub struct TypeEnv {
    /// Variable bindings with their types.
    bindings: HashMap<String, Type>,
    /// Next fresh type variable index.
    next_var: usize,
}

impl TypeEnv {
    pub fn new() -> Self {
        TypeEnv {
            bindings: HashMap::new(),
            next_var: 0,
        }
    }

    pub fn with(mut self, name: String, ty: Type) -> Self {
        self.bindings.insert(name, ty);
        self
    }

    pub fn fresh_var(&mut self) -> Type {
        let v = Type::Var(self.next_var);
        self.next_var += 1;
        v
    }

    /// Hindley-Milner type inference.
    pub fn infer(&mut self, expr: &Expr) -> Result<Type, String> {
        match expr {
            Expr::Var(name) => {
                self.bindings.get(name).cloned().ok_or_else(|| format!("unbound variable: {}", name))
            }
            Expr::Lit(lit) => Ok(match lit {
                Literal::Int(_) => Type::int(),
                Literal::Bool(_) => Type::bool(),
            }),
            Expr::Abs(param, param_ty, body) => {
                let mut inner_env = self.clone();
                inner_env.bindings.insert(param.clone(), param_ty.clone());
                let body_ty = inner_env.infer(body)?;
                Ok(Type::arrow(param_ty.clone(), body_ty))
            }
            Expr::App(func, arg) => {
                let func_ty = self.infer(func)?;
                let arg_ty = self.infer(arg)?;
                let ret_ty = self.fresh_var();
                let expected = Type::arrow(arg_ty.clone(), ret_ty.clone());
                unify(&func_ty, &expected).map_err(|e| format!("type mismatch: {}", e))?;
                // Apply the substitution to get the concrete return type
                let subst = unify(&func_ty, &expected).map_err(|e| format!("type mismatch: {}", e))?;
                Ok(apply_subst(&ret_ty, &subst))
            }
            Expr::Let(name, value, body) => {
                let val_ty = self.infer(value)?;
                let mut inner_env = self.clone();
                inner_env.bindings.insert(name.clone(), val_ty);
                inner_env.infer(body)
            }
        }
    }
}

/// Substitution mapping type variables to types.
pub type Substitution = HashMap<usize, Type>;

/// Apply substitution to a type.
pub fn apply_subst(ty: &Type, subst: &Substitution) -> Type {
    match ty {
        Type::Base(_) | Type::Var(_) if matches!(ty, Type::Var(i) if subst.contains_key(i)) => {
            if let Type::Var(i) = ty {
                subst[i].clone()
            } else {
                ty.clone()
            }
        }
        Type::Var(_) => ty.clone(),
        Type::Base(_) => ty.clone(),
        Type::Arrow(a, b) => Type::arrow(apply_subst(a, subst), apply_subst(b, subst)),
        Type::Forall(i, t) => {
            let mut new_subst = subst.clone();
            new_subst.remove(i);
            Type::forall(*i, apply_subst(t, &new_subst))
        }
    }
}

/// Unify two types, returning a substitution or error.
pub fn unify(t1: &Type, t2: &Type) -> Result<Substitution, String> {
    match (t1, t2) {
        (Type::Base(a), Type::Base(b)) if a == b => Ok(HashMap::new()),
        (Type::Var(i), t) | (t, Type::Var(i)) => {
            if let Type::Var(j) = t {
                if i == j { return Ok(HashMap::new()); }
            }
            if occurs(*i, t) {
                return Err(format!("infinite type: t{} ~ {}", i, t));
            }
            let mut s = HashMap::new();
            s.insert(*i, t.clone());
            Ok(s)
        }
        (Type::Arrow(a1, b1), Type::Arrow(a2, b2)) => {
            let s1 = unify(a1, a2)?;
            let b1_sub = apply_subst(b1, &s1);
            let b2_sub = apply_subst(b2, &s1);
            let s2 = unify(&b1_sub, &b2_sub)?;
            let mut combined = s1;
            combined.extend(s2);
            Ok(combined)
        }
        _ => Err(format!("cannot unify {} with {}", t1, t2)),
    }
}

/// Occurs check: does variable `i` appear in `ty`?
pub fn occurs(i: usize, ty: &Type) -> bool {
    match ty {
        Type::Var(j) => i == *j,
        Type::Base(_) => false,
        Type::Arrow(a, b) => occurs(i, a) || occurs(i, b),
        Type::Forall(j, t) => i != *j && occurs(i, t),
    }
}

/// Generalize a type by quantifying free variables.
pub fn generalize(env: &TypeEnv, ty: &Type) -> Type {
    let env_vars = free_type_vars_types(env.bindings.values());
    let ty_vars = free_type_vars(ty);
    let mut result = ty.clone();
    for v in ty_vars {
        if !env_vars.contains(&v) {
            result = Type::forall(v, result);
        }
    }
    result
}

fn free_type_vars(ty: &Type) -> Vec<usize> {
    match ty {
        Type::Var(i) => vec![*i],
        Type::Base(_) => vec![],
        Type::Arrow(a, b) => {
            let mut v = free_type_vars(a);
            v.extend(free_type_vars(b));
            v.sort();
            v.dedup();
            v
        }
        Type::Forall(i, t) => {
            let mut v = free_type_vars(t);
            v.retain(|x| x != i);
            v
        }
    }
}

fn free_type_vars_types<'a>(types: impl Iterator<Item = &'a Type>) -> Vec<usize> {
    let mut vars = Vec::new();
    for t in types {
        vars.extend(free_type_vars(t));
    }
    vars.sort();
    vars.dedup();
    vars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_display() {
        let t = Type::arrow(Type::int(), Type::bool());
        assert_eq!(format!("{}", t), "Int -> Bool");
    }

    #[test]
    fn test_infer_identity() {
        let expr = Expr::Abs("x".into(), Type::Var(0), Box::new(Expr::Var("x".into())));
        let mut env = TypeEnv::new();
        // Identity: λx:α.x : α -> α
        // With explicit annotation
        let expr_typed = Expr::Abs("x".into(), Type::int(), Box::new(Expr::Var("x".into())));
        let ty = env.infer(&expr_typed).unwrap();
        assert_eq!(ty, Type::arrow(Type::int(), Type::int()));
    }

    #[test]
    fn test_infer_application() {
        // (λx:Int. x) 42
        let expr = Expr::App(
            Box::new(Expr::Abs("x".into(), Type::int(), Box::new(Expr::Var("x".into())))),
            Box::new(Expr::Lit(Literal::Int(42))),
        );
        let mut env = TypeEnv::new();
        let ty = env.infer(&expr).unwrap();
        assert_eq!(ty, Type::int());
    }

    #[test]
    fn test_infer_let() {
        // let x = 42 in x
        let expr = Expr::Let(
            "x".into(),
            Box::new(Expr::Lit(Literal::Int(42))),
            Box::new(Expr::Var("x".into())),
        );
        let mut env = TypeEnv::new();
        let ty = env.infer(&expr).unwrap();
        assert_eq!(ty, Type::int());
    }

    #[test]
    fn test_unify_same_base() {
        let s = unify(&Type::int(), &Type::int()).unwrap();
        assert!(s.is_empty());
    }

    #[test]
    fn test_unify_var() {
        let s = unify(&Type::Var(0), &Type::int()).unwrap();
        assert_eq!(s[&0], Type::int());
    }

    #[test]
    fn test_unify_occurs_check() {
        let result = unify(&Type::Var(0), &Type::arrow(Type::Var(0), Type::int()));
        assert!(result.is_err());
    }

    #[test]
    fn test_unify_arrows() {
        // (Int -> t0) ~ (t1 -> Bool)
        let t1 = Type::arrow(Type::int(), Type::Var(0));
        let t2 = Type::arrow(Type::Var(1), Type::bool());
        let s = unify(&t1, &t2).unwrap();
        assert_eq!(s[&0], Type::bool());
        assert_eq!(s[&1], Type::int());
    }

    #[test]
    fn test_unify_mismatch() {
        let result = unify(&Type::int(), &Type::bool());
        assert!(result.is_err());
    }

    #[test]
    fn test_occurs_check() {
        assert!(occurs(0, &Type::Var(0)));
        assert!(!occurs(0, &Type::int()));
        assert!(occurs(0, &Type::arrow(Type::Var(0), Type::int())));
    }

    #[test]
    fn test_generalize() {
        let env = TypeEnv::new();
        let ty = Type::Var(0);
        let gen = generalize(&env, &ty);
        // Should quantify t0
        matches!(gen, Type::Forall(0, _));
    }

    #[test]
    fn test_forall_display() {
        let t = Type::forall(0, Type::arrow(Type::Var(0), Type::Var(0)));
        let s = format!("{}", t);
        assert!(s.contains("∀"));
    }
}
