# bevy-toolbox

A simple crate that provides macros for simplifying some common Bevy tasks.

Table of Contents:

- [spawn!](#spawn)
- [v!](#v)
- [c!](#c)

# `spawn!`

This macro is used to simplify the entity creation of the bevy engine.

## Spawner

Spawner is the object that have `spawn` method which takes a bevy bundle and returns
`EntityCommands` as the result.

```rs
fn foo(mut commands: Commands) {
  // commands can be used as spawner
}

fn bar(world: &mut World) {
  // world can be used as spawner
}
```

## Top level

Top level means the part of the macro thats been directly quoted by the macro itself.

```rs
spawn! { commands
  // here is the top level

  ().[
    // here is not the top level
  ];
}
```

## Entity definition

An entity definition is a tuple of components that will be spawned as an entity.

```rs
spawn! { commands
  // entity definition
  (Button, Node::default());
}
```

Top level can accept multiple entity definitions.

```rs
spawn! { commands
  // entity 1
  (Button, Node::default());

  // entity 2
  (Button, Node::default());
}
```

## Order

The order of any bit in the macro matters. The execution order is strictly follow the macro input.

```rs
// entity `a` will always being spawned before `b`
spawn! { commands
  a ();
  b ();
}
```

## Naming

An entity can be named for later reference. The variable will hold the `Entity` of the corresponding
entity, NOT THE `EntityCommands`.

```rs
spawn! { commands
  entity_1 (Button, Node::default());
  entity_2 (Button, Node::default());

  (Button)
    .(move |_: Trigger<Pointer<Click>>, mut commands: Commands| {
      // referencing the entity_1 after it's been spawned
      commands.entity(entity_1).despawn();
    });

  {
    println!("{entity_1:?}");
    println!("{entity_2:?}");
  };
}
```

## Parenting

A top level entities can have explicit parent. Parenting is done by using `>` operator.

```rs
spawn! { commands
  my_entity (Button);

  // this entity will be spawned as a child of `my_entity`
  my_entity > (Button);

  // it's also possible to use the entity outside the macro
  // just make sure the parent is `Entity` type
  some_outside_entity > (Button);

  // parenting and naming can be combined
  parent > child (Button);
}
```

## Insertion

Insertion is a way to add some components to an existing entity. The entity must be named and spawned
in advanced in order to be referenced.

```rs
spawn! { commands
  my_entity (Button);

  // add a background color to `my_entity`
  my_entity + (BackgroundColor(Color::srgb(0.0, 0.0, 0.0)));

  // in the children group, it's also possible to insert components
  my_fancy_button (Button).[
    txt (Text::new("Hello, World!"));

    // add a background color to `txt`
    txt + (BackgroundColor(Color::srgb(0.0, 0.0, 0.0)));
  ];

  // extensions are still available
  my_fancy_button + (BackgroundColor(Color::srgb(0.0, 0.0, 0.0)))
    .(move |_: Trigger<Pointer<Click>>, mut commands: Commands| { /* ... */ });
}
```

## Code block injection

Since the entities inside the macro is enclosed within a generated scope to prevent the namespace
pollution, code block injection makes it possible to execute code without leaving the macro.

```rs
spawn! { commands
  entity_1 (Button);
  entity_2 (Button);

  {
    println!("This is inside a code block!");
    println!("{entity_1:?}");
    println!("{entity_2:?}");

    // you can do whatever you want here, just make sure the ownership of spawner will not be taken
    // if you want to spawn any entities after this code block
  };
}
```

## Extension

An entity can be extended with any number of:

1. Method Call
1. Code Block

All extensions are started with `.` after the entity definition.

### Method Call

Method call is a call to a method of `EntityCommands`. The auto completion is supported for the
method name and the arguments.

```rs
spawn! { commands
  (Button)
    .observe(|_: Trigger<Pointer<Click>>| { println!("Hello, World!"); })
    .[(Text::new("Hello, World!"))];
}
```

Since `observe` is most likely to be used, a shortcut is provided to omit the method name.

```rs
spawn! { commands
  (Button)
    .(|_: Trigger<Pointer<Click>>| { println!("Hello, World!"); })
    .[(Text::new("Hello, World!"))];
}
```

To reference the current entity, you can use `this` for `Entity` and `entity` for `EntityCommands`.

```rs
spawn! { commands
  (Button, BackgroundColor(Color::srgb(0.0, 0.0, 0.0)))
    .(move |_: Trigger<Pointer<Click>>, mut commands: Commands| {
      commands.entity(this).insert(BackgroundColor(Color::srgb(1.0, 1.0, 1.0)));
    })
    .[(Text::new("Hello, World!"))];
}
```

### Code Block

Code block is a block of code that will be executed in the context of the entity. As previously
mentioned, the code block can also access `this` and `entity` variables.

```rs
spawn! { commands
  (Button)
    .{
      // print the entity id of the current entity
      println!("{this:?}");

      // manually adding a child
      entity.with_child((Text::new("Hello, World!")));
    };
}
```

## Children Group

Children group is a group of entities quoted by `[]` after the `.`. The entities in the group will
be spawned as children of the parent entity. One entity can have multiple children groups, but all
of them have to be after the extensions. This is because the `spawner` ownership will be temporarily
taken for method calls and code blocks, to prevent this from happening, the children group is forced
to be the last part of the entity definition. Within the same group, the entities can reference
each other, but entities in 2 different groups under same parent can't.

```rs
spawn! { commands
  (Button)
    .[
      a (Text::new("Hello, World!"));
      b (Text::new("Hello, World!"));

      {
        // code block injection is also possible
        // you can access `a` and `b` here
      };
    ]
    .[
      c (Text::new("Hello, World!"));

      {
        // you can't access `a`, `b`, but `c`
      };
    ];
}
```


# `v!`

This macro is used to simplify the creation of the bevy's `Val` enum.

## Syntax

There are 7 variants of the `Val` enum, so 7 corresponding syntaxes are provided.

* `+` in between tokens means there can be no space between them.

1. `Val::Auto`    - `auto`, `@`
1. `Val::Percent` - `number '%'` *space is optional* (e.g. `10%`)
1. `Val::Px`      - `number + 'px'` (e.g. `10px`)
1. `Val::Vw`      - `number + 'vw'` (e.g. `10vw`)
1. `Val::Vh`      - `number + 'vh'` (e.g. `10vh`)
1. `Val::Vmin`    - `number + 'vmin'` (e.g. `10vmin`)
1. `Val::Vmax`    - `number + 'vmax'` (e.g. `10vmax`)

```rs
v!(auto);
v!(@);
v!(10%);
v!(10px);
v!(10 vw); // space not allowed, error will be thrown
```


# `c!`

This macro is used to simplify the creation of the bevy's `Color` enum. The syntax is mimicking
the CSS color syntax. The macro fully supports the auto completion for the color names and the
color spaces.

## Syntax

### Hex

Hex colors are color codes that starts with `#` followed by 3, 4, 6, or 8 hex characters. The digits
are not case sensitive, so `#fff` is equivalent to `#FFF`. The color will be converted to `srgb`
color space.

* 3 hex digits - `#rgb`       (equivalent to `#rrggbb`)
* 4 hex digits - `#rgba`      (equivalent to `#rrggbbaa`)
* 6 hex digits - `#rrggbb`
* 8 hex digits - `#rrggbbaa`

```rs
c!(#fff);
c!(#62a7ff);
```

### Functional Notation

All functional notations can have 3 or 4 arguments. The 4th argument is the alpha channel, if not
provided, it will be set to `1.0`.

* `srgb`   - Standard RGB color space
* `linear` - Linear RGB color space
* `hsl`    - Hue, Saturation, Lightness
* `hsv`    - Hue, Saturation, Value
* `hwb`    - Hue, Whiteness, Blackness
* `lab`    - L\*a\*b\* color space
* `lch`    - Luminance, Chroma, Hue
* `oklab`  - Oklab color space
* `oklch`  - Oklch color space
* `xyz`    - CIE 1931 XYZ color space

With the function selected, follow it with the arguments in the parentheses.

```rs
c!(srgb(1.0, 0.5, 0.5));
c!(linear(1.0, 0.5, 0.5));
```

### CSS Named Colors

There are 149 named colors in CSS. The auto completion is supported for all of them. The color will
be converted to `srgb` color space.

```rs
c!(firebrick);
c!(darkolivegreen);
```

### No wrap

The `c!` macro by default will wrap the color with `Color` enum, but sometimes you might just want
the inner value that color represents. To do this, simply adding `!` before the color.

```rs, no_run
c!( #fff); // Color::Srgba(Srgba::new(1.0, 1.0, 1.0, 1.0))
c!(!#000); //              Srgba::new(0.0, 0.0, 0.0, 1.0)
```
