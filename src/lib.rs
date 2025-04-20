mod spawn;
mod value;
mod color;
mod edges;
mod turns;

use proc_macro::TokenStream;
use proc_macro2::Group;
use proc_macro2::Span;
use syn::*;
use syn::parse::*;
use syn::token::*;
use quote::*;


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
/// If you want to use the expression for the spawner, you can wrap it with `[]`.
///
/// ```rs, no_run
/// fn foo(mut commands: Commands) {
///   spawn! { [commands.reborrow()] }
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
/// 1. Method Call
/// 1. Code Block
///
/// All extensions are started with `.` after the entity definition.
///
/// ### Method Call
///
/// Method call is a call to a method of `EntityCommands`. The auto completion is supported for the
/// method name and the arguments.
///
/// ```rs, no_run
/// spawn! { commands
///   (Button)
///     .observe(|_: Trigger<Pointer<Click>>| { println!("Hello, World!"); })
///     .[(Text::new("Hello, World!"))];
/// }
/// ```
///
/// Since `observe` is most likely to be used, a shortcut is provided to omit the method name.
///
/// ```rs, no_run
/// spawn! { commands
///   (Button)
///     .(|_: Trigger<Pointer<Click>>| { println!("Hello, World!"); })
///     .[(Text::new("Hello, World!"))];
/// }
/// ```
///
/// To reference the current entity, you can use `this` for `Entity` and `entity` for `EntityCommands`.
///
/// ```rs, no_run
/// spawn! { commands
///   (Button, BackgroundColor(Color::srgb(0.0, 0.0, 0.0)))
///     .(move |_: Trigger<Pointer<Click>>, mut commands: Commands| {
///       commands.entity(this).insert(BackgroundColor(Color::srgb(1.0, 1.0, 1.0)));
///     })
///     .[(Text::new("Hello, World!"))];
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
/// ## Children Group
///
/// Children group is a group of entities quoted by `[]` after the `.`. The entities in the group will
/// be spawned as children of the parent entity. One entity can have multiple children groups, but all
/// of them have to be after the extensions. This is because the `spawner` ownership will be temporarily
/// taken for method calls and code blocks, to prevent this from happening, the children group is forced
/// to be the last part of the entity definition. Within the same group, the entities can reference
/// each other, but entities in 2 different groups under same parent can't.
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
/// }
/// ```
///
/// ## Flow Control
///
/// `if`, `if_let`, `for`, `while`, and `while_let` are supported in the macro. The syntax is similar
/// to the Rust syntax, but the body is in the DSL this macro provides. The flow control can be used
/// in the top level and children group.
///
/// ### If
///
/// ```rs, no_run
/// fn foo(mut commands: Commands, number: i32) {
///   spawn! { commands
///     if number > 0 {
///       (Text::new("It's positive!"));
///     }
///
///     // you can also use `else if` statement
///     else if number < 0 {
///       (Text::new("It's negative!"));
///     }
///
///     // or just `else`
///     else {
///       (Text::new("It's zero!"));
///     }
///   }
/// }
/// ```
///
/// ### If Let
///
/// ```rs, no_run
/// fn foo(mut commands: Commands, number: Option<u32>) {
///   spawn! { commands
///     if let None = number {
///       (Text::new("No number!"));
///     }
///
///     // `else for` is also supported, but it must be the last control flow
///     // in fact, `else for`, `else while`, `else while let` are all supported
///     // but they must be the last else statement
///     else for i in 0..number.unwrap() {
///       (Text::new(format!("Number: {i}")), Node {
///         top: v!({i as f32 * 20.0}px),
///         ..Default::default()
///       });
///     }
///   }
/// }
/// ```
///
/// ### For
///
/// ```rs, no_run
/// fn foo(mut commands: Commands, number: i32) {
///   spawn! { commands
///     for i in 0..number {
///       if i == 100 {
///         // break is supported
///         break;
///       }
///
///       (Text::new(format!("Number: {i}")), Node {
///        top: v!({i as f32 * 20.0}px),
///        ..Default::default()
///       });
///     }
///   }
/// }
/// ```
///
/// ### While
///
/// ```rs, no_run
/// fn foo(mut commands: Commands, number: i32) {
///   spawn! { commands
///     while number > 0 {
///       if number % 3 == 1 {
///         // continue is supported
///         continue;
///       }
///
///       (Text::new(format!("Number: {number}")), Node {
///         top: v!({number as f32 * 20.0}px),
///         ..Default::default()
///       });
///     }
///   }
/// }
/// ```
///
/// ### While Let
///
/// ```rs, no_run
/// fn foo(mut commands: Commands, string: String) {
///   spawn! { commands
///     while let Some(c) = string.pop() {
///       (Text::new(format!("Char: {c}")), Node {
///         top: v!({string.len() as f32 * 20.0}px),
///         ..Default::default()
///       });
///     }
///   }
/// }
/// ```
///
/// # Grammar
///
/// * `<TOKEN>*` means repeat 0-inf times separated by `TOKEN`, the last `TOKEN` is optional.
///
/// ```txt
/// spawn        ::= spawner (top_level | ';')* ;
///
/// definition   ::= '(' component<','>* ')' ('.' extension)* ('.' children)* ;
/// entity       ::= name? definition ;
///
/// parented     ::= name '>' entity ;
/// inserted     ::= name '+' definition ;
///
/// child        ::= entity | inserted | flow<child    > | code_block ;
/// top_level    ::= entity | inserted | flow<top_level> | code_block | parented ;
///
/// extension    ::= observe | method_call | code_block ;
/// observe      ::= '(' argument ')' ;
/// children     ::= '[' (child | ';')* ']' ;
/// method_call  ::= name '(' argument<','>* ')' ;
///
/// flow     <T> ::= if<T> | if_let<T> | for<T> | while<T> | while_let<T> ;
/// control  <T> ::= 'break' | 'continue' | T | ';' ;
/// if       <T> ::= 'if' EXPR '{' control<T>* '}' ('else' flow<T>)?;
/// if_let   <T> ::= 'if' 'let' PAT '=' EXPR '{' control<T>* '}' ('else' flow<T>)?;
/// for      <T> ::= 'for' PAT in EXPR '{' control<T>* '}' ;
/// while    <T> ::= 'while' EXPR '{' control<T>* '}' ;
/// while_let<T> ::= 'while' 'let' PAT '=' EXPR '{' control<T>* '}' ;
///
/// name         ::= IDENT ;
/// spawner      ::= IDENT | '[' EXPR ']' ;
/// argument     ::= EXPR ;
/// component    ::= EXPR ;
/// code_block   ::= EXPR_BLOCK ;
/// ```
#[proc_macro]
pub fn spawn(input: TokenStream) -> TokenStream {
  apply::<crate::spawn::Spawn>(input, true)
}


