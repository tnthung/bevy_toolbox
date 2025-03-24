//! # Grammar
//!
//! * `<TOKEN>*` means repeat 0-inf times separated by `TOKEN`, the last `TOKEN` is optional.
//!
//! ```txt
//! spawn       ::= spawner top_level<';'>* ;
//!
//! definition  ::= '(' component<','>* ')' ('.' extension)* ('.' children)* ;
//! entity      ::= name? definition ;
//!
//! parented    ::= name '>' entity ;
//! inserted    ::= name '+' definition ;
//!
//! child       ::= entity | inserted | code_block ;
//! top_level   ::= entity | inserted | code_block | parented ;
//!
//! extension   ::= observe | method_call | code_block ;
//! observe     ::= '(' argument ')' ;
//! children    ::= '[' child<';'>* ']' ;
//! method_call ::= name '(' argument<','>* ')' ;
//!
//! name        ::= IDENT ;
//! spawner     ::= IDENT ;
//! argument    ::= EXPR ;
//! component   ::= EXPR ;
//! code_block  ::= EXPR_BLOCK ;
//! ```

use crate::Generate;
use proc_macro::TokenStream;
use proc_macro2::Group;
use proc_macro2::Span;
use syn::*;
use syn::parse::*;
use syn::token::*;
use quote::*;


pub fn spawn_impl(input: TokenStream) -> TokenStream {
  if input.is_empty() { return TokenStream::new(); }
  Spawn::parse.parse(input).unwrap().generate().into()
}


struct Spawn {
  spawner  : Ident,
  top_level: Vec<TopLevel>,
}

impl Parse for Spawn {
  fn parse(input: ParseStream) -> Result<Self> {
    Ok(Spawn {
      spawner : input.parse()?,
      top_level: input
        .parse_terminated(TopLevel::parse, Token![;])?
        .into_iter()
        .collect(),
    })
  }
}

impl Generate for Spawn {
  fn generate(self) -> proc_macro2::TokenStream {
    let Spawn { spawner, top_level } = self;

    let mut content = quote! {
      let spawner = &mut #spawner;
    };

    for e in top_level {
      content.extend(e.generate());
    }

    quote! { { #content }; }
  }
}


struct Definition {
  components: Vec<Expr>,
  extensions: Vec<Extension>,
  children  : Vec<Children>,
}

impl Parse for Definition {
  fn parse(input: ParseStream) -> Result<Self> {
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
        if input.peek2(Bracket) {
          break;
        }

        extensions.push(input.parse()?);
      }

      extensions
    };

    let children = {
      let mut children = vec![];

      while input.peek(Token![.]) {
        if !input.peek2(Bracket) {
          return Err(input.error("Extensions cannot be chained after children group"));
        }

        input.parse::<Token![.]>()?;
        children.push(input.parse()?);
      }

      children
    };

    Ok(Definition {
      components,
      extensions,
      children,
    })
  }
}


struct Entity {
  name      : Option<Ident>,
  definition: Definition,
}

impl Parse for Entity {
  fn parse(input: ParseStream) -> Result<Self> {
    let name = if input.peek(Ident) {
      Some(input.parse()?)
    } else {
      None
    };

    Ok(Entity {
      name,
      definition: input.parse()?,
    })
  }
}

