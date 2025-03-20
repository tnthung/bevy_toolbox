use proc_macro::TokenStream;


/// This macro is used to simplify the entity creation of the bevy engine.
///
/// # Basics
///
/// ## Spawner
///
/// Spawner is the object that have `spawn` method which takes a bevy bundle and returns
/// `EntityCommands` as the result.
///
/// ```rs, no_run
/// fn foo(mut commands: Commands) {
///   // commands can be used as spawner
/// }
///
/// fn bar(world: &mut World) {
///   // world can be used as spawner
/// }
/// ```
///
/// ## Top level
///
/// Top level means the part of the macro thats been directly quoted by the macro itself.
///
/// ```rs, no_run
/// spawn! { commands
///   // here is the top level
///
///   ().[
///     // here is not the top level
///   ];
/// }
/// ```
///
/// ## Entity definition
///
/// An entity definition is a tuple of components that will be spawned as an entity.
///
/// ```rs, no_run
/// spawn! { commands
///   // entity definition
///   (Button, Node::default());
/// }
/// ```
///
/// Top level can accept multiple entity definitions.
///
/// ```rs, no_run
/// spawn! { commands
///   // entity 1
///   (Button, Node::default());
///
///   // entity 2
///   (Button, Node::default());
/// }
/// ```
///
/// ## Order
///
/// The order of any bit in the macro matters. The execution order is strictly follow the macro input.
///
/// ```rs, no_run
/// // entity `a` will always being spawned before `b`
/// spawn! { commands
///   a ();
///   b ();
/// }
/// ```
///
/// ## Naming
///
/// An entity can be named for later reference. The variable will hold the `Entity` of the corresponding
/// entity, NOT THE `EntityCommands`.
///
/// ```rs, no_run
/// spawn! { commands
///   entity_1 (Button, Node::default());
///   entity_2 (Button, Node::default());
///
///   (Button)
///     .(move |_: Trigger<Pointer<Click>>, mut commands: Commands| {
///       // referencing the entity_1 after it's been spawned
///       commands.entity(entity_1).despawn();
///     });
///
///   {
///     println!("{entity_1:?}");
///     println!("{entity_2:?}");
///   };
/// }
/// ```
///
/// ## Parenting
///
/// A top level entities can have explicit parent. Parenting is done by using `>` operator.
///
/// ```rs, no_run
/// spawn! { commands
///   my_entity (Button);
///
///   // this entity will be spawned as a child of `my_entity`
///   my_entity > (Button);
///
///   // it's also possible to use the entity outside the macro
///   // just make sure the parent is `Entity` type
///   some_outside_entity > (Button);
///
///   // parenting and naming can be combined
///   parent > child (Button);
/// }
/// ```
///
/// ## Code block injection
///
/// Since the entities inside the macro is enclosed within a generated scope to prevent the namespace
/// pollution, code block injection makes it possible to execute code without leaving the macro.
///
/// ```rs, no_run
/// spawn! { commands
///   entity_1 (Button);
///   entity_2 (Button);
///
///   {
///     println!("This is inside a code block!");
///     println!("{entity_1:?}");
///     println!("{entity_2:?}");
///
///     // you can do whatever you want here, just make sure the ownership of spawner will not be taken
///     // if you want to spawn any entities after this code block
///   };
/// }
/// ```
///
/// ## Extension
///
/// An entity can be extended with any number of:
///
/// 1. Children Group
/// 1. Method Call
/// 1. Code Block
///
/// All extensions are started with `.` after the entity definition.
///
/// ### Children Group
///
/// Children group is a group of entities that will be spawned as children of the parent entity. We
/// use `[]` to define a new children group. A children group can have multiple entities. Within the
/// same group, the entities can reference each other, but entities in 2 different groups under same
/// parent can't.
///
/// ```rs, no_run
/// spawn! { commands
///   (Button)
///     .[
///       a (Text::new("Hello, World!"));
///       b (Text::new("Hello, World!"));
///
///       {
///         // code block injection is also possible
///         // you can access `a` and `b` here
///       };
///     ]
///     .[
///       c (Text::new("Hello, World!"));
///
///       {
///         // you can't access `a`, `b`, but `c`
///       };
///     ];
/// ```
///
/// ### Method Call
///
/// Method call is a call to a method of `EntityCommands`. The auto completion is supported for the
/// method name and the arguments.
///
/// ```rs, no_run
/// spawn! { commands
///   (Button)
///     .[(Text::new("Hello, World!"))]
///     .observe(|_: Trigger<Pointer<Click>>| { println!("Hello, World!"); });
/// }
/// ```
///
/// Since `observe` is most likely to be used, a shortcut is provided to omit the method name.
///
/// ```rs, no_run
/// spawn! { commands
///   (Button)
///     .[(Text::new("Hello, World!"))]
///     .(|_: Trigger<Pointer<Click>>| { println!("Hello, World!"); });
/// }
/// ```
///
/// To reference the current entity, you can use `this` for `Entity` and `entity` for `EntityCommands`.
///
/// ```rs, no_run
/// spawn! { commands
///   (Button, BackgroundColor(Color::srgb(0.0, 0.0, 0.0)))
///     .[(Text::new("Hello, World!"))]
///     .(move |_: Trigger<Pointer<Click>>, mut commands: Commands| {
///       commands.entity(this).insert(BackgroundColor(Color::srgb(1.0, 1.0, 1.0)));
///     });
/// }
/// ```
///
/// ### Code Block
///
/// Code block is a block of code that will be executed in the context of the entity. As previously
/// mentioned, the code block can also access `this` and `entity` variables.
///
/// ```rs, no_run
/// spawn! { commands
///   (Button)
///     .{
///       // print the entity id of the current entity
///       println!("{this:?}");
///
///       // manually adding a child
///       entity.with_child((Text::new("Hello, World!")));
///     };
/// }
/// ```
///
/// # Grammar
///
/// * `<TOKEN>*` means repeat 0-inf times separated by `TOKEN`, the last `TOKEN` is optional.
///
/// ```txt
/// spawn       ::= spawner (parenting? entity | code_block)<';'>* ;
/// entity      ::= name? '(' component<','> ')' extension* ;
/// extension   ::= '.' (children | method_call | observe | code_block) ;
/// method_call ::= name '(' EXPR<','>* ')' ;
/// observe     ::= '(' EXPR_CLOSURE ')' ;
/// children    ::= '[' (entity | code_block)<';'>* ']' ;
/// parenting   ::= name '>' ;
/// method_call ::= '.' name '(' EXPR<','>* ')' ;
/// name        ::= IDENT ;
/// component   ::= EXPR ;
/// code_block  ::= EXPR_BLOCK ;
/// ```
pub fn spawn_impl(input: TokenStream) -> TokenStream {
  use syn::*;
  use syn::parse::*;
  use syn::token::*;
  use quote::*;

  if input.is_empty() { return TokenStream::new(); }

  struct Spawn {
    spawner : Ident,
    children: Vec<OrCode<Entity>>,
  }

  struct Entity {
    parent    : Option<Ident>,
    name      : Option<Ident>,
    components: Vec<Expr>,
    extensions: Vec<Extension>,
  }

  enum Extension {
    Children  (Vec<OrCode<Entity>>),
    CodeBlock (proc_macro2::Group),
    Unfinished(Token![.], Option<Ident>),
    MethodCall(Option<Ident>, Vec<Expr>),
  }

  enum OrCode<T: Parse> {
    Code(proc_macro2::Group),
    Else(T),
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
        children: input
          .parse_terminated(OrCode::<Entity>::parse, Token![;])?
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
