use cgmath::SquareMatrix as _;
use pollocad_cgal::*;
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Clone)]
struct SolidItem {
    xform: Option<cgmath::Matrix4<f64>>,
    nef: Arc<Nef3>,
    anti: bool,
}

impl SolidItem {
    fn xformed_nef(&self) -> Cow<Nef3> {
        self.xform
            .map(|x| {
                let mut clone = (*self.nef).clone();
                clone.transform(x.as_ref());
                Cow::Owned(clone)
            })
            .unwrap_or_else(|| Cow::Borrowed(&*self.nef))
    }
}

pub struct Solid(Vec<SolidItem>);

impl Solid {
    pub fn cube(x: f64, y: f64, z: f64) -> Result<Solid, String> {
        Ok(Solid(vec![SolidItem {
            xform: None,
            nef: Arc::new(Nef3::cube(x, y, z)?),
            anti: false,
        }]))
    }

    pub fn anti(&self) -> Solid {
        Solid(
            self.0
                .iter()
                .map(|i| SolidItem {
                    xform: i.xform,
                    nef: i.nef.clone(),
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
                    nef: i.nef.clone(),
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

        let mut acc = (*first.nef).clone();
        if let Some(x) = &first.xform {
            acc.transform(x.as_ref());
        }

        for item in real.iter().skip(1) {
            acc.union_with(&item.xformed_nef())?;
        }

        for item in anti {
            acc.difference_with(&item.xformed_nef())?;
        }

        Ok(Solid(vec![SolidItem {
            xform: None,
            nef: Arc::new(acc),
            anti: false,
        }]))
    }

    /*pub fn unionize<'a>(solids: impl Iterator<Item = &'a Solid>) -> Result<Solid, String> {
        let (anti, real): (Vec<_>, Vec<_>) = solids.flat_map(|s| s.0.iter()).partition(|i| i.anti);

        let Some(first) = real.first() else {
            return Ok(Solid(vec![]));
        };

        let mut acc = (*first.nef).clone();

        for item in real {
            let mut nef = Cow::Borrowed(&*item.nef);

            if let Some(xform) = &item.xform {
                let clone = (*nef).clone();
                clone.transform(xform.as_ref());
                nef = Cow::Owned(clone);
            }

            acc.union_with(&nef)?;
        }

        for item in anti {
            let mut nef = Cow::Borrowed(&*item.nef);

            if let Some(xform) = &item.xform {
                let clone = (*nef).clone();
                clone.transform(xform.as_ref());
                nef = Cow::Owned(clone);
            }

            acc.difference_with(&nef)?;
        }

        Ok(Solid(vec![SolidItem {
            xform: None,
            nef: Arc::new(acc),
            anti: false,
        }]))
    }*/

    pub fn combine<'a>(solids: impl Iterator<Item = &'a Solid>) -> Solid {
        Solid(solids.flat_map(|s| s.0.iter().cloned()).collect())
    }

    pub fn to_mesh_data(&self) -> Result<Option<MeshData>, String> {
        self.0.get(0).map(|n| n.nef.to_mesh_data()).transpose()
    }
}

pub struct Polygon;
