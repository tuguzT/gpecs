use core::{
    error::Error,
    fmt::{self, Debug, Display},
};

use crate::erased::error::IncompatibleArchetypeExactError;

#[derive(Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub struct IncompatibleBundleValueError<V> {
    pub value: V,
    pub source: IncompatibleArchetypeExactError,
}

impl<V> Display for IncompatibleBundleValueError<V>
where
    V: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use IncompatibleArchetypeExactError::{
            ComponentNotRegistered, DuplicateComponent, MissingComponent, TooFewComponents,
        };

        let Self { value, source } = self;

        write!(f, "incompatible bundle {value}: ")?;
        match source {
            DuplicateComponent(error) => Display::fmt(error, f),
            MissingComponent(error) => Display::fmt(error, f),
            ComponentNotRegistered(error) => Display::fmt(error, f),
            TooFewComponents(error) => Display::fmt(error, f),
        }
    }
}

impl<V> Error for IncompatibleBundleValueError<V>
where
    V: Debug + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { source, .. } = self;
        Some(source)
    }
}
