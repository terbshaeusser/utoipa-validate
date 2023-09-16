use regex::Regex;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

pub enum ValidationPath<'a, 'b> {
    Root,
    Field {
        parent: &'b ValidationPath<'a, 'a>,
        name: &'a str,
    },
    Item {
        parent: &'b ValidationPath<'a, 'a>,
        index: usize,
    },
}

impl Display for ValidationPath<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationPath::Root => Ok(()),
            ValidationPath::Field {
                parent: ValidationPath::Root,
                name,
            } => write!(f, "{}", name),
            ValidationPath::Field { parent, name } => write!(f, "{}.{}", parent, name),
            ValidationPath::Item {
                parent: ValidationPath::Root,
                index,
            } => write!(f, "[{}]", index),
            ValidationPath::Item { parent, index } => write!(f, "{}[{}]", parent, index),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ValidationError {
    ExclusiveMaximum {
        field: String,
        actual: String,
        maximum: String,
    },
    ExclusiveMinimum {
        field: String,
        actual: String,
        minimum: String,
    },
    Maximum {
        field: String,
        actual: String,
        maximum: String,
    },
    Minimum {
        field: String,
        actual: String,
        minimum: String,
    },
    MaxItems {
        field: String,
        actual: usize,
        max_items: usize,
    },
    MinItems {
        field: String,
        actual: usize,
        min_items: usize,
    },
    MaxLength {
        field: String,
        actual: usize,
        max_length: usize,
    },
    MinLength {
        field: String,
        actual: usize,
        min_length: usize,
    },
    Pattern {
        field: String,
        actual: String,
        pattern: String,
    },
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::ExclusiveMaximum {
                field,
                actual,
                maximum,
            } => write!(
                f,
                "{}: Must be less than {} but is {}",
                field, maximum, actual
            ),
            ValidationError::ExclusiveMinimum {
                field,
                actual,
                minimum,
            } => write!(
                f,
                "{}: Must be greater than {} but is {}",
                field, minimum, actual
            ),
            ValidationError::Maximum {
                field,
                actual,
                maximum,
            } => write!(
                f,
                "{}: Must be less than or equal to {} but is {}",
                field, maximum, actual
            ),
            ValidationError::Minimum {
                field,
                actual,
                minimum,
            } => write!(
                f,
                "{}: Must be greater than or equal to {} but is {}",
                field, minimum, actual
            ),
            ValidationError::MaxItems {
                field,
                actual,
                max_items,
            } => write!(
                f,
                "{}: Must have at most {} items but has {}",
                field, max_items, actual
            ),
            ValidationError::MinItems {
                field,
                actual,
                min_items,
            } => write!(
                f,
                "{}: Must have at least {} items but has {}",
                field, min_items, actual
            ),
            ValidationError::MaxLength {
                field,
                actual,
                max_length,
            } => write!(
                f,
                "{}: Must have at most {} characters but has {}",
                field, max_length, actual
            ),
            ValidationError::MinLength {
                field,
                actual,
                min_length,
            } => write!(
                f,
                "{}: Must have at least {} characters but has {}",
                field, min_length, actual
            ),
            ValidationError::Pattern {
                field,
                actual,
                pattern,
            } => write!(
                f,
                "{}: Must match the regular expression {} but is {}",
                field, pattern, actual
            ),
        }
    }
}

pub trait Validator<T> {
    fn validate(&self, path: &ValidationPath, value: &T, errors: &mut Vec<ValidationError>);
}

pub trait Validatable: Sized {
    type DefaultValidator: Validator<Self> + Default;

    fn validate_default(&self, path: &ValidationPath, errors: &mut Vec<ValidationError>) {
        Self::DefaultValidator::default().validate(path, self, errors);
    }

