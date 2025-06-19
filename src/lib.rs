/*
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::mem::MaybeUninit;

pub struct VecRng<T>
{
    buffer: Box<[MaybeUninit<T>]>,
    findex: usize,
    length: usize,
}
impl<T> VecRng<T>
{
    pub fn with_capacity(c: usize)
        -> Self
    {
        Self
        {
            buffer: unsafe { Box::new_uninit_slice(c).assume_init() },
            findex: 0,
            length: 0,
        }
    }
    pub const fn lens(&self)
        -> (usize, usize)
    {
        let hcp = self.buffer.len() - self.findex;
        if hcp < self.length
        {
            (hcp, self.length - hcp)
        }
        else
        {
            (self.length, 0)
        }
    }
    pub const fn as_ref(&self)
        -> (&[T], &[T])
    {
        use std::ptr::slice_from_raw_parts as from;

        let (hln, bln) = self.lens();
        let ptr = &raw const *self.buffer as *const T;
        let h = unsafe { &*from(ptr.add(self.findex), hln) };
        let b = unsafe { &*from(ptr, bln) };
        (h, b)
    }
    pub const fn as_mut(&mut self)
        -> (&mut [T], &mut [T])
    {
        use std::ptr::slice_from_raw_parts_mut as from;

        let (hln, bln) = self.lens();
        let ptr = &raw mut *self.buffer as *mut T;
        let h = unsafe { &mut *from(ptr.add(self.findex), hln) };
        let b = unsafe { &mut *from(ptr, bln) };
        (h, b)
    }
}
impl<T> Drop for VecRng<T>
{
    fn drop(&mut self)
    {
        let (f, b) = self.as_mut();
        for e in unsafe { &mut *(f as *mut _ as *mut [MaybeUninit<T>]) }
        {
            unsafe { e.assume_init_drop() }
        }
        for e in unsafe { &mut *(b as *mut _ as *mut [MaybeUninit<T>]) }
        {
            unsafe { e.assume_init_drop() }
        }
    }
}
