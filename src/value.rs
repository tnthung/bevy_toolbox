//! # Grammar
//!
//! * `<TOKEN>*` means repeat 0-inf times separated by `TOKEN`, the last `TOKEN` is optional.
//!
//! ```txt
//! v ::= val;
//!
//! val ::=
//!   | 'auto'
//!   | number '%'
//!   | number + 'px'
//!   | number + 'vw'
//!   | number + 'vh'
//!   | number + 'vmin'
//!   | number + 'vmax'
//!   ;
//!
//! number ::= INT | FLOAT ;
//! ```
use crate::*;


pub enum Value {
  Auto   (Span),
  Px     (Span, f32),
  Vw     (Span, f32),
  Vh     (Span, f32),
  VMin   (Span, f32),
  VMax   (Span, f32),
  Percent(Span, Span, f32),
}

impl Parse for Value {
  fn parse(input: ParseStream) -> Result<Self> {
    if input.peek(Ident) {
      let ident: Ident = input.parse()?;
      return match ident.to_string().as_str() {
        "auto" => Ok(Value::Auto(ident.span())),
        _ => Err(Error::new(ident.span(), "invalid value")),
      };
    }

    let (span, value, unit) = if input.peek(LitFloat) {
      let token = input.parse::<LitFloat>()?;
      let value = token.base10_parse::<f32>()?;
      let unit  = token.suffix().to_string();
      (token.span(), value, unit)
    } else if input.peek(LitInt) {
      let token = input.parse::<LitInt>()?;
      let value = token.base10_parse::<f32>()?;
      let unit  = token.suffix().to_string();
      (token.span(), value, unit)
    } else {
      return Err(input.error("expected float or int"));
    };

    if unit == "" && input.peek(Token![%]) {
      let sym = input.parse::<Token![%]>()?;
      return Ok(Value::Percent(span, sym.span, value));
    }

    match unit.as_str() {
      "px"   => Ok(Value::Px  (span, value)),
      "vw"   => Ok(Value::Vw  (span, value)),
      "vh"   => Ok(Value::Vh  (span, value)),
      "vmin" => Ok(Value::VMin(span, value)),
      "vmax" => Ok(Value::VMax(span, value)),
      _ => Err(Error::new(span, "invalid unit")),
    }
  }
}

impl Generate for Value {
  fn generate(self) -> proc_macro2::TokenStream {
    let (value, unit) = match self {
      Value::Auto(span     ) => (None     , Ident::new("Auto", span)),
      Value::Px  (span, val) => (Some(val), Ident::new("Px"  , span)),
      Value::Vw  (span, val) => (Some(val), Ident::new("Vw"  , span)),
      Value::Vh  (span, val) => (Some(val), Ident::new("Vh"  , span)),
      Value::VMin(span, val) => (Some(val), Ident::new("VMin", span)),
      Value::VMax(span, val) => (Some(val), Ident::new("VMax", span)),

      // special case
      Value::Percent(span1, span2, val) => {
        let i1 = Ident::new("Percent", span1);
        let i2 = Ident::new("Percent", span2);
        return quote! {{ bevy::ui::Val::#i1; bevy::ui::Val::#i2(#val) }};
      },
    };

    quote! { bevy::ui::Val::#unit(#value) }
  }
}
