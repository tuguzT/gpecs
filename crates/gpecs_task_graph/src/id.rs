macro_rules! id {
    ($(#[$outer:meta])* $vis:vis $name:ident ($type:ty) $(;)?) => {
        $(#[$outer])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(transparent)]
        $vis struct $name($type);

        impl $name {
            pub const fn new(id: $type) -> Self {
                Self(id)
            }

            pub const fn empty() -> Self {
                Self(<$type>::MAX)
            }

            pub const fn is_empty(&self) -> bool {
                let Self(value) = self;
                *value == <$type>::MAX
            }

            pub const fn into_inner(self) -> $type {
                let Self(value) = self;
                value
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::empty()
            }
        }

        impl From<$type> for $name {
            fn from(value: $type) -> Self {
                Self::new(value)
            }
        }

        impl From<$name> for $type {
            fn from(value: $name) -> Self {
                value.into_inner()
            }
        }
    };
    ($(#[$outer:meta])* $vis:vis $name:ident $(;)?) => {
        $crate::id::id!(
            $(#[$outer])*
            $vis $name(u32)
        );
    };
}

pub(crate) use id;

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    #[test]
    fn new() {
        id!(pub SomeId);
        let id = SomeId::new(42);

        assert!(!id.is_empty());
        assert_eq!(id.into_inner(), 42);
    }

    #[test]
    fn empty() {
        id!(pub SomeId);
        let id = SomeId::empty();

        assert!(id.is_empty());
        assert_eq!(id.into_inner(), u32::MAX);
    }

    #[test]
    fn same_size() {
        id!(pub SameSizeId(u64));

        assert_eq!(size_of::<SameSizeId>(), size_of::<u64>());
    }
}
