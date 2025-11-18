use std::fmt;
use std::hash::{Hash, Hasher};

/// Location metadata for syntactic elements originating from parsed input.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct Location {
    pub file: Option<String>,
    pub line: usize,
    pub column: usize,
}

impl Location {
    pub fn new(file: Option<String>, line: usize, column: usize) -> Self {
        Self {
            file,
            line,
            column,
        }
    }

    pub fn dummy() -> Self {
        Self {
            file: None,
            line: 0,
            column: 0,
        }
    }

    pub fn next_line(&mut self) {
        self.line += 1;
        self.column = 0;
    }

    pub fn next_column(&mut self) {
        self.column += 1;
    }

    pub fn advance_by(&mut self, n: usize) {
        self.column += n;
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(file) = &self.file {
            write!(f, "{}:{}:{}", file, self.line, self.column)
        } else {
            write!(f, "{}:{}", self.line, self.column)
        }
    }
}

/// A value paired with location metadata.
#[derive(Debug, Clone)]
pub struct WithLoc<T> {
    pub value: T,
    pub location: Location,
}

impl<T> WithLoc<T> {
    pub fn new(value: T, location: Location) -> Self {
        Self {
            value,
            location,
        }
    }

    pub fn dummy(value: T) -> Self {
        Self {
            value,
            location: Location::dummy(),
        }
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> WithLoc<U> {
        WithLoc {
            value: f(self.value),
            location: self.location,
        }
    }

    pub fn map_ref<U>(&self, f: impl FnOnce(&T) -> U) -> WithLoc<U> {
        WithLoc {
            value: f(&self.value),
            location: self.location.clone(),
        }
    }

    pub fn as_ref(&self) -> WithLoc<&T> {
        WithLoc {
            value: &self.value,
            location: self.location.clone(),
        }
    }

    pub fn take_value(self) -> T {
        self.value
    }

    pub fn location(&self) -> &Location {
        &self.location
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T: PartialEq> PartialEq for WithLoc<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Eq> Eq for WithLoc<T> {}

impl<T: Default> Default for WithLoc<T> {
    fn default() -> Self {
        Self::dummy(T::default())
    }
}

impl<T: Hash> Hash for WithLoc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<T: fmt::Debug> fmt::Display for WithLoc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} @ {}", self.value, self.location)
    }
}

// the following two methoCs allow us to coerce &WithLoc<T> into &T for convenience
impl<T> std::ops::Deref for WithLoc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> std::ops::DerefMut for WithLoc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
