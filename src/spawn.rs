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
//! spawner     ::= IDENT | '[' EXPR ']' ;
//! argument    ::= EXPR ;
//! component   ::= EXPR ;
//! code_block  ::= EXPR_BLOCK ;
//! ```
use crate::*;


#[derive(Clone)]
pub struct Spawn {
  spawner  : Spawner,
  top_level: Vec<TopLevel>,
}

impl Parse for Spawn {
  fn parse(input: ParseStream) -> Result<Self> {
    Ok(Spawn {
      spawner  : input.parse()?,
      top_level: input
        .parse_terminated(TopLevel::parse, Token![;])?
        .into_iter()
        .collect(),
    })
  }
}

impl Generate for Spawn {
  fn generate(&self) -> proc_macro2::TokenStream {
    let Spawn { spawner, top_level } = self;

    let mut content = spawner.generate();

    for e in top_level {
      content.extend(e.generate());
    }

    quote! { { #content }; }
  }
}


#[derive(Clone)]
enum Spawner {
  Ident(Ident),
  Expr (Expr),
}

impl Parse for Spawner {
  fn parse(input: ParseStream) -> Result<Self> {
    if input.peek(Ident) {
      Ok(Spawner::Ident(input.parse()?))
    } else if input.peek(Bracket) {
      let content;
      bracketed!(content in input);
      Ok(Spawner::Expr(content.parse()?))
    } else {
      Err(input.error("Expected identifier or expression"))
    }
  }
}

impl Generate for Spawner {
  fn generate(&self) -> proc_macro2::TokenStream {
    match self {
      Spawner::Ident(ident) => quote! { let spawner = &mut #ident; },
      Spawner::Expr (expr ) => quote! {
        let mut spawner = #expr ;
        let spawner = &mut spawner;
      },
    }
  }
}


#[derive(Clone)]
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
          input.parse::<Token![.]>()?;
          return Err(input.error("Extensions cannot be chained after children group"));
        }

        input.parse::<Token![.]>()?;
        children.push(input.parse()?);
      }

      children
    };

    if !input.peek(Token![;]) && !input.is_empty() {
      return Err(input.error("Unexpected token, did you forget a ';' for previous entity?"));
    }

    Ok(Definition {
      components,
      extensions,
      children,
    })
  }
}


#[derive(Clone)]
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
  fn generate(&self) -> proc_macro2::TokenStream {
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

    let naming = name.clone().map(|n| quote! { let #n = });
    quote! { #naming { #content this }; }
  }
}


#[derive(Clone)]
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
  fn generate(&self) -> proc_macro2::TokenStream {
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

    let naming = name.clone().map(|n| quote! { let #n = });
    quote! { #naming { #content this }; }
  }
}


#[derive(Clone)]
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
  fn generate(&self) -> proc_macro2::TokenStream {
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


#[derive(Clone)]
enum Child {
  Entity   (Entity),
  Inserted (Inserted),
  CodeBlock(Group),
}

impl Parse for Child {
  fn parse(input: ParseStream) -> Result<Self> {
    if input.peek(Paren) { return Ok(Child::Entity   (input.parse()?)) }
    if input.peek(Brace) { return Ok(Child::CodeBlock(input.parse()?)) }

    if input.peek(Ident) {
      if input.peek2(Paren)     { return Ok(Child::Entity  (input.parse()?)) }
      if input.peek2(Token![+]) { return Ok(Child::Inserted(input.parse()?)) }

      input.parse::<Ident>()?;
      return Err(input.error(
        if input.peek(Token![>]) { "Parented is not allowed as a child" }
        else { "Expected '+' for inserted, or '()' for entity" }));
    }

    Err(input.error("Expected entity, inserted or code block"))
  }
}


#[derive(Clone)]
enum TopLevel {
  Entity   (Entity),
  Parented (Parented),
  Inserted (Inserted),
  CodeBlock(Group),
}

impl Parse for TopLevel {
  fn parse(input: ParseStream) -> Result<Self> {
    if input.peek(Paren) { return Ok(TopLevel::Entity   (input.parse()?)) }
    if input.peek(Brace) { return Ok(TopLevel::CodeBlock(input.parse()?)) }

    if input.peek(Ident) {
      if input.peek2(Paren)     { return Ok(TopLevel::Entity  (input.parse()?)) }
      if input.peek2(Token![>]) { return Ok(TopLevel::Parented(input.parse()?)) }
      if input.peek2(Token![+]) { return Ok(TopLevel::Inserted(input.parse()?)) }

      input.parse::<Ident>()?;
      return Err(input.error("Expected '>' for parented, '+' for inserted, or '()' for entity"));
    }

    Err(input.error("Expected parented, inserted or code block"))
  }
}

impl Generate for TopLevel {
  fn generate(&self) -> proc_macro2::TokenStream {
    match self {
      TopLevel::Entity   (entity  ) => entity  .generate(),
      TopLevel::Parented (parented) => parented.generate(),
      TopLevel::Inserted (inserted) => inserted.generate(),
      TopLevel::CodeBlock(block   ) => quote! { #block },
    }
  }
}


#[derive(Clone)]
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


#[derive(Clone)]
struct Children(Vec<Child>);

impl Parse for Children {
  fn parse(input: ParseStream) -> Result<Self> {
    Ok(Children({
      let content;
      bracketed!(content in input);

      content
        .parse_terminated(Child::parse, Token![;])?
        .into_iter().collect()
    }))
  }
}

impl Generate for Children {
  fn generate(&self) -> proc_macro2::TokenStream {
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
          let entity = entity.clone();
          Parented { parent, entity }.generate()
        },
      });
    }

    quote! { { #result }; }
  }
}


#[derive(Clone)]
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
  fn generate(&self) -> proc_macro2::TokenStream {
    let MethodCall(name, args) = self;
    quote! { entity. #name (#(#args),*); }
  }
}
