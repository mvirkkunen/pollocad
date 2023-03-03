use cgmath::SquareMatrix as _;
use pollocad_cascade::*;
use std::borrow::Cow;
use std::sync::Arc;

pub use pollocad_cascade::Error;

#[derive(Clone)]
struct SolidItem {
    xform: Option<cgmath::Matrix4<f64>>,
    shape: Arc<Shape>,
    anti: bool,
}

impl SolidItem {
    fn xformed_shape(&self) -> Result<Cow<Shape>, Error> {
        match self.xform {
            Some(xform) => Ok(Cow::Owned(self.shape.transform(xform.as_ref())?)),
            None => Ok(Cow::Borrowed(&*self.shape)),
        }
    }
}

pub struct Solid(Vec<SolidItem>);

impl Solid {
    pub fn new_cube(x: f64, y: f64, z: f64) -> Result<Solid, Error> {
        Ok(Shape::new_cube(x, y, z)?.into())
    }

    pub fn new_cylinder(r: f64, h: f64, fn_: u32) -> Result<Solid, Error> {
        unimplemented!();
    }

    pub fn anti(&self) -> Solid {
        Solid(
            self.0
                .iter()
                .map(|i| SolidItem {
                    xform: i.xform,
                    shape: i.shape.clone(),
                    anti: !i.anti,
                })
                .collect(),
        )
    }

    pub fn transform(&self, mat: &cgmath::Matrix4<f64>) -> Solid {
        Solid(
            self.0
                .iter()
                .map(|i| SolidItem {
                    xform: Some(mat * i.xform.unwrap_or_else(|| cgmath::Matrix4::identity())),
                    shape: i.shape.clone(),
                    anti: i.anti,
                })
                .collect(),
        )
    }

    pub fn unionize(&self) -> Result<Solid, Error> {
        let (anti, real): (Vec<_>, Vec<_>) = self.0.iter().partition(|i| i.anti);

        let Some(first) = real.first() else {
            return Ok(Solid(vec![]));
        };

        let mut acc = first.xformed_shape()?.into_owned();
        /*if let Some(x) = &first.xform {
            acc.transform(x.as_ref())?;
        }*/

        for item in real.iter().skip(1) {
            acc = acc.boolean_op(item.xformed_shape()?.as_ref(), BooleanOp::Union)?;
        }

        for item in anti {
            acc = acc.boolean_op(item.xformed_shape()?.as_ref(), BooleanOp::Difference)?;
        }

        Ok(Solid(vec![SolidItem {
            xform: None,
            shape: Arc::new(acc),
            anti: false,
        }]))
    }

    pub fn intersectionize<'a>(solids: impl Iterator<Item = &'a Solid>) -> Result<Solid, Error> {
        let items = solids
            .map(|s| Ok(s.unionize()?.0[0].shape.clone()))
            .collect::<Result<Vec<_>, Error>>()?;

        let Some(first) = items.first() else {
            return Ok(Solid(vec![]));
        };

        let mut acc = (**first).clone();

        for s in items.iter().skip(1) {
            acc = acc.boolean_op(s, BooleanOp::Intersection)?;
        }

        Ok(Solid(vec![SolidItem {
            xform: None,
            shape: Arc::new(acc),
            anti: false,
        }]))
    }

    pub fn combine<'a>(solids: impl Iterator<Item = &'a Solid>) -> Solid {
        Solid(solids.flat_map(|s| s.0.iter().cloned()).collect())
    }

    pub fn get_single_shape(&self) -> Option<Arc<Shape>> {
        self.0.get(0).map(|n| n.shape.clone())
    }
}

impl From<Shape> for Solid {
    fn from(shape: Shape) -> Solid {
        Solid(vec![SolidItem {
            xform: None,
            shape: Arc::new(shape),
            anti: false,
        }])
    }
}
