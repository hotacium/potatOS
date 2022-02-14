
use core::mem::MaybeUninit;
use core::marker::PhantomData;

#[derive(Debug)]
pub enum FixedVecError {
    Full,
    Empty,
    IndexOutOfRange,
}
type Result<T> = core::result::Result<T, FixedVecError>;


pub struct FixedVec<'a, T, const CAPACITY: usize> {
    data: [MaybeUninit<T>; CAPACITY],
    len: usize,
    _phantom: &'a PhantomData<()>,
}

impl<'vec_lifetime, T, const CAPACITY: usize> FixedVec<'vec_lifetime, T, CAPACITY> {
    
    pub const fn new() -> Self {
        Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
            _phantom: &PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_slice(&self) -> &[T] {
        let p = self.data.as_ptr() as *const T;
        unsafe { core::slice::from_raw_parts(p, self.len) }
    }

    pub fn push(&mut self, val: T) {
        unsafe {
            self.try_push(val).unwrap();
        }
    }

    pub unsafe fn try_push(&mut self, val: T) -> Result<()> {
        if self.len < CAPACITY {
            let len = self.len;
            let ptr = (self.data.as_mut_ptr() as *mut T).add(len);
            ptr.write(val);
            self.len += 1;
            Ok(())
        } else {
            Err(FixedVecError::Full)
        }
    }

    pub fn pop(&mut self) -> T {
        unsafe {
            self.try_pop().unwrap()
        }
    }

    pub unsafe fn try_pop(&mut self) -> Result<T> {
        if self.len > 0 {
            self.len -= 1;
            let ptr = (self.data.as_mut_ptr() as *mut T).add(self.len);
            Ok(ptr.read())
        } else {
            Err(FixedVecError::Empty)
        }
    }

    pub fn get(&self, idx: usize) -> &'vec_lifetime T {
        unsafe {
            self.try_get(idx).unwrap()
        }
    }

    pub unsafe fn try_get(&self, idx: usize) -> Result<&'vec_lifetime T> {
        if idx < self.len {
            let ptr = (self.data.as_ptr() as *const T).add(idx);
            Ok(&*ptr)
        } else {
            Err(FixedVecError::IndexOutOfRange)
        }
    }

    pub fn get_mut(&mut self, idx: usize) -> &'vec_lifetime mut T {
        unsafe {
            self.try_get_mut(idx).unwrap()
        }
    }

    pub unsafe fn try_get_mut(&mut self, idx: usize) -> Result<&'vec_lifetime mut T> {
        if idx < self.len {
            let ptr = (self.data.as_mut_ptr() as *mut T).add(idx);
            Ok(&mut *ptr)
        } else {
            Err(FixedVecError::IndexOutOfRange)
        }
    }

    pub fn iter(&'vec_lifetime self) -> FixedVecIter<'vec_lifetime, T, CAPACITY> {
        FixedVecIter {
            fixed_vec: self,
            idx: 0,
        }
    }

    pub fn iter_mut(&'vec_lifetime mut self) -> FixedVecIterMut<'vec_lifetime, T, CAPACITY> {
        FixedVecIterMut {
            fixed_vec: self,
            idx: 0,
        }
    }

} 

use core::iter::Iterator;

pub struct FixedVecIter<'vec_lifetime, T, const CAPACITY: usize> {
    fixed_vec: &'vec_lifetime FixedVec<'vec_lifetime, T, CAPACITY>,
    idx: usize,
}

impl<'a, T, const CAPACITY: usize> Iterator for FixedVecIter<'a, T, CAPACITY> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == self.fixed_vec.len() {
            None
        } else {
            let val = unsafe { self.fixed_vec.try_get(self.idx).ok() };
            self.idx += 1;
            val
        }
    }
}

pub struct FixedVecIterMut<'vec_lifetime, T, const CAPACITY: usize> {
    fixed_vec: &'vec_lifetime mut FixedVec<'vec_lifetime, T, CAPACITY>,
    idx: usize,
}


impl<'a, T, const CAPACITY: usize> Iterator for FixedVecIterMut<'a, T, CAPACITY> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == self.fixed_vec.len() {
            None
        } else {
            let val = unsafe { self.fixed_vec.try_get_mut(self.idx).ok() };
            self.idx += 1;
            val
        }
    }
}
