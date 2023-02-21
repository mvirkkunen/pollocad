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

pub struct Nef3(bind::Nef3Obj);

unsafe impl Send for Nef3 {}

fn protect<R>(f: impl FnOnce(*mut bind::Error) -> R) -> Result<R, String> {
    let mut err: bind::Error = ptr::null_mut();
    let r = f(&mut err as *mut bind::Error);
    if !err.is_null() {
        unsafe {
            let err_str = String::from_utf8_lossy(CStr::from_ptr(err).to_bytes()).to_string();
            bind::error_free(err);
            return Err(err_str);
        }
    }

    return Ok(r);
}

impl Nef3 {
    pub fn cube(x: f64, y: f64, z: f64) -> Result<Nef3, String> {
        unsafe { protect(|err| bind::nef3_new_cube(x, y, z, err)).map(Nef3) }
    }

    pub fn transform(&mut self, matrix: &[f64; 16]) -> Result<(), String> {
        unsafe { protect(|err| bind::nef3_transform(self.0, matrix.as_ptr(), err)) }
    }

    pub fn union_with(&mut self, other: &Nef3) -> Result<(), String> {
        unsafe { protect(|err| bind::nef3_union(self.0, other.0, err)) }
    }

    pub fn difference_with(&mut self, other: &Nef3) -> Result<(), String> {
        unsafe { protect(|err| bind::nef3_difference(self.0, other.0, err)) }
    }

    pub fn intersection_with(&mut self, other: &Nef3) -> Result<(), String> {
        unsafe { protect(|err| bind::nef3_intersection(self.0, other.0, err)) }
    }

    pub fn to_mesh_data(&self) -> Result<MeshData, String> {
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
