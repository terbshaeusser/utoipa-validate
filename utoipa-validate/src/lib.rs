use regex::Regex;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::ops::Rem;

pub use utoipa_validate_gen::*;

/// Path to a value that is validated.
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

/// Category for validation errors that can be used to differentiate between different errors
/// independent of the error message.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ValidationErrorCategory {
    ExclusiveMaximum,
    ExclusiveMinimum,
    Maximum,
    Minimum,
    MaxItems,
    MinItems,
    MaxLength,
    MinLength,
    MultipleOf,
    Pattern,
    Other {
        /// Tag that can be used to identify the error category.
        tag: &'static str,
        /// Display function to get the error string.
        display: fn(error: &ValidationError, f: &mut Formatter<'_>) -> std::fmt::Result,
    },
}

/// Struct describing an error during validation.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ValidationError {
    /// Category of the error.
    pub category: ValidationErrorCategory,
    /// Path to the value that caused the error.
    pub path: String,
    /// The actual value.
    pub actual: String,
    /// The expected value. The meaning of this value depends on the category.
    pub expected: String,
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.category {
            ValidationErrorCategory::ExclusiveMaximum => write!(
                f,
                "{}: Must be less than {} but is {}",
                self.path, self.expected, self.actual
            ),
            ValidationErrorCategory::ExclusiveMinimum => write!(
                f,
                "{}: Must be greater than {} but is {}",
                self.path, self.expected, self.actual
            ),
            ValidationErrorCategory::Maximum => write!(
                f,
                "{}: Must be less than or equal to {} but is {}",
                self.path, self.expected, self.actual
            ),
            ValidationErrorCategory::Minimum => write!(
                f,
                "{}: Must be greater than or equal to {} but is {}",
                self.path, self.expected, self.actual
            ),
            ValidationErrorCategory::MaxItems => write!(
                f,
                "{}: Must have at most {} items but has {}",
                self.path, self.expected, self.actual
            ),
            ValidationErrorCategory::MinItems => write!(
                f,
                "{}: Must have at least {} items but has {}",
                self.path, self.expected, self.actual
            ),
            ValidationErrorCategory::MaxLength => write!(
                f,
                "{}: Must have at most {} characters but has {}",
                self.path, self.expected, self.actual
            ),
            ValidationErrorCategory::MinLength => write!(
                f,
                "{}: Must have at least {} characters but has {}",
                self.path, self.expected, self.actual
            ),
            ValidationErrorCategory::MultipleOf => write!(
                f,
                "{}: Must be a multiple of {} but is {}",
                self.path, self.expected, self.actual
            ),
            ValidationErrorCategory::Pattern => write!(
                f,
                "{}: Must match the regular expression {} but is {}",
                self.path, self.expected, self.actual
            ),
            ValidationErrorCategory::Other { tag, display } => {
                let _ = tag;

                display(self, f)
            }
        }
    }
}

/// A validator for type T.
pub trait Validator<T> {
    /// Validate the passed value stored at the passed path. Errors are added to the errors vector.
    fn validate(&self, path: &ValidationPath, value: &T, errors: &mut Vec<ValidationError>);
}

pub trait Validatable: Sized {
    /// Default validator for values of this type.
    type DefaultValidator: Validator<Self> + Default;

    /// Validate this value using the default validator.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        self.validate_with(&Self::DefaultValidator::default())
    }

    /// Validate this instance with the given validator.
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

    /// Similar to validate() except that errors are returned in the passed vector.
    fn validate_ex(&self, path: &ValidationPath, errors: &mut Vec<ValidationError>) {
        Self::DefaultValidator::default().validate(path, self, errors);
    }
}

/// A validator that is never returning errors.
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

/// A validator for Option. Implements the validator trait with a custom and the default validator
/// for the inner type.
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

/// A validator for vectors that iterates over the items. Implements the validator trait with a
/// custom and the default validator for the item type.
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

/// Validator for the 'exclusive_maximum' schema check.
pub struct ExclusiveMaximumValidator<T: PartialOrd + Display> {
    exclusive_maximum: T,
}

impl<T> ExclusiveMaximumValidator<T>
where
    T: PartialOrd + Display,
{
    pub fn new(exclusive_maximum: T) -> Self {
        Self { exclusive_maximum }
    }
}

impl<T> Validator<T> for ExclusiveMaximumValidator<T>
where
    T: PartialOrd + Display,
{
    fn validate(&self, path: &ValidationPath, value: &T, errors: &mut Vec<ValidationError>) {
        if *value >= self.exclusive_maximum {
            errors.push(ValidationError {
                category: ValidationErrorCategory::ExclusiveMaximum,
                path: path.to_string(),
                actual: value.to_string(),
                expected: self.exclusive_maximum.to_string(),
            });
        }
    }
}