/// This macro is used to simplify the creation of the bevy's `Val` enum.
///
/// # Syntax
///
/// There are 7 variants of the `Val` enum, and 2 syntax for each non-auto variant. In total, there
/// are 13 syntax.
///
/// * `+` in between tokens means there can be no space between them.
///
/// 1. `Val::Auto`    - `auto`, `@`
/// 1. `Val::Percent` - `number '%'` *space is optional* (e.g. `10%`)
/// 1. `Val::Px`      - `number + 'px'` (e.g. `10px`)
/// 1. `Val::Vw`      - `number + 'vw'` (e.g. `10vw`)
/// 1. `Val::Vh`      - `number + 'vh'` (e.g. `10vh`)
/// 1. `Val::Vmin`    - `number + 'vmin'` (e.g. `10vmin`)
/// 1. `Val::Vmax`    - `number + 'vmax'` (e.g. `10vmax`)
/// 1. `Val::Percent` - `{EXPR} + '%'` (e.g. `{10.0 + 20.0}%`)
/// 1. `Val::Px`      - `{EXPR} + 'px'` (e.g. `{10.0 + 20.0}px`)
/// 1. `Val::Vw`      - `{EXPR} + 'vw'` (e.g. `{10.0 + 20.0}vw`)
/// 1. `Val::Vh`      - `{EXPR} + 'vh'` (e.g. `{10.0 + 20.0}vh`)
/// 1. `Val::Vmin`    - `{EXPR} + 'vmin'` (e.g. `{10.0 + 20.0}vmin`)
/// 1. `Val::Vmax`    - `{EXPR} + 'vmax'` (e.g. `{10.0 + 20.0}vmax`)
///
/// ```rs, no_run
/// v!(auto);
/// v!(@);
/// v!(10%);
/// v!(10px);
/// v!({10.0 + 20.0}px);
/// v!(10 vw); // space not allowed, error will be thrown
/// ```
///
/// # Grammar
///
/// ```txt
/// v ::=
///   | 'auto'
///   | '@'
///   | number '%'
///   | number + 'px'
///   | number + 'vw'
///   | number + 'vh'
///   | number + 'vmin'
///   | number + 'vmax'
///   | '{' EXPR '}' '%'
///   | '{' EXPR '}' 'px'
///   | '{' EXPR '}' 'vw'
///   | '{' EXPR '}' 'vh'
///   | '{' EXPR '}' 'vmin'
///   | '{' EXPR '}' 'vmax'
///   ;
///
/// number ::= INT | FLOAT ;
/// ```
#[proc_macro]
pub fn v(input: TokenStream) -> TokenStream {
  apply::<crate::value::Value>(input, false)
}


