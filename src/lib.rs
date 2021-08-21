#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!("./bindings.rs");

use std::convert::TryInto;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::size_of;
use std::mem::transmute;

unsafe extern "C" fn iter_trampoline<T, I, const N: u32>(
    rect: *const c_double,
    item: *const c_void,
    user_data: *mut c_void,
) -> bool
where
    I: FnMut(&[f64], &T) -> bool,
{
    let item = &*(item as *const T);
    eprintln!("trampoline called");
    let rect_slice = std::slice::from_raw_parts(rect, N.try_into().unwrap());
    let iter_fn = transmute::<*mut c_void, *mut I>(user_data);
    (*iter_fn)(rect_slice, item)
}

#[repr(C)]
pub struct RTreeC<T, const N: u32> {
    rtree: *mut rtree,
    _phantom: PhantomData<T>,
}

unsafe impl<T, const N: u32> Send for RTreeC<T, N> {}
unsafe impl<T, const N: u32> Sync for RTreeC<T, N> {}

impl<T, const N: u32> RTreeC<T, N> {
    pub fn new() -> Self {
        let p = unsafe { rtree_new(size_of::<T>().try_into().unwrap(), N.try_into().unwrap()) };
        Self {
            rtree: p,
            _phantom: PhantomData,
        }
    }

    pub fn count(&self) -> u64 {
        #[allow(clippy::useless_conversion)]
        unsafe { rtree_count(self.rtree) }.into()
    }

    /// Resturns true if the item was deleted or false if item was not found.
    pub fn delete(&mut self, mut rect: Vec<f64>, item: &T) -> bool {
        let rect_ptr = rect.as_mut_ptr();
        let item_ptr: *const c_void = item as *const _ as *const c_void;
        let result = unsafe { rtree_delete(self.rtree, rect_ptr, item_ptr) };
        std::mem::forget(rect);
        std::mem::forget(item);
        result
    }

    // rtree_insert inserts an item into the rtree. This operation performs a copy
    // of the data that is pointed to in the second and third arguments. The R-tree
    // expects a rectangle, which is an array of doubles, that has the first N
    // values as the minimum corner of the rect, and the next N values as the
    // maximum corner of the rect, where N is the number of dimensions provided
    // to rtree_new().
    // Returns false if the system is out of memory.
    pub fn insert(&mut self, mut rect: Vec<f64>, item: &T) -> bool {
        let rect_ptr = rect.as_mut_ptr();
        let item_ptr: *const c_void = item as *const _ as *const c_void;
        let result = unsafe { rtree_insert(self.rtree, rect_ptr, item_ptr) };
        std::mem::forget(rect);
        std::mem::forget(item);
        result
    }

    pub fn search<I>(&self, mut rect: Vec<f64>, iter_fn: I) -> bool
    where
        I: FnMut(&[f64], &T) -> bool,
    {
        let rect_ptr = rect.as_mut_ptr();
        std::mem::forget(rect);
        let mut iter_fn = Box::new(iter_fn);
        let user_data = unsafe { transmute::<*mut I, *mut c_void>(&mut *iter_fn) };
        unsafe { rtree_search(self.rtree, rect_ptr, iter_trampoline::<T, I, N>, user_data) }
    }
}

impl<T, const N: u32> Drop for RTreeC<T, N> {
    fn drop(&mut self) {
        unsafe { rtree_free(self.rtree) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default, Clone)]
    struct City {
        name: String,
        lat: f64,
        lon: f64,
    }

    #[test]
    fn rtreec_new() {
        let mut rtree = RTreeC::<City, 2>::new();
        assert_eq!(rtree.count(), 0);

        let phx = City {
            name: "Phoenix".to_string(),
            lat: 33.448,
            lon: -112.073,
        };
        let enn = City {
            name: "Ennis".to_string(),
            lat: 52.843,
            lon: -8.986,
        };
        let pra = City {
            name: "Prague".to_string(),
            lat: 50.088,
            lon: -14.420,
        };
        let tai = City {
            name: "Taipei".to_string(),
            lat: 25.033,
            lon: 121.565,
        };
        let her = City {
            name: "Hermosillo".to_string(),
            lat: 29.102,
            lon: -110.977,
        };
        let him = City {
            name: "Himeji".to_string(),
            lat: 34.816,
            lon: 134.700,
        };

        rtree.insert(vec![phx.lon, phx.lat, phx.lon, phx.lat], &phx.clone());
        rtree.insert(vec![enn.lon, enn.lat, enn.lon, enn.lat], &enn);
        rtree.insert(vec![pra.lon, pra.lat, pra.lon, pra.lat], &pra);
        rtree.insert(vec![tai.lon, tai.lat, tai.lon, tai.lat], &tai);
        rtree.insert(vec![her.lon, her.lat, her.lon, her.lat], &her);
        rtree.insert(vec![him.lon, him.lat, him.lon, him.lat], &him);

        assert_eq!(rtree.count(), 6);

        let mut northwestern_cities = vec![];
        rtree.search(vec![-180.0, 0.0, 0.0, 90.0], |_rect, item| {
            eprintln!("iter! {:?} {:?}", _rect, item);
            northwestern_cities.push(item.name.to_string());
            true
        });
        assert_eq!(
            northwestern_cities,
            vec!["Phoenix", "Ennis", "Prague", "Hermosillo"]
        );

        let mut northeastern_cities = vec![];
        rtree.search(vec![0.0, 0.0, 180.0, 90.0], |_rect, item| {
            northeastern_cities.push(item.name.to_string());
            true
        });
        assert_eq!(northeastern_cities, vec!["Taipei", "Himeji"]);

        assert!(rtree.delete(vec![phx.lon, phx.lat, phx.lon, phx.lat], &phx));

        let mut northwestern_cities = vec![];
        rtree.search(vec![-180.0, 0.0, 0.0, 90.0], |_rect, item| {
            northwestern_cities.push(item.name.to_string());
            true
        });
        assert_eq!(northwestern_cities, vec!["Ennis", "Prague", "Hermosillo"]);
    }
}
