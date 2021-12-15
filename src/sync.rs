
// ------------------------------------------------------
// SpinMutex
// ------------------------------------------------------
// TODO: move spinmutex to spinmutex_like.rs
use core::sync::atomic::{AtomicBool, Ordering};
use core::cell::UnsafeCell;
pub struct SpinMutex<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>
}

impl<T> SpinMutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn try_lock(&self) -> Result<SpinMutexGuard<T>, SpinMutexErr> {
        if !self.lock.swap(true, Ordering::Acquire) {
            Ok(SpinMutexGuard { mutex: self })
        } else {
            Err(SpinMutexErr("lock error"))
        }

    }

    pub fn lock(&self) -> SpinMutexGuard<T> {
        loop {
            if let Ok(guard) = self.try_lock() {
                return guard;
            }
        }
    }

}

// Send + Sync are required for static 
unsafe impl<T> Send for SpinMutex<T> {}
unsafe impl<T> Sync for SpinMutex<T> {}

pub struct SpinMutexGuard<'a, T> {
    mutex: &'a SpinMutex<T>,
}

impl<T> SpinMutexGuard<'_, T> {
    fn unlock(&self) {
        self.mutex.lock.swap(false, Ordering::Release);
    }
}

// when drop, unlock
use core::ops::Drop;
impl<T> Drop for SpinMutexGuard<'_, T> {
    fn drop(&mut self) {
        self.unlock();
    }
}

use core::ops::Deref;
impl<T> Deref for SpinMutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

use core::ops::DerefMut;
impl<T> DerefMut for SpinMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

pub struct SpinMutexErr<'a>(&'a str);