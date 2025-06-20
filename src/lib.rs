/*
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::mem::MaybeUninit;

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
    pub fn with_capacity(c: usize)
        -> Self
    {
        Self
        {
            buffer: unsafe { Box::new_uninit_slice(c).assume_init() },
            hindex: 0,
            length: 0,
        }
    }
    const unsafe fn head_assume_op(&mut self, n: isize)
    {
        self.hindex =
        /* SAFETY (not memory safety):
         * these casts does not overflow because allocated objects cannot
         * have sizes greater than `isize::MAX`; thus:
         * isize::MAX >= self.buffer.len() >= self.length > self.hindex
         */
        ((self.hindex as isize - n) % self.buffer.len() as isize)
        as usize;
        self.length = (self.length as isize + n) as usize;
    }
    /* SAFETY:
     * length + n <= buffer.len()
     * all values in the resulting head and back are init
     *
     * assume the `n` values before head to be properly init.
     */
    pub const unsafe fn head_assume_init(&mut self, n: usize)
    {
        unsafe { self.head_assume_op(n as isize); }
    }
    /* SAFETY:
     * length - n > 0
     * all values in the resulting head and back are init
     *
     * forget to drop the first `n` values in head.
     */
    pub const unsafe fn head_forget_init(&mut self, n: usize)
    {
        unsafe { self.head_assume_op(-(n as isize)); }
    }
    pub const unsafe fn back_assume_init(&mut self, n: usize)
    {
        self.length += n;
    }
    pub const unsafe fn back_forget_init(&mut self, n: usize)
    {
        self.length -= n;
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
