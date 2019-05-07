use crate::phase::*;
use dhall_proc_macros as dhall;
use dhall_syntax::*;

/// A value that has a statically-known Dhall type.
///
/// This trait is strictly more general than [SimpleStaticType].
/// The reason is that it allows an arbitrary [Type] to be returned
/// instead of just a [SimpleType].
///
/// For now the only interesting impl is [SimpleType] itself, who
/// has a statically-known type which is not itself a [SimpleType].
pub trait StaticType {
    fn get_static_type() -> Type;
}

/// A Rust type that can be represented as a Dhall type.
///
/// A typical example is `Option<bool>`,
/// represented by the dhall expression `Optional Bool`.
///
/// This trait can and should be automatically derived.
///
/// The representation needs to be independent of the value.
/// For this reason, something like `HashMap<String, bool>` cannot implement
/// [SimpleStaticType] because each different value would
/// have a different Dhall record type.
///
/// The `Simple` in `SimpleStaticType` indicates that the returned type is
/// a [SimpleType].
pub trait SimpleStaticType {
    fn get_simple_static_type() -> SimpleType;
}

fn mktype(x: SubExpr<X, X>) -> SimpleType {
    x.into()
}

impl<T: SimpleStaticType> StaticType for T {
    fn get_static_type() -> Type {
        crate::phase::Normalized::from_thunk_and_type(
            crate::core::thunk::Thunk::from_normalized_expr(
                T::get_simple_static_type().into(),
            ),
            Type::const_type(),
        )
        .to_type()
    }
}

impl StaticType for SimpleType {
    /// By definition, a [SimpleType] has type `Type`.
    /// This returns the Dhall expression `Type`
    fn get_static_type() -> Type {
        Type::const_type()
    }
}

impl SimpleStaticType for bool {
    fn get_simple_static_type() -> SimpleType {
        mktype(dhall::subexpr!(Bool))
    }
}

impl SimpleStaticType for Natural {
    fn get_simple_static_type() -> SimpleType {
        mktype(dhall::subexpr!(Natural))
    }
}

impl SimpleStaticType for u32 {
    fn get_simple_static_type() -> SimpleType {
        mktype(dhall::subexpr!(Natural))
    }
}

impl SimpleStaticType for u64 {
    fn get_simple_static_type() -> SimpleType {
        mktype(dhall::subexpr!(Natural))
    }
}

impl SimpleStaticType for Integer {
    fn get_simple_static_type() -> SimpleType {
        mktype(dhall::subexpr!(Integer))
    }
}

impl SimpleStaticType for i32 {
    fn get_simple_static_type() -> SimpleType {
        mktype(dhall::subexpr!(Integer))
    }
}

impl SimpleStaticType for i64 {
    fn get_simple_static_type() -> SimpleType {
        mktype(dhall::subexpr!(Integer))
    }
}

impl SimpleStaticType for String {
    fn get_simple_static_type() -> SimpleType {
        mktype(dhall::subexpr!(Text))
    }
}

impl<A: SimpleStaticType, B: SimpleStaticType> SimpleStaticType for (A, B) {
    fn get_simple_static_type() -> SimpleType {
        let ta: SubExpr<_, _> = A::get_simple_static_type().into();
        let tb: SubExpr<_, _> = B::get_simple_static_type().into();
        mktype(dhall::subexpr!({ _1: ta, _2: tb }))
    }
}

impl<T: SimpleStaticType> SimpleStaticType for Option<T> {
    fn get_simple_static_type() -> SimpleType {
        let t: SubExpr<_, _> = T::get_simple_static_type().into();
        mktype(dhall::subexpr!(Optional t))
    }
}

impl<T: SimpleStaticType> SimpleStaticType for Vec<T> {
    fn get_simple_static_type() -> SimpleType {
        let t: SubExpr<_, _> = T::get_simple_static_type().into();
        mktype(dhall::subexpr!(List t))
    }
}

impl<'a, T: SimpleStaticType> SimpleStaticType for &'a T {
    fn get_simple_static_type() -> SimpleType {
        T::get_simple_static_type()
    }
}

impl<T> SimpleStaticType for std::marker::PhantomData<T> {
    fn get_simple_static_type() -> SimpleType {
        mktype(dhall::subexpr!({}))
    }
}

impl<T: SimpleStaticType, E: SimpleStaticType> SimpleStaticType
    for std::result::Result<T, E>
{
    fn get_simple_static_type() -> SimpleType {
        let tt: SubExpr<_, _> = T::get_simple_static_type().into();
        let te: SubExpr<_, _> = E::get_simple_static_type().into();
        mktype(dhall::subexpr!(< Ok: tt | Err: te>))
    }
}