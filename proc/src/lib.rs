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
/// spawn       ::= spawner entity<';'>* ;
/// entity      ::= parenting? name? '(' component<','> ')' (method_call | children)* ;
/// method_call ::= '.' name '(' EXPR<','>* ')' | observe ;
/// observe     ::= '.' '(' EXPR_CLOSURE ')' ;
/// children    ::= '.' '[' entity<';'>* ']' ;
/// parenting   ::= name '>' ;
/// component   ::= EXPR ;
/// method_call ::= '.' name '(' EXPR<','>* ')' ;
/// name        ::= IDENT ;
/// ```
#[proc_macro]
pub fn spawn(input: TokenStream) -> TokenStream {
  use syn::*;
  use syn::parse::*;
  use syn::token::*;
  use quote::*;

  if input.is_empty() { return TokenStream::new(); }

  struct Entity {
    parent      : Option<Ident>,
    name        : Option<Ident>,
    components  : Vec<Expr>,
    method_calls: Vec<(Option<Ident>, Vec<Expr>)>,
    children    : Vec<Entity>,
  }

  struct Spawn {
    spawner: Ident,
    entities: Vec<Entity>,
  }

  impl Parse for Entity {
    fn parse(input: ParseStream) -> Result<Self> {
      let mut children     = vec![];
      let mut method_calls = vec![];

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

      while input.peek(Token![.]) {
        input.parse::<Token![.]>()?;

        if input.peek(Ident) {
          let method = input.parse()?;
          let content;
          parenthesized!(content in input);

          method_calls.push((method, content.parse_terminated(
            Expr::parse, Token![,])?.into_iter().collect()));
          continue;
        }

        if input.peek(Paren) {
          let content;
          parenthesized!(content in input);

          method_calls.push((None, vec![content.parse()?]));
          continue;
        }

        if input.peek(Bracket) {
          let content;
          bracketed!(content in input);

          children.extend(content
            .parse_terminated(Entity::parse, Token![;])?);
          continue;
        }

        return Err(input.error("expected method call or children"));
      }

      Ok(Entity {
        parent,
        name,
        components,
        method_calls,
        children,
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

  fn convert(entity: Entity) -> proc_macro2::TokenStream {
    let mut result = quote! {};

    let has_children = !entity.children.is_empty();
    let children     = entity.children.into_iter().map(convert);
    let components   = entity.components;
    let method_calls = entity.method_calls;

    if let Some(name) = entity.name {
      result.extend(quote! { let #name = });
    }

    let parenting = if let Some(parent) = entity.parent {
      quote! { entity.set_parent(#parent); }
    } else {
      quote! {}
    };

    let children = if has_children {
      quote! { entity.with_children(|spawner| { #(#children)* }); }
    } else {
      quote! {}
    };

    let method_calls = method_calls.iter().map(|(method, args)| {
      let method = method.as_ref().map(|m| quote! { #m }).unwrap_or(quote! { observe });
      quote! { entity.#method(#(#args),*); }
    });

    result.extend(quote! {
      {
        let mut entity = spawner.spawn((
          #(#components),*
        ));

        let this = entity.id();

        #(#method_calls;)*

        #children
        #parenting

        this
      };
    });

    result
  }

  let Spawn { spawner, entities } =
    Spawn::parse.parse(input).unwrap();

  let mut result = quote! {};

  for entity in entities {
    result.extend(convert(entity));
  }

  quote! {{
    let spawner = &mut #spawner;
    #result
  };}.into()
}
