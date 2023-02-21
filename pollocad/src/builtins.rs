use std::collections::BTreeMap;
use std::ops::*;
use std::sync::Arc;

use crate::exec::*;
use crate::geometry::*;

struct Cube;

const EPSILON: f64 = 0.001;

impl BuiltinFunc for Cube {
    fn is_heavy(&self) -> bool {
        true
    }

    fn call(&self, args: &FuncArgs) -> Result<Value, String> {
        let mut sx = 1.0;
        let mut sy = 1.0;
        let mut sz = 1.0;

        if let Some(Value::Num(x)) = args.pos.get(0) {
            sx = x.max(EPSILON);
        }

        if let Some(Value::Num(y)) = args.pos.get(1) {
            sy = y.max(EPSILON);
        }

        if let Some(Value::Num(z)) = args.pos.get(2) {
            sz = z.max(EPSILON);
        }

        Ok(Value::Solid(Arc::new(Solid::cube(sx, sy, sz)?)))
    }
}

fn map_solid(
    args: &FuncArgs,
    f: impl FnOnce(Solid) -> Result<Solid, String>,
) -> Result<Value, String> {
    args.children
        .iter()
        .map(|c| match c {
            Value::Solid(s) => Ok(s.as_ref()),
            _ => Err("Combinators may only have solid values as children".to_string()),
        })
        .collect::<Result<Vec<_>, String>>()
        .map(|s| Solid::combine(s.iter().copied()))
        .and_then(f)
        .map(|s| Value::Solid(Arc::new(s)))
}

struct Union;

impl BuiltinFunc for Union {
    fn is_heavy(&self) -> bool {
        true
    }

    fn call(&self, args: &FuncArgs) -> Result<Value, String> {
        map_solid(args, |s| s.unionize())
    }
}

struct Anti;

impl BuiltinFunc for Anti {
    fn call(&self, args: &FuncArgs) -> Result<Value, String> {
        map_solid(args, |s| Ok(s.anti()))
    }
}

struct Translate;

impl BuiltinFunc for Translate {
    fn call(&self, args: &FuncArgs) -> Result<Value, String> {
        let mut coord: [f64; 3] = [0.0, 0.0, 0.0];

        for (i, a) in args.pos.iter().enumerate().take(3) {
            if let Value::Num(n) = a {
                coord[i] = *n;
            }
        }

        if let Some(Value::Num(x)) = args.named.get("x") {
            coord[0] = *x;
        }

        if let Some(Value::Num(y)) = args.named.get("y") {
            coord[1] = *y;
        }

        if let Some(Value::Num(z)) = args.named.get("z") {
            coord[2] = *z;
        }

        let mat = cgmath::Matrix4::from_translation(coord.into());
        map_solid(args, |s| Ok(s.transform(&mat)))
    }
}

struct NumOp(fn(a: f64, b: f64) -> f64);

impl BuiltinFunc for NumOp {
    fn call(&self, args: &FuncArgs) -> Result<Value, String> {
        assert!(args.pos.len() == 2);

        let Value::Num(a) = args.pos[0] else { return Err("not a number".to_string()) };
        let Value::Num(b) = args.pos[1] else { return Err("not a number".to_string()) };

        Ok(Value::Num(self.0(a, b)))
    }
}

trait AddFunc {
    fn add_func(&mut self, name: &str, func: impl BuiltinFunc + 'static);
}

impl AddFunc for BTreeMap<String, Value> {
    fn add_func(&mut self, name: &str, func: impl BuiltinFunc + 'static) {
        self.insert(name.to_string(), Value::BuiltinFunc(Arc::new(func)));
    }
}

pub fn get_builtins() -> BTreeMap<String, Value> {
    let mut builtins = BTreeMap::new();
    builtins.add_func("cube", Cube);
    builtins.add_func("union", Union);
    builtins.add_func("anti", Anti);
    builtins.add_func("translate", Translate);
    builtins.add_func("+", NumOp(f64::add));
    builtins.add_func("-", NumOp(f64::sub));
    builtins.add_func("*", NumOp(f64::mul));
    builtins.add_func("/", NumOp(f64::div));
    builtins
}
