use std::alloc::{self, Layout};
use std::fmt::Debug;
use std::ptr::{self, NonNull};

use crate::utils::{Array, DataType, IntoIter, Iter};

pub type F64 = Option<f64>;

/// Column of `f64` conforming to Apache Arrow's fix sized primitive
/// layout
pub struct ArrayF64 {
    /// Pointer to the values buffer
    ptr: Option<NonNull<f64>>,
    /// Pointer to the validity buffer
    val_ptr: Option<NonNull<u8>>,
    /// The number of elements in the array
    len: usize,
    /// The number of nulls in the array
    nulls: usize,
}

impl ArrayF64 {
    fn from_sized_iter<S>(sized: S) -> Self
    where
        S: Iterator<Item = F64> + ExactSizeIterator,
    {
        let len = sized.len();

        if len == 0 {
            return Self {
                ptr: None,
                val_ptr: None,
                len: 0,
                nulls: 0,
            };
        }

        let (values_ptr, validity_ptr) = Self::allocate(len);

        let mut val_byte = 0_u8;
        let mut val_offset = 0;
        let mut nulls = 0;

        for (idx, value) in sized.into_iter().enumerate() {
            match value {
                Some(value) => {
                    unsafe { ptr::write(values_ptr.as_ptr().add(idx), value) };
                    let pos = 1 << (idx % 8);
                    val_byte |= pos;
                }
                None => {
                    nulls += 1;
                    let pos = !(1 << (idx % 8));
                    val_byte &= pos;
                }
            }

            if (idx + 1) % 8 == 0 {
                unsafe {
                    ptr::write(validity_ptr.as_ptr().add(val_offset), val_byte);
                }

                val_byte = 0_u8;
                val_offset += 1;
            }
        }

        // Condition in for loop wouldn't have been triggered for the write
        if len % 8 != 0 {
            unsafe { ptr::write(validity_ptr.as_ptr().add(val_offset), val_byte) };
        }

        if nulls == 0 {
            Self::dealloc_validity(Some(validity_ptr), len);
        }

        if nulls == len {
            Self::dealloc_values(Some(values_ptr), len);
        }

        Self {
            ptr: if nulls == len { None } else { Some(values_ptr) },
            val_ptr: if nulls == 0 { None } else { Some(validity_ptr) },
            len,
            nulls,
        }
    }

    /// Creates an [`ArrayF64`] from a vec.
    pub fn from_vec(values: Vec<F64>) -> Self {
        Self::from_sized_iter(values.into_iter())
    }

    /// Returns true if the validity buffers of `Self` and `Other` are equal.
    ///
    /// Assumes both buffers are equal in length.
    fn compare_validity(&self, other: &Self) -> bool {
        let buffer_len = (self.len + 7) / 8;

        match (self.val_ptr, other.val_ptr) {
            (Some(own), Some(other)) => {
                for offset in 0..buffer_len {
                    let own = unsafe { ptr::read(own.as_ptr().add(offset)) };
                    let other = unsafe { ptr::read(other.as_ptr().add(offset)) };

                    if own != other {
                        return false;
                    }
                }
            }
            (None, Some(_)) => return false,
            (Some(_), None) => return false,
            (None, None) => return true,
        }

        true
    }

    /// Returns true if the values of `Self` and `Other` are equal.
    ///
    /// Assumes both buffers are equal in length.
    fn compare_values(&self, other: &Self) -> bool {
        let len = self.len;

        for idx in 0..len {
            let own = self.get(idx);
            let other = other.get(idx);

            if own != other {
                return false;
            }
        }

        true
    }

    /// Allocates both values and validity buffers
    ///
    /// Must ensure len != 0
    fn allocate(len: usize) -> (NonNull<f64>, NonNull<u8>) {
        // Values
        let values_size = len * std::mem::size_of::<f64>();
        let values_layout = Layout::from_size_align(values_size, 8)
            .expect("ArrayF64: values size overflowed isize::max");

        let values_ptr = unsafe { alloc::alloc(values_layout) };

        let values_ptr = match NonNull::new(values_ptr as *mut f64) {
            Some(ptr) => ptr,
            None => alloc::handle_alloc_error(values_layout),
        };

        // Validity
        let validity_size = (len + 7) / 8;
        let validity_layout = Layout::from_size_align(validity_size, 8)
            .expect("ArrayF64: validity size overflowed isize::max");

        let validity_ptr = unsafe { alloc::alloc(validity_layout) };

        let validity_ptr = match NonNull::new(validity_ptr) {
            Some(ptr) => ptr,
            None => alloc::handle_alloc_error(validity_layout),
        };

        (values_ptr, validity_ptr)
    }

