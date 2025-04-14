//! # Grammar
//!
//! * `<TOKEN>*` means repeat 0-inf times separated by `TOKEN`, the last `TOKEN` is optional.
//!
//! ```txt
//! v ::=
//!   | 'auto'
//!   | '@'
//!   | number '%'
//!   | number + 'px'
//!   | number + 'vw'
//!   | number + 'vh'
//!   | number + 'vmin'
//!   | number + 'vmax'
//!   | '{' EXPR '}' '%'
//!   | '{' EXPR '}' 'px'
//!   | '{' EXPR '}' 'vw'
//!   | '{' EXPR '}' 'vh'
//!   | '{' EXPR '}' 'vmin'
//!   | '{' EXPR '}' 'vmax'
//!   ;
//!
//! number ::= INT | FLOAT ;
//! ```
use crate::*;


#[derive(Clone)]
pub enum Value {
  Auto       (Span),
  Px         (Span, f32),
  Vw         (Span, f32),
  Vh         (Span, f32),
  VMin       (Span, f32),
  VMax       (Span, f32),
  Percent    (Span, Span, f32),
  ExprPx     (Span, proc_macro2::TokenStream),
  ExprVw     (Span, proc_macro2::TokenStream),
  ExprVh     (Span, proc_macro2::TokenStream),
  ExprVMin   (Span, proc_macro2::TokenStream),
  ExprVMax   (Span, proc_macro2::TokenStream),
  ExprPercent(Span, proc_macro2::TokenStream),
}

impl Parse for Value {
  fn parse(input: ParseStream) -> Result<Self> {
    if input.peek(Ident) {
      let ident: Ident = input.parse()?;
      return match ident.to_string().as_str() {
        "auto" => Ok(Value::Auto(ident.span())),
        _ => Err(Error::new(ident.span(), "Invalid value")),
      };
    }

    if input.peek(Token![@]) {
      let sym = input.parse::<Token![@]>()?;
      return Ok(Value::Auto(sym.span));
    }

    if input.peek(Brace) {
      let group   = input.parse::<Group>()?;
      let content = group.stream();

      if input.peek(Token![%]) {
        let sym = input.parse::<Token![%]>()?;
        return Ok(Value::ExprPercent(sym.span, content));
      }

      if input.peek(Ident) {
        let ident = input.parse::<Ident>()?;
        return match ident.to_string().as_str() {
          "px"   => Ok(Value::ExprPx     (ident.span(), content)),
          "vw"   => Ok(Value::ExprVw     (ident.span(), content)),
          "vh"   => Ok(Value::ExprVh     (ident.span(), content)),
          "vmin" => Ok(Value::ExprVMin   (ident.span(), content)),
          "vmax" => Ok(Value::ExprVMax   (ident.span(), content)),
          _      => Err(Error::new(ident.span(), "Invalid unit, expected px, vw, vh, vmin, vmax or %")),
        };
      }

      return Err(Error::new(group.span(), "Expected unit, expected px, vw, vh, vmin, vmax or %"));
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
      return Err(input.error("Expected float, int or '{'"));
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
      _ => Err(Error::new(span, "Invalid unit, expected px, vw, vh, vmin, vmax or %")),
    }
  }
}

impl Generate for Value {
  fn generate(&self) -> proc_macro2::TokenStream {
    let (value, unit) = match self {
      Value::Auto(span     ) => (None                 , Ident::new("Auto", *span)),
      Value::Px  (span, val) => (Some(quote! {(#val)}), Ident::new("Px"  , *span)),
      Value::Vw  (span, val) => (Some(quote! {(#val)}), Ident::new("Vw"  , *span)),
      Value::Vh  (span, val) => (Some(quote! {(#val)}), Ident::new("Vh"  , *span)),
      Value::VMin(span, val) => (Some(quote! {(#val)}), Ident::new("VMin", *span)),
      Value::VMax(span, val) => (Some(quote! {(#val)}), Ident::new("VMax", *span)),

      // expr
      Value::ExprPx     (span, val) => (Some(quote! {({#val})}), Ident::new("Px"     , *span)),
      Value::ExprVw     (span, val) => (Some(quote! {({#val})}), Ident::new("Vw"     , *span)),
      Value::ExprVh     (span, val) => (Some(quote! {({#val})}), Ident::new("Vh"     , *span)),
      Value::ExprVMin   (span, val) => (Some(quote! {({#val})}), Ident::new("VMin"   , *span)),
      Value::ExprVMax   (span, val) => (Some(quote! {({#val})}), Ident::new("VMax"   , *span)),
      Value::ExprPercent(span, val) => (Some(quote! {({#val})}), Ident::new("Percent", *span)),

      // special case
      Value::Percent(span1, span2, val) => {
        let i1 = Ident::new("Percent", *span1);
        let i2 = Ident::new("Percent", *span2);
        return quote! {{ bevy::ui::Val::#i1; bevy::ui::Val::#i2(#val) }};
      },
    };

    quote! { bevy::ui::Val::#unit #value }
  }

  fn generate_default() -> proc_macro2::TokenStream {
    quote! { bevy::ui::Val::default() }
  }
}

impl Value {
  pub fn span(&self) -> &Span {
    match self {
      Value::Auto       (span    ) => span,
      Value::Px         (span, _ ) => span,
      Value::Vw         (span, _ ) => span,
      Value::Vh         (span, _ ) => span,
      Value::VMin       (span, _ ) => span,
      Value::VMax       (span, _ ) => span,
      Value::Percent    (span, ..) => span,
      Value::ExprPx     (span, _ ) => span,
      Value::ExprVw     (span, _ ) => span,
      Value::ExprVh     (span, _ ) => span,
      Value::ExprVMin   (span, _ ) => span,
      Value::ExprVMax   (span, _ ) => span,
      Value::ExprPercent(span, _ ) => span,
    }
  }
}
