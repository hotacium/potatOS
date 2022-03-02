
use core::ops::{Bound, Range, RangeBounds};
pub trait BitField {
    const BIT_LENGTH: usize;

    #[must_use]
    fn get_bit(&self, bit: usize) -> bool;
    #[must_use]
    fn get_bits<T: RangeBounds<usize>>(&self, range: T) -> Self;
    #[must_use]
    fn set_bit(&mut self, bit: usize, val: bool) -> &mut Self;
    #[must_use]
    fn set_bits<T: RangeBounds<usize>>(&mut self, range: T, val: Self) -> &mut Self;
}

macro_rules! impl_bitfield {
    ( $($t: ty)* ) => ($(
        impl BitField for $t {
            const BIT_LENGTH: usize = core::mem::size_of::<Self>() as usize * 8;

            #[inline]
            fn get_bit(&self, bit: usize) -> bool {
                (*self & (1 << bit)) != 0
            }

            #[inline]
            fn get_bits<T: RangeBounds<usize>>(&self, range: T) -> Self {
                let range = to_regular_range(&range, Self::BIT_LENGTH);
                // shift away upper bits
                let upper_bits = Self::BIT_LENGTH - range.end;
                let lower_bits = range.start;
                let bits = *self << upper_bits >> upper_bits;
                // shit away lower bits
                bits >> lower_bits
            }

            #[inline]
            fn set_bit(&mut self, bit: usize, val: bool) -> &mut Self {
                if val {
                    *self |= 1 << bit;
                } else {
                    *self &= !(1 << bit);
                }
                self
            }

            #[inline]
            fn set_bits<T: RangeBounds<usize>>(&mut self, range: T, val: Self) -> &mut Self {
                let range = to_regular_range(&range, Self::BIT_LENGTH);

                let upper_bits = Self::BIT_LENGTH - range.end;
                let lower_bits = range.start;
                // Ex: u8, upper_bits = 2, lower_bits = 1,
                // 1. !0u8 = 0b1111_1111
                // 2. !0 << 2 >> 2 = 0b0011_1111
                // 3. !0 << 2 >> 2 >> 1 << 1 = 0b0011_1110
                // 4. !(!0 << 2 >> 2 >> 1 << 1) = 0b1100_0001
                let bitmask: Self = !(
                    !0 << upper_bits >> upper_bits
                    >> lower_bits << lower_bits
                );

                *self = (*self & bitmask) | (val << lower_bits);

                self
            }
        }
    )*)
}

impl_bitfield! { u8 u16 u32 u64 u128 usize }

fn to_regular_range<T: RangeBounds<usize>>(range: &T, bit_length: usize) -> Range<usize> {
    let start = match range.start_bound() {
        Bound::Excluded(&value) => value +1,
        Bound::Included(&value) => value,
        Bound::Unbounded => 0,
    };
    let end = match range.end_bound() {
        Bound::Excluded(&value) => value,
        Bound::Included(&value) => value+1,
        Bound::Unbounded => bit_length,
    };
    start..end
}