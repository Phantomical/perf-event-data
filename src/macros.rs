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
            $(
                $( #[debug($( $dbgopt:tt )*)] )?
                $fvis:vis $field:ident : $ty:ty
            ),* $(,)?
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
                        // if let Some($field) = self.$field() {
                        //     dbg.field(stringify!($field), $field);
                        // }
                        option_struct!(
                            impl(debug_field $( , #[debug($( $dbgopt )*)] )? )
                            self, dbg, $field, $ty
                        );
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
    };
    (impl(debug_field, #[debug(with = $fmt:expr)]) $self:expr, $dbg:expr, $field:ident, $fieldty:ty) => {
        if let Some($field) = $self.$field() {
            $dbg.field(stringify!($field), &$fmt($field));
        }
    };
    (impl(debug_field) $self:expr, $dbg:expr, $field:ident, $fieldty:ty) => {
        if let Some($field) = $self.$field() {
            $dbg.field(stringify!($field), $field);
        }
    }
}
