use cgmath::SquareMatrix as _;
use pollocad_cgal::*;
use std::borrow::Cow;
use std::sync::Arc;

pub use pollocad_cgal::Error;

#[derive(Clone)]
struct SolidItem {
    xform: Option<cgmath::Matrix4<f64>>,
    mesh: Arc<Mesh3>,
    anti: bool,
}

impl SolidItem {
    fn xformed_mesh(&self) -> Result<Cow<Mesh3>, Error> {
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

    pub fn cube(x: f64, y: f64, z: f64) -> Result<Solid, Error> {
        #[rustfmt::skip]
        const INDICES: &[u32] = &[
            0, 1, 3, 1, 2, 3, // front
            4, 0, 7, 0, 3, 7, // left
            5, 4, 6, 4, 7, 6, // back
            1, 5, 2, 5, 6, 2, // right
            3, 2, 7, 2, 6, 7, // top
            4, 5, 0, 5, 1, 0, // bottom
        ];

        #[rustfmt::skip]
        let vertices: &[f64] = &[
            0.0, 0.0, 0.0,
            x,   0.0, 0.0,
            x,   0.0, z,
            0.0, 0.0, z,
            0.0, y,   0.0,
            x,   y,   0.0,
            x,   y,   z,
            0.0, y,   z,
        ];

        Ok(Mesh3::from_data(vertices, INDICES)?.into())
    }

    pub fn cylinder(r: f64, h: f64, fn_: u32) -> Result<Solid, Error> {
        /*let mut indices: Vec<u32> = Vec::with_capacity(10);
        let mut vertices: Vec<f64> = Vec::with_capacity(10);

        let step = std::f64::consts::PI * 2.0 / f64::from(fn_);

        for i in 0..fn_ {
            let a = step * f64::from(i);
            let x = f64::cos(a) * r;
            let z = f64::sin(a) * r;

            vertices.push(x);
            vertices.push(0.0);
            vertices.push(z);

            vertices.push(x);
            vertices.push(h);
            vertices.push(z);

            // side
            indices.push((i * 2) % (fn_ * 2));
            indices.push((i * 2 + 2) % (fn_ * 2));
            indices.push((i * 2 + 1) % (fn_ * 2));
            indices.push((i * 2 + 2) % (fn_ * 2));
            indices.push((i * 2 + 1) % (fn_ * 2));
            indices.push((i * 2 + 3) % (fn_ * 2));

            if i < fn_ - 2 {
                // bottom
                indices.push(0);
                indices.push(i * 2 + 4);
                indices.push(i * 2 + 2);

                // top
                indices.push(1);
                indices.push(1 + (i * 2 + 2));
                indices.push(1 + (i * 2 + 4));
            }
        }

        println!("{:?} {:?} {:?}", vertices.len() / 3, indices.len(), indices);

        Ok(Mesh3::from_data(&vertices, &indices)?.into())*/

        unimplemented!();
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

    pub fn unionize(&self) -> Result<Solid, Error> {
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

    pub fn intersectionize<'a>(solids: impl Iterator<Item = &'a Solid>) -> Result<Solid, Error> {
        let items = solids
            .map(|s| Ok(s.unionize()?.0[0].mesh.clone()))
            .collect::<Result<Vec<_>, Error>>()?;

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

    pub fn to_mesh_data(&self) -> Result<Option<MeshData>, Error> {
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