/// This macro is used to simplify the creation of the bevy's `Color` enum.
///
/// # Syntax
///
/// The syntax is mimicking the CSS color syntax. The macro fully supports the auto completion for the
/// color names and the color spaces.
///
/// ## Hex
///
/// Hex colors are color codes that starts with `#` followed by 3, 4, 6, or 8 hex characters. The digits
/// are not case sensitive, so `#fff` is equivalent to `#FFF`. The color will be converted to `srgb`
/// color space.
///
/// * 3 hex digits - `#rgb`       (equivalent to `#rrggbb`)
/// * 4 hex digits - `#rgba`      (equivalent to `#rrggbbaa`)
/// * 6 hex digits - `#rrggbb`
/// * 8 hex digits - `#rrggbbaa`
///
/// ```rs, no_run
/// c!(#fff);
/// c!(#62a7ff);
/// ```
///
/// ## Functional Notation
///
/// All functional notations can have 3 or 4 arguments. The 4th argument is the alpha channel, if not
/// provided, it will be set to `1.0`.
///
/// * `srgb`   - Standard RGB color space
/// * `linear` - Linear RGB color space
/// * `hsl`    - Hue, Saturation, Lightness
/// * `hsv`    - Hue, Saturation, Value
/// * `hwb`    - Hue, Whiteness, Blackness
/// * `lab`    - L\*a\*b\* color space
/// * `lch`    - Luminance, Chroma, Hue
/// * `oklab`  - Oklab color space
/// * `oklch`  - Oklch color space
/// * `xyz`    - CIE 1931 XYZ color space
///
/// With the function selected, follow it with the arguments in the parentheses.
///
/// ```rs, no_run
/// c!(srgb(1.0, 0.5, 0.5));
/// c!(linear(1.0, 0.5, 0.5));
/// ```
///
/// ## CSS Named Colors
///
/// There are 149 named colors in CSS. The auto completion is supported for all of them. The color will
/// be converted to `srgb` color space.
///
/// ```rs, no_run
/// c!(firebrick);
/// c!(darkolivegreen);
/// ```
///
/// ## No wrap
///
/// The `c!` macro by default will wrap the color with `Color` enum, but sometimes you might just want
/// the inner value that color represents. To do this, simply adding `!` before the color.
///
/// ```rs, no_run
/// c!( #fff); // Color::Srgba(Srgba::new(1.0, 1.0, 1.0, 1.0))
/// c!(!#000); //              Srgba::new(0.0, 0.0, 0.0, 1.0)
/// ```
///
/// # Grammar
///
/// ```txt
/// c ::= '!'? color;
///
/// color ::=
///   | '#' + hex{3}    // #rgb
///   | '#' + hex{4}    // #rgba
///   | '#' + hex{6}    // #rrggbb
///   | '#' + hex{8}    // #rrggbbaa
///   | 'srgb'   '(' number<','>{3, 4} ')'
///   | 'linear' '(' number<','>{3, 4} ')'
///   | 'hsl'    '(' number<','>{3, 4} ')'
///   | 'hsv'    '(' number<','>{3, 4} ')'
///   | 'hwb'    '(' number<','>{3, 4} ')'
///   | 'lab'    '(' number<','>{3, 4} ')'
///   | 'lch'    '(' number<','>{3, 4} ')'
///   | 'oklab'  '(' number<','>{3, 4} ')'
///   | 'oklch'  '(' number<','>{3, 4} ')'
///   | 'xyz'    '(' number<','>{3, 4} ')'
///   // too many to list here
///   | <<<149 CSS named colors>>>
///   ;
///
/// hex    ::= '0'..'9' | 'a'..'f' | 'A'..'F' ;
/// number ::= INT | FLOAT ;
/// ```
#[proc_macro]
pub fn c(input: TokenStream) -> TokenStream {
  apply::<crate::color::Color>(input, false)
}


