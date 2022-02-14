
#[derive(Default, Debug)]
pub struct InitOnce<T> {
    inner: Option<T>,
}

impl<T> InitOnce<T> {

    pub fn init<F>(&mut self, f: F) 
    where F: FnOnce() -> T {
        if self.inner.is_none() {
            self.inner = Some(f());
        }
    }

    pub fn is_initalized(&self) -> bool {
        self.inner.is_some()
    }

    pub fn get_or_init<F, E>(&mut self, f: F) -> &T
    where F: FnOnce() -> T  {
        match self.get_or_try_init( || Ok::<T, E>(f()) ) {
            Ok(val) => val,
            Err(_) => panic!(), // should be initalized (unreachable)
        }
    }

    pub fn get_or_try_init<F, E>(&mut self, f: F) -> Result<&T, E>
    where 
        F: FnOnce() -> Result<T, E> 
    {   
        if self.inner.is_some() {
            return Ok(self.inner.as_ref().unwrap());
        }

        let val = f()?;
        assert!(self.set(val).is_ok(), "already initialized");
        Ok(self.get().unwrap())

    }

    pub fn get(&self) -> Option<&T> {
        self.inner.as_ref()
    }

    pub fn set(&mut self, val: T) -> Result<(), T> {
        if self.get().is_some() {
            Err(val)
        } else {
            self.inner = Some(val);
            Ok(())
        }
    }

    pub fn take(&mut self) -> Option<T> {
        self.inner.take()
    }

    pub fn into_inner(self) -> Option<T> {
        self.inner
    }
}