    fn dealloc_validity(ptr: Option<NonNull<u8>>, len: usize) {
        let Some(val_ptr) = ptr else { return };
        let validity_size = (len + 7) / 8;
        let validity_layout = Layout::from_size_align(validity_size, 8)
            .expect("ArrayF64 drop: validity size overflowed isize::max");
        let ptr = val_ptr.as_ptr();
        unsafe { alloc::dealloc(ptr, validity_layout) };
    }

    fn dealloc_values(ptr: Option<NonNull<f64>>, len: usize) {
        let Some(ptr) = ptr else { return };
        let values_size = len * std::mem::size_of::<f64>();
        let values_layout = Layout::from_size_align(values_size, 8)
            .expect("ArrayF64 drop: values size overflowed isize::max");
        let ptr = ptr.as_ptr() as *mut u8;

        unsafe { alloc::dealloc(ptr, values_layout) };
    }
}

impl Array for ArrayF64 {
    type DataType = f64;

    fn new<I>(values: I) -> Self
    where
        I: IntoIterator<Item = Option<Self::DataType>>,
        I::IntoIter: ExactSizeIterator,
    {
        Self::from_sized_iter(values.into_iter())
    }

    fn get(&self, idx: usize) -> Option<Self::DataType> {
        if idx >= self.len {
            return None;
        }

        if self.is_null(idx) {
            return None;
        }

        let ptr = self.ptr?;
        let val = unsafe { ptr::read(ptr.as_ptr().add(idx)) };

        Some(val)
    }

    fn get_ref(&self, idx: usize) -> Option<&Self::DataType> {
        if idx >= self.len {
            return None;
        }

        if self.is_null(idx) {
            return None;
        }

        let ptr = self.ptr?;
        let val = unsafe {
            let ptr = ptr.as_ptr().add(idx);
            &*ptr
        };

        Some(val)
    }

    fn len(&self) -> usize {
        self.len
    }

    fn data_type(&self) -> DataType {
        DataType::F64
    }

    fn is_null(&self, idx: usize) -> bool {
        assert!(
            idx < self.len,
            "Tried to index {} when array length is {}",
            idx,
            self.len
        );
        let Some(val_ptr) = self.val_ptr else {
            return false;
        };

        let byte_index = idx / 8;

        let val_byte = unsafe { ptr::read(val_ptr.as_ptr().add(byte_index)) };
        val_byte & (1 << (idx % 8)) == 0
    }
}

impl Drop for ArrayF64 {
    fn drop(&mut self) {
        Self::dealloc_values(self.ptr, self.len());
        Self::dealloc_validity(self.val_ptr, self.len())
    }
}

impl Clone for ArrayF64 {
    fn clone(&self) -> Self {
        let iter = self.iter().map(|val| val.copied());
        Self::from_sized_iter(iter)
    }
}

impl Debug for ArrayF64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut vals = self
            .iter()
            .map(|val| match val {
                Some(val) => val.to_string(),
                None => "null".into(),
            })
            .peekable();

        let vals = {
            let mut acc = String::new();
            while let Some(val) = vals.next() {
                let join = match vals.peek() {
                    Some(_) => ", ",
                    None => "",
                };
                acc = format!("{acc}{val}{join}");
            }
            acc
        };

        write!(f, "ArrayF64 [{vals}]")
    }
}

impl PartialEq for ArrayF64 {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        if self.nulls != other.nulls {
            return false;
        }

        if !self.compare_validity(other) {
            return false;
        }

        if !self.compare_values(other) {
            return false;
        }

        true
    }
}

