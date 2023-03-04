use cpp::{cpp, cpp_class};

use crate::{CppResult, Result};

cpp! {{
    #include <Bnd_Box.hxx>
    #include <BRepAlgoAPI_Common.hxx>
    #include <BRepAlgoAPI_Cut.hxx>
    #include <BRepAlgoAPI_Fuse.hxx>
    #include <BRepBndLib.hxx>
    #include <BRepBuilderAPI_Copy.hxx>
    #include <BRepBuilderAPI_Transform.hxx>
    #include <BRepPrimAPI_MakeBox.hxx>
    #include <BRepPrimAPI_MakeCylinder.hxx>
    #include <TopoDS_Shape.hxx>

    #include "protect.hpp"
}}

cpp_class!(pub unsafe struct Shape as "TopoDS_Shape");

cpp_class!(unsafe struct ShapeResult as "CppResult<TopoDS_Shape>");

impl CppResult for ShapeResult {
    type Value = Shape;

    unsafe fn get(self, err: *mut *mut i8) -> Self::Value {
        cpp!([self as "CppResult<TopoDS_Shape>", err as "char **"] -> Shape as "TopoDS_Shape" {
            return self.get(err);
        })
    }
}

pub use crate::constants::BooleanOp;

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct BoundingBox {
    pub xmin: f64,
    pub ymin: f64,
    pub zmin: f64,
    pub xmax: f64,
    pub ymax: f64,
    pub zmax: f64,
}

impl Shape {
    pub fn new_cube(x: f64, y: f64, z: f64) -> Result<Shape> {
        cpp!(unsafe [x as "double", y as "double", z as "double"] -> ShapeResult as "CppResult<TopoDS_Shape>" {
            return protect<TopoDS_Shape>([=]{ return BRepPrimAPI_MakeBox{x, y, z}; });
        }).result()
    }

    pub fn new_cylinder(r: f64, h: f64) -> Result<Shape> {
        cpp!(unsafe [r as "double", h as "double"] -> ShapeResult as "CppResult<TopoDS_Shape>" {
            return protect<TopoDS_Shape>([=] { return BRepPrimAPI_MakeCylinder{r, h}; });
        }).result()
    }

    pub fn transform(&self, matrix: &[f64; 16]) -> Result<Shape> {
        cpp!(unsafe [self as "TopoDS_Shape *", matrix as "double *"] -> ShapeResult as "CppResult<TopoDS_Shape>" {
            return protect<TopoDS_Shape>([=]() {
                gp_Trsf xform{};
                xform.SetValues(
                    matrix[0], matrix[4], matrix[8], matrix[12],
                    matrix[1], matrix[5], matrix[9], matrix[13],
                    matrix[2], matrix[6], matrix[10], matrix[14]
                );
                return TopoDS_Shape{BRepBuilderAPI_Transform{*self, xform}};
            });
        }).result()
    }

    pub fn boolean_op(&self, other: &Shape, op: BooleanOp) -> Result<Shape> {
        cpp!(unsafe [self as "TopoDS_Shape *", other as "TopoDS_Shape *", op as "BooleanOp"] -> ShapeResult as "CppResult<TopoDS_Shape>" {
            return protect<TopoDS_Shape>([=] {
                switch (op) {
                    default:
                    case BooleanOp::Union:
                        return TopoDS_Shape{BRepAlgoAPI_Fuse{*self, *other}};
                    case BooleanOp::Difference:
                        return TopoDS_Shape{BRepAlgoAPI_Cut{*self, *other}};
                    case BooleanOp::Intersection:
                        return TopoDS_Shape{BRepAlgoAPI_Common{*self, *other}};
                }
            });
        }).result()
    }

    pub fn bounds(&self) -> BoundingBox {
        let mut r = BoundingBox::default();

        unsafe {
            let r = &mut r as *mut _ as *mut f64;
            cpp!([self as "const TopoDS_Shape *", r as "double *"] {
                Bnd_Box b;
                BRepBndLib::Add(*self, b);
                b.Get(r[0], r[1], r[2], r[3], r[4], r[5]);
            })
        }

        r
    }
}
