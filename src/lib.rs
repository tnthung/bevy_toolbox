use proc_macro::TokenStream;

mod spawn;


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
/// ## Insertion
///
/// Insertion is a way to add some components to an existing entity. The entity must be named and spawned
/// in advanced in order to be referenced.
///
/// ```rs, no_run
/// spawn! { commands
///   my_entity (Button);
///
///   // add a background color to `my_entity`
///   my_entity + (BackgroundColor(Color::srgb(0.0, 0.0, 0.0)));
///
///   // in the children group, it's also possible to insert components
///   my_fancy_button (Button).[
///     txt (Text::new("Hello, World!"));
///
///     // add a background color to `txt`
///     txt + (BackgroundColor(Color::srgb(0.0, 0.0, 0.0)));
///   ];
///
///   // extensions are still available
///   my_fancy_button + (BackgroundColor(Color::srgb(0.0, 0.0, 0.0)))
///     .(move |_: Trigger<Pointer<Click>>, mut commands: Commands| { /* ... */ });
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
/// spawn       ::= spawner top_level<';'>* ;
///
/// definition  ::= '(' component<','>* ')' ('.' extension)* ;
/// entity      ::= name? definition ;
///
/// parented    ::= name '>' entity ;
/// inserted    ::= name '+' definition ;
///
/// child       ::= entity | inserted | code_block ;
/// top_level   ::= entity | inserted | code_block | parented ;
///
/// extension   ::= observe | children | method_call | code_block ;
/// observe     ::= '(' argument ')' ;
/// children    ::= '[' child<';'>* ']' ;
/// method_call ::= name '(' argument<','>* ')' ;
///
/// name        ::= IDENT ;
/// spawner     ::= IDENT ;
/// argument    ::= EXPR ;
/// component   ::= EXPR ;
/// code_block  ::= EXPR_BLOCK ;
/// ```
#[proc_macro]
pub fn spawn(input: TokenStream) -> TokenStream {
  crate::spawn::spawn_impl(input)
}


trait Generate {
  fn generate(self) -> proc_macro2::TokenStream;
}
