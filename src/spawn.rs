//! # Grammar
//!
//! * `<TOKEN>*` means repeat 0-inf times separated by `TOKEN`, the last `TOKEN` is optional.
//!
//! ```txt
//! spawn        ::= spawner (top_level | ';')* ;
//!
//! definition   ::= '(' component<','>* ')' ('.' extension)* ('.' children)* ;
//! entity       ::= name? definition ;
//!
//! parented     ::= name '>' entity ;
//! inserted     ::= name '+' definition ;
//!
//! child        ::= entity | inserted | flow<child    > | code_block ;
//! top_level    ::= entity | inserted | flow<top_level> | code_block | parented ;
//!
//! extension    ::= observe | method_call | code_block ;
//! observe      ::= '(' argument ')' ;
//! children     ::= '[' (child | ';')* ']' ;
//! method_call  ::= name '(' argument<','>* ')' ;
//!
//! flow     <T> ::= if<T> | if_let<T> | for<T> | while<T> | while_let<T> ;
//! control  <T> ::= 'break' | 'continue' | T | ';' ;
//! if       <T> ::= 'if' EXPR '{' control<T>* '}' ('else' flow<T>)?;
//! if_let   <T> ::= 'if' 'let' PAT '=' EXPR '{' control<T>* '}' ('else' flow<T>)?;
//! for      <T> ::= 'for' PAT in EXPR '{' control<T>* '}' ;
//! while    <T> ::= 'while' EXPR '{' control<T>* '}' ;
//! while_let<T> ::= 'while' 'let' PAT '=' EXPR '{' control<T>* '}' ;
//!
//! name         ::= IDENT ;
//! spawner      ::= IDENT | '[' EXPR ']' ;
//! argument     ::= EXPR ;
//! component    ::= EXPR ;
//! code_block   ::= EXPR_BLOCK ;
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
      top_level: {
        let mut top_level = vec![];

        while !input.is_empty() {
          if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
            continue;
          }

          top_level.push(input.parse()?);
        }

        top_level
      },
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
  Expr (proc_macro2::TokenStream),
}

impl Parse for Spawner {
  fn parse(input: ParseStream) -> Result<Self> {
    if input.peek(Ident) {
      Ok(Spawner::Ident(input.parse()?))
    } else if input.peek(Bracket) {
      let token = input.parse::<Group>()?;
      Ok(Spawner::Expr(token.stream()))
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
        let mut spawner = (#expr);
        let spawner = &mut spawner;
      },
    }
  }
}


#[derive(Clone)]
struct Definition {
  components: proc_macro2::TokenStream,
  extensions: Vec<Extension>,
  children  : Vec<Children>,
}