impl IntoIterator for ArrayF64 {
    type Item = Option<f64>;
    type IntoIter = IntoIter<Self>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

impl From<ArrayF64> for Vec<F64> {
    fn from(value: ArrayF64) -> Self {
        value.into_iter().collect()
    }
}

impl From<Vec<f64>> for ArrayF64 {
    fn from(value: Vec<f64>) -> Self {
        Self::from_sized_iter(value.into_iter().map(Some))
    }
}

impl From<Vec<F64>> for ArrayF64 {
    fn from(value: Vec<F64>) -> Self {
        Self::from_vec(value)
    }
}

impl<const N: usize> From<&[f64; N]> for ArrayF64 {
    fn from(value: &[f64; N]) -> Self {
        Self::from_sized_iter(value.iter().copied().map(Some))
    }
}

impl<const N: usize> From<&[F64; N]> for ArrayF64 {
    fn from(value: &[F64; N]) -> Self {
        Self::from_sized_iter(value.iter().copied())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::f64::consts;

    #[test]
    fn test_partial_eq() {
        let one = [
            Some(-consts::PI),
            Some(-10.0),
            None,
            Some(0.000),
            Some(consts::E),
            Some(std::f64::INFINITY),
            None,
        ];
        let one = ArrayF64::new(one);

        // Zero: Self equality without NaN
        assert_eq!(one, one);

        let none = [
            Some(-consts::PI),
            Some(-10.0),
            None,
            Some(0.000),
            Some(consts::E),
            Some(std::f64::INFINITY),
            None,
            Some(std::f64::NAN),
        ];
        let none = ArrayF64::new(none);

        // Zero: Self equality with NaN
        assert_ne!(none, none);

        // One: Perfect case
        let two = [
            Some(-consts::PI),
            Some(-10.0),
            None,
            Some(0.000),
            Some(consts::E),
            Some(std::f64::INFINITY),
            None,
        ];
        let two = ArrayF64::new(two);

        assert_eq!(one, two);
        // One: Symmetry
        assert_eq!(two, one);

        // Two: Varying order
        let two = [
            Some(-10.0),
            Some(-consts::PI),
            Some(0.000),
            None,
            Some(std::f64::INFINITY),
            Some(consts::E),
            None,
        ];
        let two = ArrayF64::new(two);

        assert_ne!(one, two);

        // Two: Varying order
        let two = [
            Some(-10.0),
            Some(-consts::PI),
            Some(0.000),
            Some(std::f64::INFINITY),
            Some(consts::E),
            None,
            None,
        ];
        let two = ArrayF64::new(two);

        assert_ne!(one, two);

        // Two: Varying order
        let two = vec![Some(0.0), Some(2.0), Some(4.0)];
        let two = ArrayF64::new(two);
        let three = vec![Some(4.0), Some(0.0), Some(2.0)];
        let three = ArrayF64::new(three);

        assert_ne!(three, two);

        // Four: Varying length
        let two = [
            Some(-consts::PI),
            Some(-10.0),
            None,
            Some(consts::E),
            Some(std::f64::INFINITY),
            None,
        ];
        let two = ArrayF64::new(two);

        assert_ne!(one, two);

        // Five: Varying null count
        let two = vec![None, None, None, None, Some(0.0)];
        let two = ArrayF64::new(two);

        assert_ne!(one, two);
    }

    #[test]
    fn test_into_iter() {
        let one = [Some(0.0), None, Some(2.0), None, Some(-4.2)];
        let one = ArrayF64::new(one);

        let mut iter = one.into_iter();

        assert_eq!(Some(0.0), iter.next().unwrap());
        assert_eq!(None, iter.next().unwrap());
        iter.next();
        assert_eq!(None, iter.next().unwrap());
        assert_eq!(Some(-4.2), iter.next().unwrap());
    }

    #[test]
    fn test_all_nulls() {
        let one = vec![None, None, None, None, None];

        let one = ArrayF64::new(one);

        assert_eq!(5, one.len());

        assert!(one.is_null(0));

        assert!(one.is_null(2));

        assert!(one.is_null(4));

        let mut iter = one.into_iter();

        assert_eq!(None, iter.next().unwrap());
        iter.next();
        assert_eq!(None, iter.next().unwrap())
    }

    #[test]
    fn test_empty() {
        let one = vec![];
        let one = ArrayF64::new(one);

        assert_eq!(0, one.len());
    }
}