/// This macro is used to simplify the creation of the bevy's `UiRect` struct.
///
/// # Syntax
///
/// Within the macro, you can specify 1-4 values separated by space. The values will be used for the
/// top, right, bottom, and left sides of the `UiRect`. Each value can be a `Val` or `_` for default.
/// It basically follows how CSS sides selection works.
///
/// ```rs, no_run
/// e!(10px);                     // all sides are 10px
/// e!(10px 20px);                // top and bottom are 10px, right and left are 20px
/// e!(10px 20px 30px);           // top is 10px, right and left are 20px, bottom is 30px
/// e!(10px 20px 30px 40px);      // top is 10px, right is 20px, bottom is 30px, left is 40px
/// e!(10px 20px 30px 40px 50px); // error, only 4 values are allowed
/// ```
///
/// # Grammar
///
/// * `<TOKEN>*` means repeat 0-inf times separated by `TOKEN`, the last `TOKEN` is optional.
///
/// ```txt
/// e ::= val_or_omit{1,4};
///
/// val_or_omit ::= v | '_';
/// ```
#[proc_macro]
pub fn e(input: TokenStream) -> TokenStream {
  apply::<crate::edges::Edges>(input, false)
}


trait Generate {
  fn generate(&self) -> proc_macro2::TokenStream;

  fn generate_default() -> proc_macro2::TokenStream {
    unimplemented!()
  }
}


fn apply<P: Parse+Generate>(input: TokenStream, allow_empty: bool) -> TokenStream {
  if input.is_empty() {
    if allow_empty {
      return TokenStream::new();
    }

    return Error::new(
      Span::call_site(),
      "This macro can't be called without any input",
    ).to_compile_error().into();
  }

  match P::parse.parse(input) {
    Ok(value) => value.generate(),
    Err(err) => err.to_compile_error(),
  }.into()
}


#[derive(Clone)]
enum MightOmit<T> {
  Value(T),
  Omit(Span),
}

impl<T: Parse> Parse for MightOmit<T> {
  fn parse(input: ParseStream) -> Result<Self> {
    if input.peek(Token![_]) {
      let token = input.parse::<Token![_]>()?;
      return Ok(MightOmit::Omit(token.span));
    }

    Ok(MightOmit::Value(T::parse(input)?))
  }
}

impl<T: Generate> Generate for MightOmit<T> {
  fn generate(&self) -> proc_macro2::TokenStream {
    match self {
      MightOmit::Value(value) => value.generate(),
      MightOmit::Omit(_) => T::generate_default(),
    }
  }

  fn generate_default() -> proc_macro2::TokenStream {
    T::generate_default()
  }
}
