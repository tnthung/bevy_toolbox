//! # Grammar
//!
//! * `<TOKEN>*` means repeat 0-inf times separated by `TOKEN`, the last `TOKEN` is optional.
//!
//! ```txt
//! e ::= val_or_omit{1,4};
//!
//! val_or_omit ::= v | '_';
//! ```
use crate::*;
use value::ValueOrOmit;


pub struct Edges {
  pub top   : ValueOrOmit,
  pub right : ValueOrOmit,
  pub bottom: ValueOrOmit,
  pub left  : ValueOrOmit,
}

impl Parse for Edges {
  fn parse(input: ParseStream) -> Result<Self> {
    let mut values = Vec::new();

    while !input.is_empty() && values.len() < 4 {
      values.push(ValueOrOmit::parse(input)?);
    }

    if !input.is_empty() || values.is_empty() {
      return Err(input.error("Expected 1-4 value or '_'"));
    }

    match values.as_slice() {
      [v1] => Ok(Edges {
        top   : v1.clone(),
        right : v1.clone(),
        bottom: v1.clone(),
        left  : v1.clone(),
      }),

      [v1, v2] => return Ok(Edges {
        top   : v1.clone(),
        right : v2.clone(),
        bottom: v1.clone(),
        left  : v2.clone(),
      }),

      [v1, v2, v3] => return Ok(Edges {
        top   : v1.clone(),
        right : v2.clone(),
        bottom: v3.clone(),
        left  : v2.clone(),
      }),

      [v1, v2, v3, v4] =>  return Ok(Edges {
        top   : v1.clone(),
        right : v2.clone(),
        bottom: v3.clone(),
        left  : v4.clone(),
      }),

      _ => unreachable!()
    }
  }
}

impl Generate for Edges {
  fn generate(&self) -> proc_macro2::TokenStream {
    let top    = self.top   .generate();
    let right  = self.right .generate();
    let bottom = self.bottom.generate();
    let left   = self.left  .generate();

    quote! {
      bevy::ui::UiRect {
        top:    #top,
        right:  #right,
        bottom: #bottom,
        left:   #left,
      }
    }
  }
}
