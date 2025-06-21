/*
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::{mem::{MaybeUninit, transmute}, ptr::NonNull};

const fn dangling_boxed_slice<T>()
    -> Box<[T]>
{
    let ptr = NonNull::slice_from_raw_parts(NonNull::<T>::dangling(), 0);
    unsafe { transmute(ptr) }
}

pub struct VecRng<T>
{
    /* LAYOUT:
     * [ back | uninit | head ]
     * e.g. buffer.len() == 3 && hindex == 2 && length == 2
     * OR
     * [ uninit | contig | uninit ]
     * e.g. buffer.len() == 3 && hindex == 1 && length == 1
     */
    buffer: Box<[MaybeUninit<T>]>,
    hindex: usize,
    length: usize,
}
impl<T> VecRng<T>
{
    const MINCAP: usize =
    {
        let siz = size_of::<T>();
        if siz == 1 { 8 }
        else if siz <= 1024 { 4 }
        else { 1 }
    };
    pub const fn new()
        -> Self
    {
        Self
        {
            buffer: dangling_boxed_slice(),
            hindex: 0,
            length: 0,
        }
    }
    pub fn with_capacity(c: usize)
        -> Self
    {
        let mut ret = Self::new();
        ret.grow(Self::MINCAP.max(c));
        ret
    }
    fn grow(&mut self, to_c: usize)
    {
        /* panics if `to_c > isize::MAX` */
        let mut newbuf = Box::<[T]>::new_uninit_slice(to_c);
        let (hln, bln) = self.lens();
        let src = self.buffer.as_mut_ptr();
        let dst = newbuf.as_mut_ptr();
        unsafe { src.copy_to_nonoverlapping(dst.add(hln), bln); }
        unsafe { src.add(self.hindex).copy_to_nonoverlapping(dst, hln); }
        self.buffer = newbuf;
        self.hindex = 0;
    }
    pub fn reserve(&mut self, addc: usize)
    {
        let to_c = addc.checked_add(self.length).unwrap();
        let ccap = self.buffer.len();
        if to_c < ccap
        {
            return ();
        }
        self.grow(Self::MINCAP.max(to_c).max(ccap << 1));
    }
    /* SAFETY:
     * 0 <= length + n <= buffer.len()
     * all values in the resulting head and back are init
     *
     * n > 0: assume the `n` values before head to be properly init.
     * n < 0: forget the `n` values at the start of head to be init.
     */
    pub const unsafe fn head_init_change(&mut self, n: isize)
    {
        self.hindex =
        /* SAFETY (not memory safety):
         * these casts does not overflow because allocated objects cannot
         * have sizes greater than `isize::MAX`; thus:
         * isize::MAX >= self.buffer.len() >= self.length > self.hindex
         */
        ((self.hindex as isize - n) % self.buffer.len() as isize)
        as usize;
        unsafe { self.back_init_change(n); }
    }
    /* SAFETY:
     * 0 <= length + n <= buffer.len()
     * all values in the resulting head and back are init
     *
     * n > 0: assume the `n` values after back to be properly init.
     * n < 0: forget the `n` values at the end of back to be init.
     */
    pub const unsafe fn back_init_change(&mut self, n: isize)
    {
        self.length = (self.length as isize + n) as usize;
    }
    #[inline] /* desperately needs optimization on repeated calls. */
    pub const fn lens(&self)
        -> (usize, usize)
    {
        let hcp = self.buffer.len() - self.hindex;
        if hcp < self.length
        {
            (hcp, self.length - hcp)
        }
        else
        {
            (self.length, 0)
        }
    }
    #[inline] /* desperately needs optimization on repeated calls. */
    pub const fn as_ref(&self)
        -> (&[T], &[T])
    {
        use std::ptr::slice_from_raw_parts as from;

        let (hln, bln) = self.lens();
        let ptr = &raw const *self.buffer as *const T;
        let h = unsafe { &*from(ptr.add(self.hindex), hln) };
        let b = unsafe { &*from(ptr, bln) };
        (h, b)
    }
    #[inline] /* desperately needs optimization on repeated calls. */
    pub const fn as_mut(&mut self)
        -> (&mut [T], &mut [T])
    {
        use std::ptr::slice_from_raw_parts_mut as from;

        let (hln, bln) = self.lens();
        let ptr = &raw mut *self.buffer as *mut T;
        let h = unsafe { &mut *from(ptr.add(self.hindex), hln) };
        let b = unsafe { &mut *from(ptr, bln) };
        (h, b)
    }
    #[inline] /* desperately needs optimization on repeated calls. */
    pub const fn spare_capacity_mut(&mut self)
        -> (&mut [MaybeUninit<T>], &mut [MaybeUninit<T>])
    {
        use std::ptr::slice_from_raw_parts_mut as from;

        let (hln, bln) = self.lens();
        let cap = self.buffer.len();
        let ptr = &raw mut *self.buffer as *mut MaybeUninit<T>;
        let a = self.hindex + hln;
        // [ back | uninit | head ] or [ uninit | contig | uninit ]
        //                        ^ here (empty)           ^ here
        unsafe { (&mut *from(ptr.add(a), cap - a),
        // [ back | uninit | head ] or [ uninit | contig | uninit ]
        //          ^ here               ^ here
        &mut *from(ptr.add(bln), self.hindex - bln)) }
    }
}
impl<T> Drop for VecRng<T>
{
    fn drop(&mut self)
    {
        let (h, b) = self.as_mut();
        for e in h
        {
            unsafe { (e as *mut T).drop_in_place() }
        }
        for e in b
        {
            unsafe { (e as *mut T).drop_in_place() }
        }
    }
}
