use std::borrow::Cow;

pub(crate) trait CowSliceExt: Sized {
    /// Divide the cow into two, splitting it at `len`.
    ///
    /// This should have the same behaviour as split_at on a slice.
    ///
    /// # Panics
    /// Panics if `len > self.len()`
    fn split_at(self, len: usize) -> (Self, Self);

    /// Same as `Vec::truncate`.
    fn truncate(&mut self, len: usize);
}

impl<'a, T> CowSliceExt for Cow<'a, [T]>
where
    T: Clone,
{
    fn split_at(self, len: usize) -> (Self, Self) {
        match self {
            Self::Borrowed(slice) => {
                let (head, rest) = slice.split_at(len);
                (head.into(), rest.into())
            }
            Self::Owned(data) if len == 0 => (data.into(), Cow::Borrowed(&[])),
            Self::Owned(data) if len == data.len() => (Cow::Borrowed(&[]), data.into()),
            Self::Owned(mut data) => {
                let rest: Vec<T> = data.drain(len..).collect();
                (data.into(), rest.into())
            }
        }
    }

    fn truncate(&mut self, len: usize) {
        match self {
            Cow::Owned(data) => data.truncate(len),
            Cow::Borrowed(slice) if slice.len() < len => (),
            Cow::Borrowed(slice) => *slice = slice.split_at(len).0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_owned() {
        let mut cow: Cow<[u8]> = Cow::Owned(vec![0u8, 1, 2, 3, 4, 5]);
        cow.truncate(4);

        assert_eq!(&cow[..], &[0, 1, 2, 3]);
    }

    #[test]
    fn truncate_borrowed() {
        let mut cow: Cow<[u8]> = Cow::Borrowed(&[0, 1, 2, 3, 4, 5]);
        cow.truncate(4);

        assert_eq!(&cow[..], &[0, 1, 2, 3]);
    }
}
