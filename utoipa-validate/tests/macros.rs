use utoipa::ToSchema;
use utoipa_validate::{Validatable, ValidationError, ValidationErrorCategory};

#[derive(ToSchema, Validatable)]
struct IntegerFields {
    #[schema(minimum = - 1, maximum = 3)]
    pub signed8: i8,
    #[schema(minimum = - 2, maximum = 12)]
    pub signed16: i16,
    #[schema(minimum = - 3, maximum = 14)]
    pub signed32: i32,
    #[schema(exclusive_minimum = - 4, exclusive_maximum = 8)]
    pub signed64: i64,
    #[schema(minimum = 1, maximum = 6)]
    pub unsigned8: u8,
    #[schema(minimum = 2, maximum = 16)]
    pub unsigned16: u16,
    #[schema(multiple_of = 2)]
    pub unsigned32: u32,
    #[schema(exclusive_minimum = 4, exclusive_maximum = 6)]
    pub unsigned64: u64,
}

#[test]
fn valid_integer_fields() {
    let result = IntegerFields {
        signed8: -1,
        signed16: -2,
        signed32: -3,
        signed64: -3,
        unsigned8: 1,
        unsigned16: 2,
        unsigned32: 2,
        unsigned64: 5,
    }
    .validate();

    assert!(result.is_ok());

    let result = IntegerFields {
        signed8: 3,
        signed16: 12,
        signed32: 14,
        signed64: 7,
        unsigned8: 3,
        unsigned16: 12,
        unsigned32: 0,
        unsigned64: 5,
    }
    .validate();

    assert!(result.is_ok());
}

#[test]
fn invalid_integer_fields() {
    let result = IntegerFields {
        signed8: -2,
        signed16: -3,
        signed32: -4,
        signed64: -4,
        unsigned8: 0,
        unsigned16: 1,
        unsigned32: 1,
        unsigned64: 4,
    }
    .validate();

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.len(), 8);
    assert_eq!(
        error[0],
        ValidationError {
            category: ValidationErrorCategory::Minimum,
            path: "signed8".to_owned(),
            actual: "-2".to_owned(),
            expected: "-1".to_owned(),
        }
    );
    assert_eq!(
        error[1],
        ValidationError {
            category: ValidationErrorCategory::Minimum,
            path: "signed16".to_owned(),
            actual: "-3".to_owned(),
            expected: "-2".to_owned(),
        }
    );
    assert_eq!(
        error[2],
        ValidationError {
            category: ValidationErrorCategory::Minimum,
            path: "signed32".to_owned(),
            actual: "-4".to_owned(),
            expected: "-3".to_owned(),
        }
    );
    assert_eq!(
        error[3],
        ValidationError {
            category: ValidationErrorCategory::ExclusiveMinimum,
            path: "signed64".to_owned(),
            actual: "-4".to_owned(),
            expected: "-4".to_owned(),
        }
    );
    assert_eq!(
        error[4],
        ValidationError {
            category: ValidationErrorCategory::Minimum,
            path: "unsigned8".to_owned(),
            actual: "0".to_owned(),
            expected: "1".to_owned(),
        }
    );
    assert_eq!(
        error[5],
        ValidationError {
            category: ValidationErrorCategory::Minimum,
            path: "unsigned16".to_owned(),
            actual: "1".to_owned(),
            expected: "2".to_owned(),
        }
    );
    assert_eq!(
        error[6],
        ValidationError {
            category: ValidationErrorCategory::MultipleOf,
            path: "unsigned32".to_owned(),
            actual: "1".to_owned(),
            expected: "2".to_owned(),
        }
    );
    assert_eq!(
        error[7],
        ValidationError {
            category: ValidationErrorCategory::ExclusiveMinimum,
            path: "unsigned64".to_owned(),
            actual: "4".to_owned(),
            expected: "4".to_owned(),
        }
    );

    let result = IntegerFields {
        signed8: 4,
        signed16: 13,
        signed32: 15,
        signed64: 8,
        unsigned8: 7,
        unsigned16: 17,
        unsigned32: 3,
        unsigned64: 6,
    }
    .validate();

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.len(), 8);
    assert_eq!(
        error[0],
        ValidationError {
            category: ValidationErrorCategory::Maximum,
            path: "signed8".to_owned(),
            actual: "4".to_owned(),
            expected: "3".to_owned(),
        }
    );
    assert_eq!(
        error[1],
        ValidationError {
            category: ValidationErrorCategory::Maximum,
            path: "signed16".to_owned(),
            actual: "13".to_owned(),
            expected: "12".to_owned(),
        }
    );
    assert_eq!(
        error[2],
        ValidationError {
            category: ValidationErrorCategory::Maximum,
            path: "signed32".to_owned(),
            actual: "15".to_owned(),
            expected: "14".to_owned(),
        }
    );
    assert_eq!(
        error[3],
        ValidationError {
            category: ValidationErrorCategory::ExclusiveMaximum,
            path: "signed64".to_owned(),
            actual: "8".to_owned(),
            expected: "8".to_owned(),
        }
    );
    assert_eq!(
        error[4],
        ValidationError {
            category: ValidationErrorCategory::Maximum,
            path: "unsigned8".to_owned(),
            actual: "7".to_owned(),
            expected: "6".to_owned(),
        }
    );
    assert_eq!(
        error[5],
        ValidationError {
            category: ValidationErrorCategory::Maximum,
            path: "unsigned16".to_owned(),
            actual: "17".to_owned(),
            expected: "16".to_owned(),
        }
    );
    assert_eq!(
        error[6],
        ValidationError {
            category: ValidationErrorCategory::MultipleOf,
            path: "unsigned32".to_owned(),
            actual: "3".to_owned(),
            expected: "2".to_owned(),
        }
    );
    assert_eq!(
        error[7],
        ValidationError {
            category: ValidationErrorCategory::ExclusiveMaximum,
            path: "unsigned64".to_owned(),
            actual: "6".to_owned(),
            expected: "6".to_owned(),
        }
    );
}

