#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::float_cmp
)]

use std::any::Any;

use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextMode {
    SingleLine,
    Multiline,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NumberConstraints {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: f64,
}

impl Default for NumberConstraints {
    fn default() -> Self {
        Self {
            min: None,
            max: None,
            step: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ControlKind {
    Boolean,
    Text { mode: TextMode },
    Number { constraints: NumberConstraints },
    Enum { options: &'static [&'static str] },
}

impl ControlKind {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::Text { .. } => "text",
            Self::Number { .. } => "number",
            Self::Enum { .. } => "enum",
        }
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn multiline(self) -> Self {
        match self {
            Self::Text { .. } => Self::Text {
                mode: TextMode::Multiline,
            },
            _ => panic!("multiline is only valid for text controls"),
        }
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn constrained(self, constraints: NumberConstraints) -> Self {
        match self {
            Self::Number { .. } => Self::Number { constraints },
            _ => panic!("min, max, and step are only valid for numeric controls"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ControlValue {
    Boolean(bool),
    Text(String),
    Number(f64),
    Enum(String),
}

impl ControlValue {
    #[must_use]
    pub const fn kind_name(&self) -> &'static str {
        match self {
            Self::Boolean(_) => "boolean",
            Self::Text(_) => "text",
            Self::Number(_) => "number",
            Self::Enum(_) => "enum",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlDefinition {
    pub id: &'static str,
    pub label: &'static str,
    pub docs: &'static str,
    pub kind: ControlKind,
}

impl ControlDefinition {
    /// Validates metadata-level constraints before a typed field accepts a value.
    ///
    /// # Errors
    /// Returns an error when a numeric value violates configured bounds or step.
    pub fn validate(&self, value: &ControlValue) -> Result<(), ControlError> {
        let (ControlKind::Number { constraints }, ControlValue::Number(value)) = (self.kind, value)
        else {
            return Ok(());
        };
        if let Some(min) = constraints.min
            && *value < min
        {
            return Err(ControlError::BelowMinimum {
                control: self.id,
                min,
                value: *value,
            });
        }
        if let Some(max) = constraints.max
            && *value > max
        {
            return Err(ControlError::AboveMaximum {
                control: self.id,
                max,
                value: *value,
            });
        }
        let origin = constraints.min.unwrap_or(0.0);
        let quotient = (*value - origin) / constraints.step;
        let tolerance = f64::EPSILON * 16.0 * quotient.abs().max(1.0);
        if (quotient - quotient.round()).abs() > tolerance {
            return Err(ControlError::StepMismatch {
                control: self.id,
                step: constraints.step,
                value: *value,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum ControlError {
    #[error("unknown control '{0}'")]
    UnknownControl(String),
    #[error("control '{control}' expects {expected}, received {actual}")]
    TypeMismatch {
        control: &'static str,
        expected: &'static str,
        actual: &'static str,
    },
    #[error("{value} is not a valid value for numeric control '{control}'")]
    InvalidNumber { control: &'static str, value: f64 },
    #[error("{value} is below the minimum {min} for numeric control '{control}'")]
    BelowMinimum {
        control: &'static str,
        min: f64,
        value: f64,
    },
    #[error("{value} is above the maximum {max} for numeric control '{control}'")]
    AboveMaximum {
        control: &'static str,
        max: f64,
        value: f64,
    },
    #[error("{value} does not align to step {step} for numeric control '{control}'")]
    StepMismatch {
        control: &'static str,
        step: f64,
        value: f64,
    },
    #[error("'{value}' is not a valid option for control '{control}'")]
    InvalidOption {
        control: &'static str,
        value: String,
    },
}

pub trait HblankProps: Any + Send {
    fn definitions(&self) -> &'static [ControlDefinition];
    fn control_value(&self, id: &str) -> Option<ControlValue>;
    /// Replaces one control value by its stable field identifier.
    ///
    /// # Errors
    /// Returns an error when the identifier, value kind, number, or enum option is invalid.
    fn set_control(&mut self, id: &str, value: ControlValue) -> Result<(), ControlError>;
    fn clone_box(&self) -> Box<dyn HblankProps>;
    fn as_any(&self) -> &dyn Any;
}

impl Clone for Box<dyn HblankProps> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[doc(hidden)]
pub trait ControlField: Sized {
    const KIND: ControlKind;

    fn to_control_value(&self) -> ControlValue;
    fn set_control_value(
        &mut self,
        control: &'static str,
        value: ControlValue,
    ) -> Result<(), ControlError>;
}

impl ControlField for bool {
    const KIND: ControlKind = ControlKind::Boolean;

    fn to_control_value(&self) -> ControlValue {
        ControlValue::Boolean(*self)
    }

    fn set_control_value(
        &mut self,
        control: &'static str,
        value: ControlValue,
    ) -> Result<(), ControlError> {
        let ControlValue::Boolean(value) = value else {
            return Err(ControlError::TypeMismatch {
                control,
                expected: Self::KIND.name(),
                actual: value.kind_name(),
            });
        };
        *self = value;
        Ok(())
    }
}

impl ControlField for String {
    const KIND: ControlKind = ControlKind::Text {
        mode: TextMode::SingleLine,
    };

    fn to_control_value(&self) -> ControlValue {
        ControlValue::Text(self.clone())
    }

    fn set_control_value(
        &mut self,
        control: &'static str,
        value: ControlValue,
    ) -> Result<(), ControlError> {
        let ControlValue::Text(value) = value else {
            return Err(ControlError::TypeMismatch {
                control,
                expected: Self::KIND.name(),
                actual: value.kind_name(),
            });
        };
        *self = value;
        Ok(())
    }
}

macro_rules! numeric_control {
    ($($type:ty),+ $(,)?) => {
        $(
            impl ControlField for $type {
                const KIND: ControlKind = ControlKind::Number {
                    constraints: NumberConstraints {
                        min: None,
                        max: None,
                        step: 1.0,
                    },
                };

                fn to_control_value(&self) -> ControlValue {
                    ControlValue::Number(*self as f64)
                }

                fn set_control_value(
                    &mut self,
                    control: &'static str,
                    value: ControlValue,
                ) -> Result<(), ControlError> {
                    let ControlValue::Number(value) = value else {
                        return Err(ControlError::TypeMismatch {
                            control,
                            expected: Self::KIND.name(),
                            actual: value.kind_name(),
                        });
                    };
                    if !value.is_finite()
                        || value < <$type>::MIN as f64
                        || value > <$type>::MAX as f64
                        || (value as $type) as f64 != value
                    {
                        return Err(ControlError::InvalidNumber { control, value });
                    }
                    *self = value as $type;
                    Ok(())
                }
            }
        )+
    };
}

numeric_control!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

macro_rules! floating_control {
    ($($type:ty),+ $(,)?) => {
        $(
            impl ControlField for $type {
                const KIND: ControlKind = ControlKind::Number {
                    constraints: NumberConstraints {
                        min: None,
                        max: None,
                        step: 1.0,
                    },
                };

                fn to_control_value(&self) -> ControlValue {
                    ControlValue::Number(f64::from(*self))
                }

                fn set_control_value(
                    &mut self,
                    control: &'static str,
                    value: ControlValue,
                ) -> Result<(), ControlError> {
                    let ControlValue::Number(value) = value else {
                        return Err(ControlError::TypeMismatch {
                            control,
                            expected: Self::KIND.name(),
                            actual: value.kind_name(),
                        });
                    };
                    if !value.is_finite() || value < <$type>::MIN as f64 || value > <$type>::MAX as f64 {
                        return Err(ControlError::InvalidNumber { control, value });
                    }
                    *self = value as $type;
                    Ok(())
                }
            }
        )+
    };
}

floating_control!(f32, f64);

pub trait HblankEnum: Clone + Send + 'static {
    const VARIANTS: &'static [&'static str];

    fn variant_name(&self) -> &'static str;
    fn from_variant_name(value: &str) -> Option<Self>;
}

impl<T: HblankEnum> ControlField for T {
    const KIND: ControlKind = ControlKind::Enum {
        options: T::VARIANTS,
    };

    fn to_control_value(&self) -> ControlValue {
        ControlValue::Enum(self.variant_name().to_owned())
    }

    fn set_control_value(
        &mut self,
        control: &'static str,
        value: ControlValue,
    ) -> Result<(), ControlError> {
        let ControlValue::Enum(value) = value else {
            return Err(ControlError::TypeMismatch {
                control,
                expected: Self::KIND.name(),
                actual: value.kind_name(),
            });
        };
        let Some(next) = T::from_variant_name(&value) else {
            return Err(ControlError::InvalidOption { control, value });
        };
        *self = next;
        Ok(())
    }
}
