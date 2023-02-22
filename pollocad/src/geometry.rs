use cgmath::SquareMatrix as _;
use pollocad_cgal::*;
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Clone)]
struct SolidItem {
    xform: Option<cgmath::Matrix4<f64>>,
    mesh: Arc<Mesh3>,
    anti: bool,
}

impl SolidItem {
    fn xformed_mesh(&self) -> Result<Cow<Mesh3>, String> {
        match self.xform {
            Some(xform) => {
                let mut clone = (*self.mesh).clone();
                clone.transform(xform.as_ref())?;
                Ok(Cow::Owned(clone))
            }
            None => Ok(Cow::Borrowed(&*self.mesh)),
        }
    }
}

pub struct Solid(Vec<SolidItem>);

impl Solid {
    fn primitive(mesh: Mesh3) -> Solid {
        Solid(vec![SolidItem {
            xform: None,
            mesh: Arc::new(mesh),
            anti: false,
        }])
    }

    pub fn cube(x: f64, y: f64, z: f64) -> Result<Solid, String> {
        Ok(Mesh3::cube(x, y, z)?.into())
    }

    pub fn cylinder(r: f64, h: f64, fn_: u32) -> Result<Solid, String> {
        Ok(Mesh3::cylinder(r, h, fn_)?.into())
    }

    pub fn anti(&self) -> Solid {
        Solid(
            self.0
                .iter()
                .map(|i| SolidItem {
                    xform: i.xform,
                    mesh: i.mesh.clone(),
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
                    mesh: i.mesh.clone(),
                    anti: i.anti,
                })
                .collect(),
        )
    }

    pub fn unionize(&self) -> Result<Solid, String> {
        let (anti, real): (Vec<_>, Vec<_>) = self.0.iter().partition(|i| i.anti);

        let Some(first) = real.first() else {
            return Ok(Solid(vec![]));
        };

        let mut acc = (*first.mesh).clone();
        if let Some(x) = &first.xform {
            acc.transform(x.as_ref())?;
        }

        for item in real.iter().skip(1) {
            acc.boolean_op_with(item.xformed_mesh()?.as_ref(), BooleanOp::Union)?;
        }

        for item in anti {
            acc.boolean_op_with(item.xformed_mesh()?.as_ref(), BooleanOp::Difference)?;
        }

        Ok(Solid(vec![SolidItem {
            xform: None,
            mesh: Arc::new(acc),
            anti: false,
        }]))
    }

    pub fn intersectionize<'a>(solids: impl Iterator<Item = &'a Solid>) -> Result<Solid, String> {
        let items = solids
            .map(|s| Ok(s.unionize()?.0[0].mesh.clone()))
            .collect::<Result<Vec<_>, String>>()?;

        let Some(first) = items.first() else {
            return Ok(Solid(vec![]));
        };

        let mut acc = (**first).clone();

        for s in items.iter().skip(1) {
            acc.boolean_op_with(s, BooleanOp::Intersection)?;
        }

        Ok(Solid(vec![SolidItem {
            xform: None,
            mesh: Arc::new(acc),
            anti: false,
        }]))
    }

    pub fn combine<'a>(solids: impl Iterator<Item = &'a Solid>) -> Solid {
        Solid(solids.flat_map(|s| s.0.iter().cloned()).collect())
    }

    pub fn to_mesh_data(&self) -> Result<Option<MeshData>, String> {
        self.0.get(0).map(|n| n.mesh.to_mesh_data()).transpose()
    }
}

impl From<Mesh3> for Solid {
    fn from(mesh: Mesh3) -> Solid {
        Solid(vec![SolidItem {
            xform: None,
            mesh: Arc::new(mesh),
            anti: false,
        }])
    }
}

pub struct Polygon;