#[derive(ToSchema, Validatable)]
struct FloatFields {
    #[schema(minimum = 1.0, exclusive_maximum = 3.0)]
    pub float32: f32,
    #[schema(exclusive_minimum = - 0.5, maximum = 0.0)]
    pub float64: f64,
}

#[test]
fn valid_float_fields() {
    let result = FloatFields {
        float32: 1.0,
        float64: -0.49,
    }
    .validate();

    assert!(result.is_ok());

    let result = FloatFields {
        float32: 2.9,
        float64: 0.0,
    }
    .validate();

    assert!(result.is_ok());
}

#[test]
fn invalid_float_fields() {
    let result = FloatFields {
        float32: 0.9,
        float64: -0.5,
    }
    .validate();

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.len(), 2);
    assert_eq!(
        error[0],
        ValidationError {
            category: ValidationErrorCategory::Minimum,
            path: "float32".to_owned(),
            actual: "0.9".to_owned(),
            expected: "1".to_owned(),
        }
    );
    assert_eq!(
        error[1],
        ValidationError {
            category: ValidationErrorCategory::ExclusiveMinimum,
            path: "float64".to_owned(),
            actual: "-0.5".to_owned(),
            expected: "-0.5".to_owned(),
        }
    );

    let result = FloatFields {
        float32: 3.0,
        float64: 0.1,
    }
    .validate();

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.len(), 2);
    assert_eq!(
        error[0],
        ValidationError {
            category: ValidationErrorCategory::ExclusiveMaximum,
            path: "float32".to_owned(),
            actual: "3".to_owned(),
            expected: "3".to_owned(),
        }
    );
    assert_eq!(
        error[1],
        ValidationError {
            category: ValidationErrorCategory::Maximum,
            path: "float64".to_owned(),
            actual: "0.1".to_owned(),
            expected: "0".to_owned(),
        }
    );
}

#[derive(ToSchema, Validatable)]
struct StringFields {
    #[schema(min_length = 1, max_length = 5)]
    pub s: String,
    #[schema(pattern = "^[0-9a-f]+$")]
    pub hex: String,
}

#[test]
fn valid_string_fields() {
    let result = StringFields {
        s: "1".to_owned(),
        hex: "deef".to_owned(),
    }
    .validate();

    assert!(result.is_ok());

    let result = StringFields {
        s: "12345".to_owned(),
        hex: "bee138".to_owned(),
    }
    .validate();

    assert!(result.is_ok());
}

