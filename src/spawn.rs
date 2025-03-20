//! # Grammar
//!
//! * `<TOKEN>*` means repeat 0-inf times separated by `TOKEN`, the last `TOKEN` is optional.
//!
//! ```txt
//! spawn       ::= spawner (parenting? entity | code_block)<';'>* ;
//! entity      ::= name? '(' component<','> ')' extension* ;
//! extension   ::= '.' (children | method_call | observe | code_block) ;
//! method_call ::= name '(' EXPR<','>* ')' ;
//! observe     ::= '(' EXPR_CLOSURE ')' ;
//! children    ::= '[' (entity | code_block)<';'>* ']' ;
//! parenting   ::= name '>' ;
//! method_call ::= '.' name '(' EXPR<','>* ')' ;
//! name        ::= IDENT ;
//! component   ::= EXPR ;
//! code_block  ::= EXPR_BLOCK ;
//! ```

use proc_macro::TokenStream;
use syn::*;
use syn::parse::*;
use syn::token::*;
use quote::*;


pub fn spawn_impl(input: TokenStream) -> TokenStream {
  if input.is_empty() { return TokenStream::new(); }

  let Spawn { spawner, children } =
    Spawn::parse.parse(input).unwrap();

  let mut result = quote! {};

  for child in children {
    match child {
      OrCode::Code(code  ) => result.extend(quote! { #code }),
      OrCode::Else(entity) => result.extend(convert(entity, true)),
    }
  }

  quote! {{
    let spawner = &mut #spawner;
    #result
  };}.into()
}


struct Spawn {
  spawner : Ident,
  children: Vec<OrCode<Entity>>,
}

impl Parse for Spawn {
  fn parse(input: ParseStream) -> Result<Self> {
    Ok(Spawn {
      spawner: input.parse()?,
      children: input
        .parse_terminated(OrCode::<Entity>::parse, Token![;])?
        .into_iter()
        .collect(),
    })
  }
}


struct Entity {
  parent    : Option<Ident>,
  name      : Option<Ident>,
  components: Vec<Expr>,
  extensions: Vec<Extension>,
}

impl Parse for Entity {
  fn parse(input: ParseStream) -> Result<Self> {
    let parent = if input.peek(Ident) && input.peek2(Token![>]) {
      let parent = Some(input.parse()?);
      input.parse::<Token![>]>()?;
      parent
    } else {
      None
    };

    let name = if input.peek(Ident) {
      Some(input.parse()?)
    } else {
      None
    };

    let components = {
      let content;
      parenthesized!(content in input);

      content
        .parse_terminated(Expr::parse, Token![,])?
        .into_iter().collect()
    };

    let extensions = {
      let mut extensions = vec![];

      while input.peek(Token![.]) {
        extensions.push(input.parse()?);
      }

      extensions
    };

    Ok(Entity {
      parent,
      name,
      components,
      extensions,
    })
  }
}


enum Extension {
  Children  (Vec<OrCode<Entity>>),
  CodeBlock (proc_macro2::Group),
  Unfinished(Token![.], Option<Ident>),
  MethodCall(Option<Ident>, Vec<Expr>),
}

impl Parse for Extension {
  fn parse(input: ParseStream) -> Result<Self> {
    let dot = input.parse::<Token![.]>()?;

    if input.peek(Ident) {
      let method = input.parse().ok();

      if input.peek(Paren) && method.is_some() {
        let content;
        parenthesized!(content in input);

        return Ok(Extension::MethodCall(
          method,
          content
            .parse_terminated(Expr::parse, Token![,])?
            .into_iter().collect()));
      }

      return Ok(Extension::Unfinished(dot, method));
    }

    if input.peek(Paren) {
      let content;
      parenthesized!(content in input);

      return Ok(Extension::MethodCall(
        None, vec![content.parse()?]))
    }

    if input.peek(Bracket) {
      let content;
      bracketed!(content in input);

      let mut children = vec![];

      for mut child in content.parse_terminated(OrCode::<Entity>::parse, Token![;])? {
        if let OrCode::Else(ref mut child) = child {
          if child.parent.is_some() {
            return Err(Error::new(
              child.parent.as_ref().unwrap().span(),
              "Only top level entity can have parent"));
          }

          child.parent = Some(Ident::new("parent", proc_macro2::Span::call_site()));
        }

        children.push(child);
      }

      return Ok(Extension::Children(children));
    }

    if input.peek(Brace) {
      return Ok(Extension::CodeBlock(input.parse()?));
    }

    return Ok(Extension::Unfinished(dot, None));
  }
}


enum OrCode<T: Parse> {
  Code(proc_macro2::Group),
  Else(T),
}

impl Parse for OrCode<Entity> {
  fn parse(input: ParseStream) -> Result<Self> {
    if input.peek(Brace) {
      Ok(OrCode::Code(input.parse()?))
    } else {
      Ok(OrCode::Else(input.parse()?))
    }
  }
}


fn convert(entity: Entity, top_level: bool) -> proc_macro2::TokenStream {
  let mut content = quote! {};

  let Entity {
    parent,
    name,
    components,
    mut extensions,
  } = entity;

  if !top_level {
    content.extend(quote! {
      let parent = this;
    });
  }

  content.extend(quote! {
    let mut entity = spawner.spawn((
      #(#components),*
    ));

    let this = entity.id();
  });

  if let Some(parent) = parent {
    content.extend(quote! {
      entity.set_parent(#parent);
    });
  }

  for ext in extensions.drain(..) {
    content.extend(match ext {
      Extension::Children(children) => {
        let mut result = quote! {};

        for child in children {
          match child {
            OrCode::Code(code  ) => result.extend(quote! { #code }),
            OrCode::Else(entity) => result.extend(convert(entity, false)),
          }
        }

        quote! {{ #result }}
      },

      Extension::CodeBlock (block           ) => quote! { { #block } },
      Extension::Unfinished(dot, name       ) => quote! { entity #dot #name },
      Extension::MethodCall(None      , args) => quote! { entity.observe(#(#args),*); },
      Extension::MethodCall(Some(name), args) => quote! { entity. #name (#(#args),*); },
    });
  }

  let naming = name.map(|n| quote! { let #n = });
  quote! { #naming { #content this }; }
}
