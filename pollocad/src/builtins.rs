use std::collections::HashMap;
use std::error::Error;
use std::ops::*;
use std::sync::Arc;

use crate::geometry::Solid;
use crate::runtime::{BuiltinFunc, CallCtx, Value};
const EPSILON: f64 = 0.001;

trait MapHelpers {
    fn add_func(&mut self, name: &str, func: impl BuiltinFunc + 'static);
}

impl MapHelpers for HashMap<String, Value> {
    fn add_func(&mut self, name: &str, func: impl BuiltinFunc + 'static) {
        self.insert(name.to_string(), Value::BuiltinFunc(Arc::new(func)));
    }
}

#[derive(Clone, Debug)]
struct BuiltinError(String);

impl BuiltinError {
    fn new(msg: impl ToString) -> Self {
        BuiltinError(msg.to_string())
    }
}

impl std::fmt::Display for BuiltinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CGAL error: {}", self.0)
    }
}

impl std::error::Error for BuiltinError {}

fn err<V>(msg: impl ToString) -> Result<V, Box<dyn Error>> {
    Err(Box::new(BuiltinError(msg.to_string())))
}

struct Cube;
impl BuiltinFunc for Cube {
    fn is_heavy(&self) -> bool {
        true
    }

    fn call(&self, c: &mut CallCtx) -> Result<Value, Box<dyn Error>> {
        let sx = c
            .named_num("x")?
            .or(c.pos_num(0, "x")?)
            .unwrap_or(1.0)
            .max(EPSILON);
        let sy = c
            .named_num("y")?
            .or(c.pos_num(1, "x")?)
            .unwrap_or(1.0)
            .max(EPSILON);
        let sz = c
            .named_num("z")?
            .or(c.pos_num(2, "x")?)
            .unwrap_or(1.0)
            .max(EPSILON);

        Ok(Value::Solid(Arc::new(Solid::new_cube(
            sx.max(EPSILON),
            sy.max(EPSILON),
            sz.max(EPSILON),
        )?)))
    }
}

struct Cylinder;
impl BuiltinFunc for Cylinder {
    fn is_heavy(&self) -> bool {
        true
    }

    fn call(&self, c: &mut CallCtx) -> Result<Value, Box<dyn Error>> {
        let d = c.named_num("d")?;
        let r = c.named_num("r")?;
        if d.is_some() && r.is_some() {
            return err(
                "Cannot specify both diameter and radius for cylinder",
            );
        }
        let r = r.or(d.map(|d| d * 0.5)).unwrap_or(1.0); //.max(EPSILON);

        let h = c.named_num("h")?.unwrap_or(1.0); //.max(EPSILON);

        Ok(Value::Solid(Arc::new(Solid::new_cylinder(r, h)?)))
    }
}

fn map_solid(
    c: &CallCtx,
    f: impl FnOnce(Solid) -> Result<Solid, Box<dyn Error>>,
) -> Result<Value, Box<dyn Error>> {
    c.children
        .iter()
        .map(|c| match c {
            Value::Solid(s) => Ok(s.as_ref()),
            _ => err("Combinators may only have solid values as children"),
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()
        .map(|s| Solid::combine(s.iter().copied()))
        .and_then(f)
        .map(|s| Value::Solid(Arc::new(s)))
}

struct Union;
impl BuiltinFunc for Union {
    fn is_heavy(&self) -> bool {
        true
    }

    fn call(&self, c: &mut CallCtx) -> Result<Value, Box<dyn Error>> {
        map_solid(c, |s| Ok(s.unionize()?))
    }
}

struct Intersection;
impl BuiltinFunc for Intersection {
    fn is_heavy(&self) -> bool {
        true
    }

    fn call(&self, c: &mut CallCtx) -> Result<Value, Box<dyn Error>> {
        let children = c
            .children
            .iter()
            .map(|c| match c {
                Value::Solid(s) => Ok(s.as_ref()),
                _ => err("Combinators may only have solid values as children"),
            })
            .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

        Ok(Value::Solid(Arc::new(Solid::intersectionize(
            children.into_iter(),
        )?)))
    }
}

struct Anti;
impl BuiltinFunc for Anti {
    fn call(&self, c: &mut CallCtx) -> Result<Value, Box<dyn Error>> {
        map_solid(c, |s| Ok(s.anti()))
    }
}

struct Translate;
impl BuiltinFunc for Translate {
    fn call(&self, c: &mut CallCtx) -> Result<Value, Box<dyn Error>> {
        let mut coord: [f64; 3] = [0.0, 0.0, 0.0];

        for (i, a) in c.pos.iter().enumerate().take(3) {
            if let Value::Num(n) = a {
                coord[i] = *n;
            }
        }

        if let Some(Value::Num(x)) = c.named.get("x") {
            coord[0] = *x;
        }

        if let Some(Value::Num(y)) = c.named.get("y") {
            coord[1] = *y;
        }

        if let Some(Value::Num(z)) = c.named.get("z") {
            coord[2] = *z;
        }

        let mat = cgmath::Matrix4::from_translation(coord.into());
        map_solid(c, |s| Ok(s.transform(&mat)))
    }
}

struct NumOp(fn(a: f64, b: f64) -> f64);
impl BuiltinFunc for NumOp {
    fn call(&self, c: &mut CallCtx) -> Result<Value, Box<dyn Error>> {
        assert!(c.pos.len() == 2);

        let Value::Num(a) = c.pos[0] else { return err("not a number") };
        let Value::Num(b) = c.pos[1] else { return err("not a number") };

        Ok(Value::Num(self.0(a, b)))
    }
}

pub fn get_builtins() -> HashMap<String, Value> {
    let mut builtins = HashMap::new();
    builtins.add_func("cube", Cube);
    builtins.add_func("cylinder", Cylinder);
    builtins.add_func("union", Union);
    builtins.add_func("intersection", Intersection);
    builtins.add_func("anti", Anti);
    builtins.add_func("translate", Translate);
    builtins.add_func("+", NumOp(f64::add));
    builtins.add_func("-", NumOp(f64::sub));
    builtins.add_func("*", NumOp(f64::mul));
    builtins.add_func("/", NumOp(f64::div));
    builtins
}
