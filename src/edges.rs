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
use value::*;


pub struct Edges {
  pub top   : MightOmit<Value>,
  pub right : MightOmit<Value>,
  pub bottom: MightOmit<Value>,
  pub left  : MightOmit<Value>,
}

impl Parse for Edges {
  fn parse(input: ParseStream) -> Result<Self> {
    let mut values = Vec::new();

    while !input.is_empty() && values.len() < 4 {
      values.push(MightOmit::<Value>::parse(input)?);
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

    let mut result = quote! {};

    {
      let span = match &self.top {
        MightOmit::Omit (s) => s,
        MightOmit::Value(t) => t.span(),
      };

      result.extend(quote! { struct });
      result.extend(quote_spanned! {*span=> Top });
      result.extend(quote! { ; });
    }

    {
      let span = match &self.right {
        MightOmit::Omit (s) => s,
        MightOmit::Value(t) => t.span(),
      };

      result.extend(quote! { struct });
      result.extend(quote_spanned! {*span=> Right });
      result.extend(quote! { ; });
    }

    {
      let span = match &self.bottom {
        MightOmit::Omit (s) => s,
        MightOmit::Value(t) => t.span(),
      };

      result.extend(quote! { struct });
      result.extend(quote_spanned! {*span=> Bottom });
      result.extend(quote! { ; });
    }
    {
      let span = match &self.left {
        MightOmit::Omit (s) => s,
        MightOmit::Value(t) => t.span(),
      };

      result.extend(quote! { struct });
      result.extend(quote_spanned! {*span=> Left });
      result.extend(quote! { ; });
    }

    result.extend(quote! {
      bevy::ui::UiRect {
        top:    #top,
        right:  #right,
        bottom: #bottom,
        left:   #left,
      }
    });

    quote! {{ #result }}
  }
}
