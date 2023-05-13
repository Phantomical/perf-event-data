//! Macros used within perf-event-data.

/// A helper macro for silencing warnings when a type is only implemented so
/// that it can be linked in the docs.
macro_rules! used_in_docs {
    ($( $t:ident ),+) => {
        const _: () = {
            // Using a module here means that this macro can accept any identifier that
            // would normally be used in an import statement.
            #[allow(unused_imports)]
            mod use_item {
                $( use super::$t; )+
            }
        };
    };
}

/// Macro for defining a binding to a C-like enum.
///
/// Normally, we would like to use a rust enum to represent a C enum. However,
/// with an interface like perf_event_open most of the enums we are dealing
/// with can gain new variants in a backwards compatible manner. If we tried
/// to use rust enums for this we'd end up with messy conversions and an
/// awkward `Unknown(x)` variant on every enum. In addition, adding a new
/// variant would break downstream code that was relying on `Unknown(x)`
/// working.
///
/// The solution to this is to not use rust enums. Instead, we define a C enum
/// wrapper struct like this
/// ```
/// pub struct MyEnum(pub u32);
/// ```
/// and then add associated constants for all the enum variants.
///
/// This macro is a helper macro (in the style of bitflags!) which defines an
/// enum as described above and also derives a specialized Debug impl for it.
///
/// # Example
/// If we declare a simple enum like this
/// ```ignore
/// c_enum! {
///     /// Insert docs here
///     pub struct SomeEnum : u32 {
///         const A = 0;
///         const B = 1;
///     }
/// }
/// ```
///
/// Then the resulting rust code would look (roughly) like this
/// ```
/// /// Insert docs here
/// #[derive(Copy, Clone, Eq, PartialEq, Hash, Default)]
/// pub struct SomeEnum(pub u32);
///
/// #[allow(missing_docs)]
/// impl SomeEnum {
///     pub const A: Self = Self(0);
///     pub const B: Self = Self(1);
/// }
///
/// impl std::fmt::Debug for SomeEnum {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         match self {
///             &Self::A => f.write_str("SomeEnum::A"),
///             &Self::B => f.write_str("SomeEnum::B"),
///             Self(value) => f.debug_tuple("SomeEnum").field(value).finish(),
///         }
///     }
/// }
///
/// impl std::convert::From<u32> for SomeEnum {
///     fn from(value: u32) -> Self {
///         Self(value)
///     }
/// }
/// ```
macro_rules! c_enum {
    {
        $( #[doc = $doc:expr] )*
        $( #[allow($warning:ident)] )*
        $vis:vis struct $name:ident : $inner:ty {
            $(
                $( #[ $field_attr:meta ] )*
                const $field:ident = $value:expr;
            )*
        }
    } => {
        $( #[doc = $doc] )*
        $( #[allow($warning)] )*
        #[derive(Copy, Clone, Eq, PartialEq, Hash)]
        $vis struct $name(pub $inner);

        $( #[allow($warning)] )*
        impl $name {
            $(
                $( #[$field_attr] )*
                pub const $field: Self = Self($value);
            )*
        }

        impl $name {
            #[doc = concat!("Create a new `", stringify!($name), "` from a `", stringify!($inner), "`.")]
            pub const fn new(value: $inner) -> Self {
                Self(value)
            }
        }

        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match *self {
                    $( Self::$field => f.write_str(concat!(stringify!($name), "::", stringify!($field))), )*
                    Self(value) => f.debug_tuple(stringify!($name)).field(&value).finish()
                }
            }
        }

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }
    }
}

macro_rules! first_arg {
    {
        { $( $first:tt )*}
        $( { $( $rest:tt )* } )*
    } => {
        $( $first )*
    }
}

/// Helper macro for defining Debug implementations.
///
/// This is useful because we happen to have bindings for structs which have
/// lots of optional fields that may not be relevant (e.g. `Sample`). In the
/// debug impl we don't want to show the fields that people haven't configured
/// but writing out the relevant debug impl by hand is very repetitive.
///
/// # Example
/// The declaration
/// ```ignore
/// let dbg = debug_if! {
///     f.debug_struct("Test") => {
///         a => self.a,
///         b if should_do_b() => self.b()
///     }
/// }
/// ```
/// will (roughly) expand to
/// ```ignore
/// let dbg = {
///     let mut dbg = f.debug_struct("Test");
///
///     dbg.field("a", &self.a);
///     if should_do_b() {
///         dbg.field("b", &self.b());
///     }
///
///     dbg
/// }
/// ```
macro_rules! debug_if {
    {
        $dbg:expr => {
            $( $field:ident $( if $ifexpr:expr )? => $value:expr ),*  $(,)?
        }
    } => {{
        let mut dbg = $dbg;

        $(
            if first_arg!($( { $ifexpr } )? { true }) {
                dbg.field(stringify!($field), &$value);
            }
        )*

        dbg
    }}
}

macro_rules! option_struct {
    {
        $( #[$attr:meta] )*
        $( ##[copy $( $copy:tt )?] )?
        $vis:vis struct $name:ident$(<$lt:lifetime>)?: $flag:ty {
            $( $fvis:vis $field:ident : $ty:ty ),* $(,)?
        }
    } => {
        $( #[$attr] )*
        $( #[derive(Copy, $( _ $copy:tt )?)] )?
        $vis struct $name$(<$lt>)? {
            __flags: $flag,

            $( $field : ::std::mem::MaybeUninit<$ty>, )*
        }

        const _: () = {
            use std::fmt;

            pub enum Offsets {}

            #[allow(dead_code, non_upper_case_globals)]
            impl Offsets {
                const FIELD_COUNT: u32 = {
                    let count = 0u32 $( + first_arg!({ 1 } { stringify!($field) }) )*;
                    assert!(count < <$flag>::BITS, "too many fields for the flag type");
                    count
                };

                option_struct!(impl(index_consts, 0, $flag) $( $field )*);
            }

            #[allow(dead_code)]
            impl$(<$lt>)? $name$(<$lt>)? {
                #[allow(clippy::too_many_arguments)]
                pub fn new(
                    $( $field : Option<$ty> ),*
                ) -> Self {
                    assert!(Offsets::FIELD_COUNT < <$flag>::BITS);

                    let mut __flags = 0;

                    Self {
                        $(
                            $field: match $field {
                                Some(val) => {
                                    __flags |= 1 << Offsets::$field;
                                    ::std::mem::MaybeUninit::new(val)
                                },
                                None => ::std::mem::MaybeUninit::uninit(),
                            },
                        )*
                        __flags
                    }
                }

                $(
                    #[inline]
                    $fvis const fn $field(&self) -> Option<&$ty> {
                        if (self.__flags & (1 << Offsets::$field)) != 0 {
                            Some(unsafe { self.$field.assume_init_ref() })
                        } else {
                            None
                        }
                    }
                )*
            }

            option_struct!(impl(drop $(, #[copy $($copy:tt)?])?) $name, $($lt,)? $( $field, )*);

            impl$(<$lt>)? Clone for $name$(<$lt>)?
            where
                $( $ty : Clone ),*
            {
                fn clone(&self) -> Self {
                    Self::new(
                        $( self.$field().cloned() ),*
                    )
                }
            }

            impl$(<$lt>)? fmt::Debug for $name$(<$lt>)? {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    let mut dbg = f.debug_struct(stringify!($name));

                    $(
                        if let Some($field) = self.$field() {
                            dbg.field(stringify!($field), $field);
                        }
                    )*

                    dbg.finish_non_exhaustive()
                }
            }

            impl$(<$lt>)? Default for $name$(<$lt>)? {
                fn default() -> Self {
                    let mut this = ::std::mem::MaybeUninit::<Self>::uninit();

                    unsafe {
                        ::std::ptr::addr_of_mut!((*this.as_mut_ptr()).__flags).write(0);
                        this.assume_init()
                    }
                }
            }
        };


    };
    (impl(index_consts, $index:expr, $ty:ty)) => {};
    (impl(index_consts, $index:expr, $ty:ty) $first:ident $($rest:ident)*) => {
        const $first: $ty = $index;
        option_struct!(impl(index_consts, $index + 1, $ty) $( $rest )*);
    };
    (impl(drop, #[copy]) $name:ident $(, $lt:lifetime)? $(, $field:ident )* $(,)?) => {};
    (impl(drop         ) $name:ident $(, $lt:lifetime)? $(, $field:ident )* $(,)?) => {
        impl$(<$lt>)? Drop for $name$(<$lt>)? {
            fn drop(&mut self) {
                $(
                    if (self.__flags & (1 << Offsets::$field)) != 0 {
                        unsafe { self.$field.assume_init_drop() };
                    }

                )*
            }
        }
    }

}