impl Generate for Entity {
  fn generate(self) -> proc_macro2::TokenStream {
    let Entity     { name, definition } = self;
    let Definition { components, extensions, children } = definition;

    let mut content = quote! {
      let mut entity = spawner.spawn((
        #(#components),*
      ));

      let this = entity.id();
    };

    for ext in extensions {
      content.extend(match ext {
        Extension::MethodCall(method   ) => method.generate(),
        Extension::Observe   (arg      ) => quote! { entity.observe(#arg); },
        Extension::CodeBlock (block    ) => quote! { #block },
        Extension::Unfinished(dot, name) => quote! { entity #dot #name },
      });
    }

    for group in children {
      content.extend(group.generate());
    }

    let naming = name.map(|n| quote! { let #n = });
    quote! { #naming { #content this }; }
  }
}


struct Parented {
  parent: Ident,
  entity: Entity,
}

impl Parse for Parented {
  fn parse(input: ParseStream) -> Result<Self> {
    let parent = input.parse()?;
    input.parse::<Token![>]>()?;

    Ok(Parented {
      parent,
      entity: input.parse()?,
    })
  }
}

impl Generate for Parented {
  fn generate(self) -> proc_macro2::TokenStream {
    let Parented   { parent, entity } = self;
    let Entity     { name, definition } = entity;
    let Definition { components, extensions, children } = definition;

    let mut content = quote! {
      let mut entity = spawner.spawn((
        #(#components),*
      ));

      let this = entity.id();
      entity.set_parent(#parent);
    };

    for ext in extensions {
      content.extend(match ext {
        Extension::MethodCall(method   ) => method.generate(),
        Extension::Observe   (arg      ) => quote! { entity.observe(#arg); },
        Extension::CodeBlock (block    ) => quote! { #block },
        Extension::Unfinished(dot, name) => quote! { entity #dot #name },
      });
    }

    for group in children {
      content.extend(group.generate());
    }

    let naming = name.map(|n| quote! { let #n = });
    quote! { #naming { #content this }; }
  }
}


struct Inserted {
  base  : Ident,
  entity: Definition,
}

impl Parse for Inserted {
  fn parse(input: ParseStream) -> Result<Self> {
    let base = input.parse()?;
    input.parse::<Token![+]>()?;

    Ok(Inserted {
      base,
      entity: input.parse()?,
    })
  }
}

impl Generate for Inserted {
  fn generate(self) -> proc_macro2::TokenStream {
    let Inserted   { base, entity } = self;
    let Definition { components, extensions, children } = entity;

    let mut content = quote! {
      let mut entity = spawner.entity(#base);
      let mut entity = entity.insert((
        #(#components),*
      ));

      let this = entity.id();
    };

    for ext in extensions {
      content.extend(match ext {
        Extension::MethodCall(method   ) => method.generate(),
        Extension::Observe   (arg      ) => quote! { entity.observe(#arg); },
        Extension::CodeBlock (block    ) => quote! { #block },
        Extension::Unfinished(dot, name) => quote! { entity #dot #name },
      });
    }

    for group in children {
      content.extend(group.generate());
    }

    quote! { { #content }; }
  }
}


enum Child {
  Entity   (Entity),
  Inserted (Inserted),
  CodeBlock(Group),
}

impl Parse for Child {
  fn parse(input: ParseStream) -> Result<Self> {
    if input.peek(Ident) && input.peek2(Token![+]) {
      return Ok(Child::Inserted(input.parse()?))
    }

    if input.peek(Ident) || input.peek(Paren) {
      return Ok(Child::Entity(input.parse()?))
    }

    if input.peek(Brace) {
      return Ok(Child::CodeBlock(input.parse()?))
    }

    Err(input.error("Expected entity, inserted or code block"))
  }
}


enum TopLevel {
  Entity   (Entity),
  Parented (Parented),
  Inserted (Inserted),
  CodeBlock(Group),
}

impl Parse for TopLevel {
  fn parse(input: ParseStream) -> Result<Self> {
    if input.peek(Ident) && input.peek2(Token![>]) {
      return Ok(TopLevel::Parented(input.parse()?))
    }

    if input.peek(Ident) && input.peek2(Token![+]) {
      return Ok(TopLevel::Inserted(input.parse()?))
    }

    if input.peek(Ident) || input.peek(Paren) {
      return Ok(TopLevel::Entity(input.parse()?))
    }

    if input.peek(Brace) {
      return Ok(TopLevel::CodeBlock(input.parse()?))
    }

    Err(input.error("Expected parented, inserted or code block"))
  }
}

impl Generate for TopLevel {
  fn generate(self) -> proc_macro2::TokenStream {
    match self {
      TopLevel::Entity   (entity  ) => entity  .generate(),
      TopLevel::Parented (parented) => parented.generate(),
      TopLevel::Inserted (inserted) => inserted.generate(),
      TopLevel::CodeBlock(block   ) => quote! { #block },
    }
  }
}


enum Extension {
  Observe   (Expr),
  MethodCall(MethodCall),
  CodeBlock (Group),

  /// Unfinished is not a valid part of the grammar, it is used to allow the text editor correctly
  /// shows the autocomplete suggestions.
  Unfinished(Token![.], Option<Ident>),
}

impl Parse for Extension {
  fn parse(input: ParseStream) -> Result<Self> {
    let dot = input.parse::<Token![.]>()?;

    if input.peek(Ident) {
      if input.peek2(Paren) {
        return Ok(Extension::MethodCall(input.parse()?));
      }

      return Ok(Extension::Unfinished(dot, Some(input.parse()?)));
    }

    if input.peek(Paren) {
      return Ok(Extension::Observe({
        let content;
        parenthesized!(content in input);
        content.parse()?
      }));
    }

    if input.peek(Brace) {
      return Ok(Extension::CodeBlock(input.parse()?));
    }

    return Ok(Extension::Unfinished(dot, None));
  }
}


struct Children(Vec<Child>);

impl Parse for Children {
  fn parse(input: ParseStream) -> Result<Self> {
    let content;
    bracketed!(content in input);

    Ok(Children(content.parse_terminated(Child::parse, Token![;])?.into_iter().collect()))
  }
}

impl Generate for Children {
  fn generate(self) -> proc_macro2::TokenStream {
    let Children(children) = self;

    let mut result = quote! {
      let parent = this;
    };

    for child in children {
      result.extend(match child {
        Child::CodeBlock(block   ) => quote! { #block },
        Child::Inserted (inserted) => inserted.generate(),
        Child::Entity   (entity  ) => {
          let parent = Ident::new("parent", Span::call_site());
          Parented { parent, entity }.generate()
        },
      });
    }

    quote! { { #result }; }
  }
}


struct MethodCall(Ident, Vec<Expr>);

impl Parse for MethodCall {
  fn parse(input: ParseStream) -> Result<Self> {
    let name = input.parse()?;

    let args = {
      let content;
      parenthesized!(content in input);

      content
        .parse_terminated(Expr::parse, Token![,])?
        .into_iter().collect()
    };

    Ok(MethodCall(name, args))
  }
}

impl Generate for MethodCall {
  fn generate(self) -> proc_macro2::TokenStream {
    let MethodCall(name, args) = self;
    quote! { entity. #name (#(#args),*); }
  }
}
