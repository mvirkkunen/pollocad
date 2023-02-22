use std::ffi::CStr;
use std::ptr;
use std::slice;

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(unused)]
mod bind {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[derive(Clone, Debug)]
pub struct Error(String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CGAL error: {}", self.0)
    }
}

impl std::error::Error for Error {}

fn protect<R>(f: impl FnOnce(*mut bind::Error) -> R) -> Result<R, Error> {
    let mut err: bind::Error = ptr::null_mut();
    let r = f(&mut err as *mut bind::Error);
    if !err.is_null() {
        unsafe {
            let err_str = String::from_utf8_lossy(CStr::from_ptr(err).to_bytes()).to_string();
            bind::error_free(err);
            return Err(Error(err_str));
        }
    }

    return Ok(r);
}

pub struct Mesh3(bind::Mesh3Obj);

#[repr(u32)]
pub enum BooleanOp {
    Union = bind::BooleanOp_BOOLEAN_OP_UNION as u32,
    Difference = bind::BooleanOp_BOOLEAN_OP_DIFFERENCE as u32,
    Intersection = bind::BooleanOp_BOOLEAN_OP_INTERSECTION as u32,
}

unsafe impl Send for Mesh3 {}

impl Mesh3 {
    pub fn from_data(vertices: &[f64], indices: &[u32]) -> Result<Mesh3, Error> {
        unsafe {
            protect(|err| {
                bind::mesh3_new_from_data(
                    vertices.as_ptr(),
                    vertices.len() as u32,
                    indices.as_ptr(),
                    indices.len() as u32,
                    err,
                )
            })
            .map(Mesh3)
        }
    }

    pub fn transform(&mut self, matrix: &[f64; 16]) -> Result<(), Error> {
        unsafe { protect(|err| bind::mesh3_transform(self.0, matrix.as_ptr(), err)) }
    }

    pub fn boolean_op_with(&mut self, other: &Mesh3, op: BooleanOp) -> Result<bool, Error> {
        let mut nef_fallback: u8 = 0;

        unsafe {
            protect(|err| {
                bind::mesh3_boolean_op(self.0, other.0, op as u32, &mut nef_fallback, err)
            })?;
        }

        Ok(nef_fallback != 0)
    }

    pub fn to_mesh_data(&self) -> Result<MeshData, Error> {
        unsafe { protect(|err| bind::mesh3_to_mesh_data(self.0, err)).map(MeshData) }
    }
}

impl Drop for Mesh3 {
    fn drop(&mut self) {
        unsafe {
            bind::mesh3_free(self.0);
        }
    }
}

impl Clone for Mesh3 {
    fn clone(&self) -> Self {
        unsafe { Mesh3(bind::mesh3_clone(self.0)) }
    }
}

pub struct Nef3(bind::Nef3Obj);

unsafe impl Send for Nef3 {}

impl Nef3 {
    pub fn transform(&mut self, matrix: &[f64; 16]) -> Result<(), Error> {
        unsafe { protect(|err| bind::nef3_transform(self.0, matrix.as_ptr(), err)) }
    }

    pub fn union_with(&mut self, other: &Nef3) -> Result<(), Error> {
        unsafe { protect(|err| bind::nef3_union(self.0, other.0, err)) }
    }

    pub fn difference_with(&mut self, other: &Nef3) -> Result<(), Error> {
        unsafe { protect(|err| bind::nef3_difference(self.0, other.0, err)) }
    }

    pub fn intersection_with(&mut self, other: &Nef3) -> Result<(), Error> {
        unsafe { protect(|err| bind::nef3_intersection(self.0, other.0, err)) }
    }

    pub fn to_mesh_data(&self) -> Result<MeshData, Error> {
        unsafe { protect(|err| bind::nef3_to_mesh_data(self.0, err)).map(MeshData) }
    }
}

impl Drop for Nef3 {
    fn drop(&mut self) {
        unsafe {
            bind::nef3_free(self.0);
        }
    }
}

impl Clone for Nef3 {
    fn clone(&self) -> Self {
        unsafe { Nef3(bind::nef3_clone(self.0)) }
    }
}

pub struct MeshData(*mut bind::MeshData);

unsafe impl Send for MeshData {}
unsafe impl Sync for MeshData {}

impl MeshData {
    pub fn stride(&self) -> usize {
        unsafe { (*self.0).stride }
    }

    pub fn vertex_data(&self) -> &[u8] {
        unsafe { slice::from_raw_parts((*self.0).vertex_data, (*self.0).vertex_len) }
    }

    pub fn index_data(&self) -> &[u8] {
        unsafe { slice::from_raw_parts((*self.0).index_data, (*self.0).index_len) }
    }
}

impl Drop for MeshData {
    fn drop(&mut self) {
        unsafe {
            bind::mesh_data_free(self.0);
        }
    }
}
