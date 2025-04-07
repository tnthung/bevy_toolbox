//! # Grammar
//!
//! * `<TOKEN>*` means repeat 0-inf times separated by `TOKEN`, the last `TOKEN` is optional.
//!
//! ```txt
//! turns ::= val_or_omit{1,4};
//!
//! val_or_omit ::= v | '_';
//! ```
use crate::*;
use value::ValueOrOmit;


pub struct Turns {
  pub top_left    : ValueOrOmit,
  pub top_right   : ValueOrOmit,
  pub bottom_right: ValueOrOmit,
  pub bottom_left : ValueOrOmit,
}

impl Parse for Turns {
  fn parse(input: ParseStream) -> Result<Self> {
    let mut values = Vec::new();

    while !input.is_empty() && values.len() < 4 {
      values.push(ValueOrOmit::parse(input)?);
    }

    if !input.is_empty() || values.is_empty() {
      return Err(input.error("Expected 1-4 value or '_'"));
    }

    match values.as_slice() {
      [v1] => {
        Ok(Turns {
          top_left    : v1.clone(),
          top_right   : v1.clone(),
          bottom_right: v1.clone(),
          bottom_left : v1.clone(),
        })
      }

      [v1, v2] => {
        Ok(Turns {
          top_left    : v1.clone(),
          top_right   : v1.clone(),
          bottom_right: v2.clone(),
          bottom_left : v2.clone(),
        })
      }

      [v1, v2, v3] => {
        Ok(Turns {
          top_left    : v1.clone(),
          top_right   : v2.clone(),
          bottom_right: v3.clone(),
          bottom_left : v3.clone(),
        })
      }

      [v1, v2, v3, v4] => {
        Ok(Turns {
          top_left    : v1.clone(),
          top_right   : v2.clone(),
          bottom_right: v3.clone(),
          bottom_left : v4.clone(),
        })
      }

      _ => unreachable!(),
    }
  }
}
