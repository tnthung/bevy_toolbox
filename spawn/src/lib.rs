use proc_macro::TokenStream;


/// This macro is used to simplify the entity creation of the bevy engine.
///
/// # Example
///
/// ## To create a single entity with a transform component:
///
/// ```rs, no_run
/// spawn! { commands // world, or anything that have `spawn` method which returns `EntityCommands`
///   (Transform::default())
/// }
/// ```
///
/// ## To create a single entity with a button component with a text child:
///
/// ```rs, no_run
/// spawn! { commands
///   (Button)
///   .[ // children
///     (Text::new("Hello, World!"))
///   ];
/// }
/// ```
///
/// ## To create a single entity with a button component with a text child, print `Hello, World!` when
/// clicked:
///
/// ```rs, no_run
/// spawn! { commands
///   (Button)
///     .[ // children
///       (Text::new("Hello, World!"))
///     ]
///     .observe(|_: Trigger<Pointer<Click>>| { println!("Hello, World!"); });
/// }
/// ```
///
/// ## Spawn multiple entities:
///
/// ```rs, no_run
/// spawn! { commands
///   (Button);
///   (Button);
///   (Button);
/// }
/// ```
///
/// ## Spawn children for existing entity:
///
/// * Assuming the parent `Entity` called `uwu` and already spawned.
///
/// ```rs, no_run
/// spawn! { commands
///   uwu > (Button);
/// }
/// ```
///
/// ## Name a entity and reference it later:
///
/// ```rs, no_run
/// spawn! { commands
///   owo (Text::new("Hello, World!"));
///   uwu (Button);
///
///   uwu > (Button);  // another way of spawning children of `uwu`
///
///   (Button)
///     .observe(move |_: Trigger<Pointer<Click>>, mut commands: Commands| {
///       commands.entity(owo).insert(Text::new("This is new text!"));
///     });
/// }
/// ```
///
/// ## Reference current entity with `this`:
///
/// ```rs, no_run
/// spawn! { commands
///   (Button, BackgroundColor(Color::srgb(0.0, 0.0, 0.0)))
///     .observe(move |_: Trigger<Pointer<Click>>, mut commands: Commands| {
///        commands.entity(this).insert(BackgroundColor(Color::srgb(1.0, 1.0, 1.0)));
///     });
/// }
/// ```
///
/// ## `observe` method name can be omitted:
///
/// ```rs, no_run
/// spawn! { commands
///   (Button)
///     .(|_: Trigger<Pointer<Click>>| { println!("Hello, World!"); });
/// }
/// ```
///
/// # Grammar
///
/// * `<TOKEN>*` means repeat 0-inf times separated by `TOKEN`, the last `TOKEN` is optional.
///
/// ```txt
/// spawn       ::= spawner (parenting? entity)<';'>* ;
/// entity      ::= name? '(' component<','> ')' extension* ;
/// extension   ::= '.' (children | method_call | observe) ;
/// method_call ::= name '(' EXPR<','>* ')' ;
/// observe     ::= '(' EXPR_CLOSURE ')' ;
/// children    ::= '[' entity<';'>* ']' ;
/// parenting   ::= name '>' ;
/// method_call ::= '.' name '(' EXPR<','>* ')' ;
/// name        ::= IDENT ;
/// component   ::= EXPR ;
/// ```
#[proc_macro]
pub fn spawn(input: TokenStream) -> TokenStream {
  use syn::*;
  use syn::parse::*;
  use syn::token::*;
  use quote::*;

  if input.is_empty() { return TokenStream::new(); }

  struct Spawn {
    spawner : Ident,
    entities: Vec<Entity>,
  }

  struct Entity {
    parent    : Option<Ident>,
    name      : Option<Ident>,
    components: Vec<Expr>,
    extensions: Vec<Extension>,
  }

  enum Extension {
    Children(Vec<Entity>),
    Unfinished(Token![.], Option<Ident>),
    MethodCall(Option<Ident>, Vec<Expr>),
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

  impl Parse for Spawn {
    fn parse(input: ParseStream) -> Result<Self> {
      Ok(Spawn {
        spawner: input.parse()?,
        entities: input
          .parse_terminated(Entity::parse, Token![;])?
          .into_iter()
          .collect(),
      })
    }
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

        for mut child in content.parse_terminated(Entity::parse, Token![;])? {
          if child.parent.is_some() {
            return Err(Error::new(
              child.parent.as_ref().unwrap().span(),
              "Only top level entity can have parent"));
          }

          child.parent = Some(Ident::new("parent", proc_macro2::Span::call_site()));
          children.push(child);
        }

        return Ok(Extension::Children(children));
      }

      return Ok(Extension::Unfinished(dot, None));
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
        Extension::Children(entities) => {
          let mut result = quote! {};

          for entity in entities {
            result.extend(convert(entity, false));
          }

          quote! {{ #result }}
        },

        Extension::Unfinished(dot, name       ) => quote! { entity #dot #name },
        Extension::MethodCall(None      , args) => quote! { entity.observe(#(#args),*); },
        Extension::MethodCall(Some(name), args) => quote! { entity. #name (#(#args),*); },
      });
    }

    let naming = name.map(|n| quote! { let #n = });
    quote! { #naming { #content this }; }
  }

  let Spawn { spawner, entities } =
    Spawn::parse.parse(input).unwrap();

  let mut result = quote! {};

  for entity in entities {
    result.extend(convert(entity, true));
  }

  quote! {{
    let spawner = &mut #spawner;
    #result
  };}.into()
}