impl Parse for Definition {
  fn parse(input: ParseStream) -> Result<Self> {
    if !input.peek(Paren) {
      return Err(input.error("Expected '(' for definition"));
    }

    let components = input.parse::<Group>()?.stream();

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
      let mut entity = spawner.spawn((#components));

      let this = entity.id();
    };

    for ext in extensions {
      content.extend(ext.generate());
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
      let mut entity = spawner.spawn((#components));

      let this = entity.id();
      entity.set_parent(#parent);
    };

    for ext in extensions {
      content.extend(ext.generate());
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
      let mut entity = entity.insert((#components));

      let this = entity.id();
    };

    for ext in extensions {
      content.extend(ext.generate());
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
  Flow     (Flow<Child>),
  CodeBlock(Group),
}

impl Parse for Child {
  fn parse(input: ParseStream) -> Result<Self> {
    if input.peek(Paren) { return Ok(Child::Entity   (input.parse()?)) }
    if input.peek(Brace) { return Ok(Child::CodeBlock(input.parse()?)) }

    if input.peek(Token![if   ]) { return Ok(Child::Flow(input.parse()?)) }
    if input.peek(Token![for  ]) { return Ok(Child::Flow(input.parse()?)) }
    if input.peek(Token![while]) { return Ok(Child::Flow(input.parse()?)) }

    if input.peek(Ident) {
      if input.peek2(Paren)     { return Ok(Child::Entity  (input.parse()?)) }
      if input.peek2(Token![+]) { return Ok(Child::Inserted(input.parse()?)) }

      input.parse::<Ident>()?;
      return Err(input.error(
        if input.peek(Token![>]) { "Parented is not allowed as a child" }
        else { "Expected '+' for inserted, or '()' for entity" }));
    }

    Err(input.error("Expected entity, inserted, flow statement or code block"))
  }
}

impl Generate for Child {
  fn generate(&self) -> proc_macro2::TokenStream {
    match self {
      Child::CodeBlock(block   ) => quote! { #block },
      Child::Inserted (inserted) => inserted.generate(),
      Child::Flow     (flow    ) => flow    .gen_irrefutable(),
      Child::Entity   (entity  ) => {
        let parent = Ident::new("parent", Span::call_site());
        let entity = entity.clone();
        Parented { parent, entity }.generate()
      },
    }
  }
}


#[derive(Clone)]
enum TopLevel {
  Entity   (Entity),
  Parented (Parented),
  Inserted (Inserted),
  Flow     (Flow<TopLevel>),
  CodeBlock(Group),
}

impl Parse for TopLevel {
  fn parse(input: ParseStream) -> Result<Self> {
    if input.peek(Paren) { return Ok(TopLevel::Entity   (input.parse()?)) }
    if input.peek(Brace) { return Ok(TopLevel::CodeBlock(input.parse()?)) }

    if input.peek(Token![if   ]) { return Ok(TopLevel::Flow(input.parse()?)) }
    if input.peek(Token![for  ]) { return Ok(TopLevel::Flow(input.parse()?)) }
    if input.peek(Token![while]) { return Ok(TopLevel::Flow(input.parse()?)) }

    if input.peek(Ident) {
      if input.peek2(Paren)     { return Ok(TopLevel::Entity  (input.parse()?)) }
      if input.peek2(Token![>]) { return Ok(TopLevel::Parented(input.parse()?)) }
      if input.peek2(Token![+]) { return Ok(TopLevel::Inserted(input.parse()?)) }

      input.parse::<Ident>()?;
      return Err(input.error("Expected '>' for parented, '+' for inserted, or '()' for entity"));
    }

    Err(input.error("Expected parented, inserted, flow statement or code block"))
  }
}

impl Generate for TopLevel {
  fn generate(&self) -> proc_macro2::TokenStream {
    match self {
      TopLevel::Entity   (entity  ) => entity  .generate(),
      TopLevel::Parented (parented) => parented.generate(),
      TopLevel::Inserted (inserted) => inserted.generate(),
      TopLevel::Flow     (flow    ) => flow    .gen_irrefutable(),
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

impl Generate for Extension {
  fn generate(&self) -> proc_macro2::TokenStream {
    match self {
      Extension::Observe   (arg      ) => quote! { entity.observe(#arg); },
      Extension::MethodCall(method   ) => method.generate(),
      Extension::CodeBlock (block    ) => quote! {{ let mut entity = entity.reborrow(); #block }},
      Extension::Unfinished(dot, name) => {
        if let Some(name) = name {
          quote! { #dot #name }
        } else {
          quote! { #dot }
        }
      },
    }
  }
}


#[derive(Clone)]
struct Children(Vec<Child>);

impl Parse for Children {
  fn parse(input: ParseStream) -> Result<Self> {
    Ok(Children({
      let content;
      bracketed!(content in input);

      let mut children = vec![];
      while !content.is_empty() {
        if content.peek(Token![;]) {
          content.parse::<Token![;]>()?;
          continue;
        }

        children.push(content.parse()?);
      }

      children
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
      result.extend(child.generate());
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


#[derive(Clone)]
enum Flow<T: Generate+Parse> {
  If      (If<T>),
  IfLet   (IfLet<T>),
  For     (For<T>),
  While   (While<T>),
  WhileLet(WhileLet<T>),
}

impl<T: Generate+Parse> Parse for Flow<T> {
  fn parse(input: ParseStream) -> Result<Self> {
    if input.peek(Token![if]) {
      if input.peek2(Token![let]) {
        return Ok(Flow::IfLet(input.parse()?));
      } else {
        return Ok(Flow::If(input.parse()?));
      }
    }

    if input.peek(Token![for]) {
      return Ok(Flow::For(input.parse()?));
    }

    if input.peek(Token![while]) {
      if input.peek2(Token![let]) {
        return Ok(Flow::WhileLet(input.parse()?));
      } else {
        return Ok(Flow::While(input.parse()?));
      }
    }

    Err(input.error("Expected flow statement"))
  }
}

impl<T: Generate+Parse> Generate for Flow<T> {
  fn generate(&self) -> proc_macro2::TokenStream {
    match self {
      Flow::If      (if_      ) => if_      .generate(),
      Flow::IfLet   (if_let   ) => if_let   .generate(),
      Flow::For     (for_     ) => for_     .generate(),
      Flow::While   (while_   ) => while_   .generate(),
      Flow::WhileLet(while_let) => while_let.generate(),
    }
  }
}

impl<T: Generate+Parse> Flow<T> {
  fn gen_irrefutable(&self) -> proc_macro2::TokenStream {
    let content = self.generate();
    quote! {
      #[allow(irrefutable_let_patterns)]
      #content
    }
  }
}


#[derive(Clone)]
struct If<T: Generate+Parse> {
  if_      : syn::token::If,
  condition: Expr,
  body     : Vec<Control<T>>,
  else_    : Option<(syn::token::Else, std::boxed::Box<Flow<T>>)>,
}

impl<T: Generate+Parse> Parse for If<T> {
  fn parse(input: ParseStream) -> Result<Self> {
    let if_ = input.parse::<Token![if]>()?;

    let condition = input.parse()?;
    let body = {
      let content;
      braced!(content in input);

      let mut body = vec![];
      while !content.is_empty() {
        if content.peek(Token![;]) {
          content.parse::<Token![;]>()?;
          continue;
        }

        body.push(content.parse()?);
      }

      body
    };

    let else_ = if input.peek(Token![else]) {
      let else_ = input.parse::<Token![else]>()?;
      Some((else_, std::boxed::Box::new(input.parse()?)))
    } else {
      None
    };

    Ok(If { if_, condition, body, else_ })
  }
}

impl<T: Generate+Parse> Generate for If<T> {
  fn generate(&self) -> proc_macro2::TokenStream {
    let If { if_, condition, body, else_ } = self;

    let mut content = quote! {
      #if_ #condition
    };

    let mut content_body = quote! {};
    for item in body {
      content_body.extend(item.generate());
    }

    content.extend(quote! {{ #content_body }});

    if let Some((kw, else_)) = else_ {
      let else_gen = else_.generate();

      if matches!(**else_, Flow::If(_) | Flow::IfLet(_)) {
        content.extend(quote! { #kw #else_gen });
      } else {
        content.extend(quote! { #kw { #else_gen } });
      }
    }

    content
  }
}


#[derive(Clone)]
struct IfLet<T: Generate+Parse> {
  if_      : syn::token::If,
  let_     : syn::token::Let,
  pattern  : Pat,
  condition: Expr,
  body     : Vec<Control<T>>,
  else_    : Option<(syn::token::Else, std::boxed::Box<Flow<T>>)>,
}

impl<T: Generate+Parse> Parse for IfLet<T> {
  fn parse(input: ParseStream) -> Result<Self> {
    let if_  = input.parse::<Token![if]>()?;
    let let_ = input.parse::<Token![let]>()?;

    let pattern = Pat::parse_multi(input)?;
    input.parse::<Token![=]>()?;
    let condition = input.parse()?;

    let body = {
      let content;
      braced!(content in input);

      let mut body = vec![];
      while !content.is_empty() {
        if content.peek(Token![;]) {
          content.parse::<Token![;]>()?;
          continue;
        }

        body.push(content.parse()?);
      }

      body
    };

    let else_ = if input.peek(Token![else]) {
      let else_ = input.parse::<Token![else]>()?;
      Some((else_, std::boxed::Box::new(input.parse()?)))
    } else {
      None
    };

    Ok(IfLet { if_, let_, pattern, condition, body, else_ })
  }
}

impl<T: Generate+Parse> Generate for IfLet<T> {
  fn generate(&self) -> proc_macro2::TokenStream {
    let IfLet { if_, let_, pattern, condition, body, else_ } = self;

    let mut content = quote! {
      #if_ #let_ #pattern = #condition
    };

    let mut content_body = quote! {};
    for item in body {
      content_body.extend(item.generate());
    }

    content.extend(quote! {{ #content_body }});

    if let Some((kw, else_)) = else_ {
      let else_gen = else_.generate();

      if matches!(**else_, Flow::If(_) | Flow::IfLet(_)) {
        content.extend(quote! { #kw #else_gen });
      } else {
        content.extend(quote! { #kw { #else_gen } });
      }
    }

    content
  }
}


#[derive(Clone)]
struct For<T: Generate+Parse> {
  for_   : syn::token::For,
  in_    : syn::token::In,
  pattern: Pat,
  iter   : Expr,
  body   : Vec<Control<T>>,
}

impl<T: Generate+Parse> Parse for For<T> {
  fn parse(input: ParseStream) -> Result<Self> {
    let for_ = input.parse::<Token![for]>()?;

    let pattern = Pat::parse_multi(input)?;
    let in_     =input.parse::<Token![in]>()?;
    let iter    = input.parse()?;

    let body = {
      let content;
      braced!(content in input);

      let mut body = vec![];
      while !content.is_empty() {
        if content.peek(Token![;]) {
          content.parse::<Token![;]>()?;
          continue;
        }

        body.push(content.parse()?);
      }

      body
    };

    Ok(For { for_, in_, pattern, iter, body })
  }
}

impl<T: Generate+Parse> Generate for For<T> {
  fn generate(&self) -> proc_macro2::TokenStream {
    let For { for_, in_, pattern, iter, body } = self;

    let header = quote! {
      #for_ #pattern #in_ #iter
    };

    let mut content_body = quote! {};
    for item in body {
      content_body.extend(item.generate());
    }

    quote! {#header { #content_body }}
  }
}


#[derive(Clone)]
struct While<T: Generate+Parse> {
  while_   : syn::token::While,
  condition: Expr,
  body     : Vec<Control<T>>,
}

impl<T: Generate+Parse> Parse for While<T> {
  fn parse(input: ParseStream) -> Result<Self> {
    let while_ = input.parse::<Token![while]>()?;

    let condition = input.parse()?;

    let body = {
      let content;
      braced!(content in input);

      let mut body = vec![];
      while !content.is_empty() {
        if content.peek(Token![;]) {
          content.parse::<Token![;]>()?;
          continue;
        }

        body.push(content.parse()?);
      }

      body
    };

    Ok(While { while_, condition, body })
  }
}

impl<T: Generate+Parse> Generate for While<T> {
  fn generate(&self) -> proc_macro2::TokenStream {
    let While { while_, condition, body } = self;

    let header = quote! {
      #while_ #condition
    };

    let mut content_body = quote! {};
    for item in body {
      content_body.extend(item.generate());
    }

    quote! {#header { #content_body }}
  }
}


#[derive(Clone)]
struct WhileLet<T: Generate+Parse> {
  while_   : syn::token::While,
  let_     : syn::token::Let,
  pattern  : Pat,
  condition: Expr,
  body     : Vec<Control<T>>,
}

impl<T: Generate+Parse> Parse for WhileLet<T> {
  fn parse(input: ParseStream) -> Result<Self> {
    let while_ = input.parse::<Token![while]>()?;
    let let_   = input.parse::<Token![let]>()?;

    let pattern = Pat::parse_multi(input)?;
    input.parse::<Token![=]>()?;
    let condition = input.parse()?;

    let body = {
      let content;
      braced!(content in input);

      let mut body = vec![];
      while !content.is_empty() {
        if content.peek(Token![;]) {
          content.parse::<Token![;]>()?;
          continue;
        }

        body.push(content.parse()?);
      }

      body
    };

    Ok(WhileLet { while_, let_, pattern, condition, body })
  }
}

impl<T: Generate+Parse> Generate for WhileLet<T> {
  fn generate(&self) -> proc_macro2::TokenStream {
    let WhileLet { while_, let_, pattern, condition, body } = self;

    let header = quote! {
      #[allow(irrefutable_let_patterns)]
      #while_ #let_ #pattern = #condition
    };

    let mut content_body = quote! {};
    for item in body {
      content_body.extend(item.generate());
    }

    quote! {#header { #content_body }}
  }
}


#[derive(Clone)]
enum Control<T: Generate+Parse> {
  Break(syn::token::Break),
  Continue(syn::token::Continue),
  Item(T),
}

impl<T: Generate+Parse> Parse for Control<T> {
  fn parse(input: ParseStream) -> Result<Self> {
    if input.peek(Token![break]) {
      return Ok(Control::Break(input.parse()?));
    }

    if input.peek(Token![continue]) {
      return Ok(Control::Continue(input.parse()?));
    }

    Ok(Control::Item(input.parse()?))
  }
}

impl<T: Generate+Parse> Generate for Control<T> {
  fn generate(&self) -> proc_macro2::TokenStream {
    match self {
      Control::Break   (item) => quote! { #item; },
      Control::Continue(item) => quote! { #item; },
      Control::Item    (item) => item.generate(),
    }
  }
}