#[test]
fn invalid_string_fields() {
    let result = StringFields {
        s: "".to_owned(),
        hex: "".to_owned(),
    }
    .validate();

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.len(), 2);
    assert_eq!(
        error[0],
        ValidationError {
            category: ValidationErrorCategory::MinLength,
            path: "s".to_owned(),
            actual: "0".to_owned(),
            expected: "1".to_owned(),
        }
    );
    assert_eq!(
        error[1],
        ValidationError {
            category: ValidationErrorCategory::Pattern,
            path: "hex".to_owned(),
            actual: "".to_owned(),
            expected: "^[0-9a-f]+$".to_owned(),
        }
    );

    let result = StringFields {
        s: "123456".to_owned(),
        hex: "abz".to_owned(),
    }
    .validate();

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.len(), 2);
    assert_eq!(
        error[0],
        ValidationError {
            category: ValidationErrorCategory::MaxLength,
            path: "s".to_owned(),
            actual: "6".to_owned(),
            expected: "5".to_owned(),
        }
    );
    assert_eq!(
        error[1],
        ValidationError {
            category: ValidationErrorCategory::Pattern,
            path: "hex".to_owned(),
            actual: "abz".to_owned(),
            expected: "^[0-9a-f]+$".to_owned(),
        }
    );
}

#[derive(ToSchema, Validatable)]
struct UnnamedOption(#[schema(minimum = 3)] Option<i32>);

#[test]
fn valid_unnamed_option() {
    let result = UnnamedOption(None).validate();

    assert!(result.is_ok());

    let result = UnnamedOption(Some(4)).validate();

    assert!(result.is_ok());
}

#[test]
fn invalid_unnamed_option() {
    let result = UnnamedOption(Some(2)).validate();

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.len(), 1);
    assert_eq!(
        error[0],
        ValidationError {
            category: ValidationErrorCategory::Minimum,
            path: "0".to_owned(),
            actual: "2".to_owned(),
            expected: "3".to_owned(),
        }
    );
}

#[derive(ToSchema, Validatable)]
struct UnnamedVec(#[schema(min_items = 1, max_items = 3)] Vec<i32>);

#[test]
fn valid_unnamed_vec() {
    let result = UnnamedVec(vec![1]).validate();

    assert!(result.is_ok());

    let result = UnnamedVec(vec![1, 2, 3]).validate();

    assert!(result.is_ok());
}

#[test]
fn invalid_unnamed_vec() {
    let result = UnnamedVec(vec![]).validate();

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.len(), 1);
    assert_eq!(
        error[0],
        ValidationError {
            category: ValidationErrorCategory::MinItems,
            path: "0".to_owned(),
            actual: "0".to_owned(),
            expected: "1".to_owned(),
        }
    );

    let result = UnnamedVec(vec![1, 2, 3, 4]).validate();

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.len(), 1);
    assert_eq!(
        error[0],
        ValidationError {
            category: ValidationErrorCategory::MaxItems,
            path: "0".to_owned(),
            actual: "4".to_owned(),
            expected: "3".to_owned(),
        }
    );
}

#[derive(ToSchema, Validatable)]
enum Enum {
    A {
        #[schema(minimum = 1)]
        one: i32,
    },
    B(#[schema(minimum = 1)] i32),
    C,
}

#[test]
fn valid_enum() {
    let result = Enum::A { one: 1 }.validate();

    assert!(result.is_ok());

    let result = Enum::B(1).validate();

    assert!(result.is_ok());

    let result = Enum::C.validate();

    assert!(result.is_ok());
}

#[test]
fn invalid_enum() {
    let result = Enum::A { one: 0 }.validate();

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.len(), 1);
    assert_eq!(
        error[0],
        ValidationError {
            category: ValidationErrorCategory::Minimum,
            path: "A.one".to_owned(),
            actual: "0".to_owned(),
            expected: "1".to_owned(),
        }
    );

    let result = Enum::B(0).validate();

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.len(), 1);
    assert_eq!(
        error[0],
        ValidationError {
            category: ValidationErrorCategory::Minimum,
            path: "B._0".to_owned(),
            actual: "0".to_owned(),
            expected: "1".to_owned(),
        }
    );
}

#[derive(ToSchema, Validatable)]
struct Nested {
    o: UnnamedOption,
}

#[test]
fn valid_nested() {
    let result = Nested {
        o: UnnamedOption(Some(4)),
    }
    .validate();

    assert!(result.is_ok());
}

#[test]
fn invalid_nested() {
    let result = Nested {
        o: UnnamedOption(Some(2)),
    }
    .validate();

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.len(), 1);
    assert_eq!(
        error[0],
        ValidationError {
            category: ValidationErrorCategory::Minimum,
            path: "o.0".to_owned(),
            actual: "2".to_owned(),
            expected: "3".to_owned(),
        }
    );
}
