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


pub struct Edges(Vec<ValueOrOmit>);

impl Parse for Edges {
  fn parse(input: ParseStream) -> Result<Self> {
    let mut values = Vec::new();

    while !input.is_empty() && values.len() < 4 {
      values.push(ValueOrOmit::parse(input)?);
    }

    if !input.is_empty() || values.is_empty() {
      return Err(input.error("Expected 1-4 value or '_'"));
    }

    Ok(Edges(values))
  }
}

impl Generate for Edges {
  fn generate(&self) -> proc_macro2::TokenStream {
    match self.0.as_slice() {
      // all sides
      [v1] => {
        let a = v1.generate();
        quote! { bevy::ui::UiRect { top: #a, right: #a, bottom: #a, left: #a } }
      }

      // vertical, horizontal
      [v1, v2] => {
        let v = v1.generate();
        let h = v2.generate();
        quote! { bevy::ui::UiRect { top: #v, right: #h, bottom: #v, left: #h } }
      }

      // top, horizontal, bottom
      [v1, v2, v3] => {
        let t = v1.generate();
        let h = v2.generate();
        let b = v3.generate();
        quote! { bevy::ui::UiRect { top: #t, right: #h, bottom: #b, left: #h } }
      }

      // top, right, bottom, left
      [v1, v2, v3, v4] => {
        let t = v1.generate();
        let r = v2.generate();
        let b = v3.generate();
        let l = v4.generate();
        quote! { bevy::ui::UiRect { top: #t, right: #r, bottom: #b, left: #l } }
      }

      _ => unreachable!(),
    }
  }
}
