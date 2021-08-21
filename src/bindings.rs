use std::os::raw::c_int;
use std::os::raw::c_double;
use std::os::raw::c_ulong;
use std::os::raw::c_void;

pub type size_t = c_ulong;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rtree {
    _unused: [u8; 0],
}

pub type IterCallback = unsafe extern "C" fn(rect: *const c_double, item: *const c_void, udata: *mut c_void) -> bool;

extern "C" {
    pub fn rtree_new(
        elsize: size_t,
        dims: c_int,
    ) -> *mut rtree;
    pub fn rtree_free(rtree: *mut rtree);
    pub fn rtree_count(rtree: *mut rtree) -> size_t;
    pub fn rtree_insert(rtree: *mut rtree, rect: *mut c_double, item: *const c_void) -> bool;
    pub fn rtree_delete(rtree: *mut rtree, rect: *mut c_double, item: *const c_void) -> bool;
    pub fn rtree_search(rtree: *mut rtree, rect: *mut c_double, iter: IterCallback, udata: *mut c_void) -> bool;
}