/// Validator for the 'exclusive_minimum' schema check.
pub struct ExclusiveMinimumValidator<T: PartialOrd + Display> {
    exclusive_minimum: T,
}

impl<T> ExclusiveMinimumValidator<T>
where
    T: PartialOrd + Display,
{
    pub fn new(exclusive_minimum: T) -> Self {
        Self { exclusive_minimum }
    }
}

impl<T> Validator<T> for ExclusiveMinimumValidator<T>
where
    T: PartialOrd + Display,
{
    fn validate(&self, path: &ValidationPath, value: &T, errors: &mut Vec<ValidationError>) {
        if *value <= self.exclusive_minimum {
            errors.push(ValidationError {
                category: ValidationErrorCategory::ExclusiveMinimum,
                path: path.to_string(),
                actual: value.to_string(),
                expected: self.exclusive_minimum.to_string(),
            });
        }
    }
}

/// Validator for the 'maximum' schema check.
pub struct MaximumValidator<T: PartialOrd + Display> {
    maximum: T,
}

impl<T> MaximumValidator<T>
where
    T: PartialOrd + Display,
{
    pub fn new(maximum: T) -> Self {
        Self { maximum }
    }
}

impl<T> Validator<T> for MaximumValidator<T>
where
    T: PartialOrd + Display,
{
    fn validate(&self, path: &ValidationPath, value: &T, errors: &mut Vec<ValidationError>) {
        if *value > self.maximum {
            errors.push(ValidationError {
                category: ValidationErrorCategory::Maximum,
                path: path.to_string(),
                actual: value.to_string(),
                expected: self.maximum.to_string(),
            });
        }
    }
}

/// Validator for the 'minimum' schema check.
pub struct MinimumValidator<T: PartialOrd + Display> {
    minimum: T,
}

impl<T> MinimumValidator<T>
where
    T: PartialOrd + Display,
{
    pub fn new(minimum: T) -> Self {
        Self { minimum }
    }
}

impl<T> Validator<T> for MinimumValidator<T>
where
    T: PartialOrd + Display,
{
    fn validate(&self, path: &ValidationPath, value: &T, errors: &mut Vec<ValidationError>) {
        if *value < self.minimum {
            errors.push(ValidationError {
                category: ValidationErrorCategory::Minimum,
                path: path.to_string(),
                actual: value.to_string(),
                expected: self.minimum.to_string(),
            });
        }
    }
}

/// Validator for the 'max_length' schema check.
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
            errors.push(ValidationError {
                category: ValidationErrorCategory::MaxLength,
                path: path.to_string(),
                actual: value.len().to_string(),
                expected: self.max_length.to_string(),
            });
        }
    }
}

/// Validator for the 'min_length' schema check.
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
            errors.push(ValidationError {
                category: ValidationErrorCategory::MinLength,
                path: path.to_string(),
                actual: value.len().to_string(),
                expected: self.min_length.to_string(),
            });
        }
    }
}

/// Validator for the 'pattern' schema check.
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
            errors.push(ValidationError {
                category: ValidationErrorCategory::Pattern,
                path: path.to_string(),
                actual: value.to_string(),
                expected: self.pattern.to_string(),
            });
        }
    }
}

/// Validator for the 'max_items' schema check.
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
            errors.push(ValidationError {
                category: ValidationErrorCategory::MaxItems,
                path: path.to_string(),
                actual: value.len().to_string(),
                expected: self.max_items.to_string(),
            });
        }
    }
}

/// Validator for the 'min_items' schema check.
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
            errors.push(ValidationError {
                category: ValidationErrorCategory::MinItems,
                path: path.to_string(),
                actual: value.len().to_string(),
                expected: self.min_items.to_string(),
            });
        }
    }
}

/// Validator for the 'multiple_of' schema check.
pub struct MultipleOfValidator<T>
where
    T: Rem<T, Output = T> + PartialEq + Default + Copy + Display,
{
    multiple_of: T,
    phantom: PhantomData<T>,
}

impl<T> MultipleOfValidator<T>
where
    T: Rem<T, Output = T> + PartialEq + Default + Copy + Display,
{
    pub fn new(multiple_of: T) -> Self {
        Self {
            multiple_of,
            phantom: PhantomData::default(),
        }
    }
}

impl<T> Validator<T> for MultipleOfValidator<T>
where
    T: Rem<T, Output = T> + PartialEq + Default + Copy + Display,
{
    fn validate(&self, path: &ValidationPath, value: &T, errors: &mut Vec<ValidationError>) {
        if *value % self.multiple_of != T::default() {
            errors.push(ValidationError {
                category: ValidationErrorCategory::MultipleOf,
                path: path.to_string(),
                actual: value.to_string(),
                expected: self.multiple_of.to_string(),
            });
        }
    }
}
