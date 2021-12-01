
use core::mem::MaybeUninit;

#[derive(Debug)]
pub enum FixedVecError {
    Full,
    Empty,
    IndexOutOfRange,
}
type Result<T> = core::result::Result<T, FixedVecError>;


pub struct FixedVec<T, const CAPACITY: usize> {
    data: MaybeUninit<[T; CAPACITY]>,
    len: usize,
}

impl<T, const CAPACITY: usize> FixedVec<T, CAPACITY> {
    
    pub const fn new() -> Self {
        Self {
            data: MaybeUninit::uninit(),
            len: 0,
        }
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

    pub fn get(&self, idx: usize) -> &T {
        unsafe {
            self.try_get(idx).unwrap()
        }
    }

    pub unsafe fn try_get(&self, idx: usize) -> Result<&T> {
        if idx < self.len {
            let ptr = (self.data.as_ptr() as *const T).add(idx);
            Ok(&*ptr)
        } else {
            Err(FixedVecError::IndexOutOfRange)
        }
    }

    pub fn get_mut(&mut self, idx: usize) -> &mut T {
        unsafe {
            self.try_get_mut(idx).unwrap()
        }
    }

    pub unsafe fn try_get_mut(&mut self, idx: usize) -> Result<&mut T> {
        if idx < self.len {
            let ptr = (self.data.as_mut_ptr() as *mut T).add(idx);
            Ok(&mut *ptr)
        } else {
            Err(FixedVecError::IndexOutOfRange)
        }
    }

} 

// impl<T, const CAPACITY: usize> FixedVec<T, CAPACITY> {
//     pub const fn new() -> Self {
//         Self {
//             buf: unsafe { MaybeUninit::uninit().assume_init() },
//             len: 0,
//         }
//     }
//     pub fn capacity(&self) -> usize {
//         CAPACITY
//     }
//     pub fn len(&self) -> usize {
//         self.len
//     }
// 
//     pub fn get(&self, idx: usize) -> Option<&T> {
//         if idx < self.len {
//             let item = self.buf[idx];
//             Some(unsafe { &item.assume_init() })
//         } else {
//             None
//         }
//     }
// 
//     pub fn get_mut(&self, idx: usize) -> Option<&mut T> {
//         if idx < self.len {
//             let item = &self.buf[idx];
//             Some(unsafe { item.assume_init_mut() })
//         } else {
//             None
//         }
//     }
// 
//     pub fn push(&mut self, val: T) -> Option<usize> {
//         if self.len < CAPACITY {
//             let idx = self.len;
//             self.buf[idx].write(val);
//             self.len += 1;
//             Some(idx)
//         } else {
//             None
//         }
//     }
//     pub fn pop(&mut self) -> Option<T> {
//         if self.len > 0 {
//             let idx = self.len()-1;
//             self.len -= 1;
//             Some(unsafe { self.buf[idx].assume_init() })
//         } else {
//             None
//         }
//     }
// }