    fn validate_with<V>(&self, validator: &V) -> Result<(), Vec<ValidationError>>
        where
            V: Validator<Self>,
    {
        let mut errors = Vec::new();
        validator.validate(&ValidationPath::Root, self, &mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        self.validate_with(&Self::DefaultValidator::default())
    }
}

#[derive(Default)]
pub struct AlwaysValidValidator {}

impl<T> Validator<T> for AlwaysValidValidator {
    fn validate(&self, _path: &ValidationPath, _value: &T, _errors: &mut Vec<ValidationError>) {}
}

macro_rules! validatable {
    ($type:ty) => {
        impl Validatable for $type {
            type DefaultValidator = AlwaysValidValidator;
        }
    };
}

validatable!(bool);
validatable!(i8);
validatable!(i16);
validatable!(i32);
validatable!(i64);
validatable!(isize);
validatable!(u8);
validatable!(u16);
validatable!(u32);
validatable!(u64);
validatable!(usize);
validatable!(f32);
validatable!(f64);
validatable!(char);
validatable!(String);

pub struct OptionValidator<T, V>
    where
        T: Validatable,
        V: Validator<T>,
{
    inner: V,
    phantom: PhantomData<T>,
}

impl<T, V> OptionValidator<T, V>
    where
        T: Validatable,
        V: Validator<T>,
{
    pub fn new(inner: V) -> Self {
        Self {
            inner,
            phantom: PhantomData::default(),
        }
    }
}

impl<T: Validatable> Default for OptionValidator<T, T::DefaultValidator> {
    fn default() -> Self {
        Self {
            inner: T::DefaultValidator::default(),
            phantom: PhantomData::default(),
        }
    }
}

impl<T, V> Validator<Option<T>> for OptionValidator<T, V>
    where
        T: Validatable,
        V: Validator<T>,
{
    fn validate(
        &self,
        path: &ValidationPath,
        value: &Option<T>,
        errors: &mut Vec<ValidationError>,
    ) {
        if let Some(value) = value {
            self.inner.validate(path, value, errors);
        }
    }
}

impl<T> Validatable for Option<T>
    where
        T: Validatable,
{
    type DefaultValidator = OptionValidator<T, T::DefaultValidator>;
}

pub struct VecValidator<T, V>
    where
        T: Validatable,
        V: Validator<T>,
{
    inner: V,
    phantom: PhantomData<T>,
}

impl<T: Validatable> Default for VecValidator<T, T::DefaultValidator> {
    fn default() -> Self {
        Self {
            inner: T::DefaultValidator::default(),
            phantom: PhantomData::default(),
        }
    }
}

impl<T, V> Validator<Vec<T>> for VecValidator<T, V>
    where
        T: Validatable,
        V: Validator<T>,
{
    fn validate(&self, path: &ValidationPath, value: &Vec<T>, errors: &mut Vec<ValidationError>) {
        for (index, item) in value.iter().enumerate() {
            let item_path = ValidationPath::Item {
                parent: path,
                index,
            };

            self.inner.validate(&item_path, item, errors);
        }
    }
}

impl<T> Validatable for Vec<T>
    where
        T: Validatable,
{
    type DefaultValidator = VecValidator<T, T::DefaultValidator>;
}

pub struct ExclusiveMaximumValidator<T: PartialOrd + Display> {
    max: T,
}

impl<T> ExclusiveMaximumValidator<T>
    where
        T: PartialOrd + Display,
{
    pub fn new(max: T) -> Self {
        Self { max }
    }
}

impl<T> Validator<T> for ExclusiveMaximumValidator<T>
    where
        T: PartialOrd + Display,
{
    fn validate(&self, path: &ValidationPath, value: &T, errors: &mut Vec<ValidationError>) {
        if *value >= self.max {
            errors.push(ValidationError::ExclusiveMaximum {
                field: path.to_string(),
                actual: value.to_string(),
                maximum: self.max.to_string(),
            });
        }
    }
}

pub struct ExclusiveMinimumValidator<T: PartialOrd + Display> {
    min: T,
}

impl<T> ExclusiveMinimumValidator<T>
    where
        T: PartialOrd + Display,
{
    pub fn new(min: T) -> Self {
        Self { min }
    }
}

impl<T> Validator<T> for ExclusiveMinimumValidator<T>
    where
        T: PartialOrd + Display,
{
    fn validate(&self, path: &ValidationPath, value: &T, errors: &mut Vec<ValidationError>) {
        if *value <= self.min {
            errors.push(ValidationError::ExclusiveMinimum {
                field: path.to_string(),
                actual: value.to_string(),
                minimum: self.min.to_string(),
            });
        }
    }
}

pub struct MaximumValidator<T: PartialOrd + Display> {
    max: T,
}

impl<T> MaximumValidator<T>
    where
        T: PartialOrd + Display,
{
    pub fn new(max: T) -> Self {
        Self { max }
    }
}

impl<T> Validator<T> for MaximumValidator<T>
    where
        T: PartialOrd + Display,
{
    fn validate(&self, path: &ValidationPath, value: &T, errors: &mut Vec<ValidationError>) {
        if *value > self.max {
            errors.push(ValidationError::Maximum {
                field: path.to_string(),
                actual: value.to_string(),
                maximum: self.max.to_string(),
            });
        }
    }
}

pub struct MinimumValidator<T: PartialOrd + Display> {
    min: T,
}

impl<T> MinimumValidator<T>
    where
        T: PartialOrd + Display,
{
    pub fn new(min: T) -> Self {
        Self { min }
    }
}

impl<T> Validator<T> for MinimumValidator<T>
    where
        T: PartialOrd + Display,
{
    fn validate(&self, path: &ValidationPath, value: &T, errors: &mut Vec<ValidationError>) {
        if *value < self.min {
            errors.push(ValidationError::Minimum {
                field: path.to_string(),
                actual: value.to_string(),
                minimum: self.min.to_string(),
            });
        }
    }
}

pub struct MaxLengthValidator {
    max_length: usize,
}

impl MaxLengthValidator {
    pub fn new(max_length: usize) -> Self {
        Self { max_length }
    }
}

impl Validator<String> for MaxLengthValidator {
    fn validate(&self, path: &ValidationPath, value: &String, errors: &mut Vec<ValidationError>) {
        if value.len() > self.max_length {
            errors.push(ValidationError::MaxLength {
                field: path.to_string(),
                actual: value.len(),
                max_length: self.max_length,
            });
        }
    }
}

pub struct MinLengthValidator {
    min_length: usize,
}

impl MinLengthValidator {
    pub fn new(min_length: usize) -> Self {
        Self { min_length }
    }
}

impl Validator<String> for MinLengthValidator {
    fn validate(&self, path: &ValidationPath, value: &String, errors: &mut Vec<ValidationError>) {
        if value.len() < self.min_length {
            errors.push(ValidationError::MinLength {
                field: path.to_string(),
                actual: value.len(),
                min_length: self.min_length,
            });
        }
    }
}

pub struct PatternValidator {
    pattern: Regex,
}

impl PatternValidator {
    pub fn new(pattern: Regex) -> Self {
        Self { pattern }
    }
}

impl Validator<String> for PatternValidator {
    fn validate(&self, path: &ValidationPath, value: &String, errors: &mut Vec<ValidationError>) {
        if !self.pattern.is_match(value) {
            errors.push(ValidationError::Pattern {
                field: path.to_string(),
                actual: value.clone(),
                pattern: self.pattern.to_string(),
            });
        }
    }
}

pub struct MaxItemsValidator<T> {
    max_items: usize,
    phantom: PhantomData<T>,
}

impl<T> MaxItemsValidator<T> {
    pub fn new(max_items: usize) -> Self {
        Self {
            max_items,
            phantom: PhantomData::default(),
        }
    }
}

impl<T> Validator<Vec<T>> for MaxItemsValidator<T> {
    fn validate(&self, path: &ValidationPath, value: &Vec<T>, errors: &mut Vec<ValidationError>) {
        if value.len() > self.max_items {
            errors.push(ValidationError::MaxItems {
                field: path.to_string(),
                actual: value.len(),
                max_items: self.max_items,
            });
        }
    }
}

pub struct MinItemsValidator<T> {
    min_items: usize,
    phantom: PhantomData<T>,
}

impl<T> MinItemsValidator<T> {
    pub fn new(min_items: usize) -> Self {
        Self {
            min_items,
            phantom: PhantomData::default(),
        }
    }
}

impl<T> Validator<Vec<T>> for MinItemsValidator<T> {
    fn validate(&self, path: &ValidationPath, value: &Vec<T>, errors: &mut Vec<ValidationError>) {
        if value.len() < self.min_items {
            errors.push(ValidationError::MinItems {
                field: path.to_string(),
                actual: value.len(),
                min_items: self.min_items,
            });
        }
    }
}
