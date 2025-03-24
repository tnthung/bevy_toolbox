# bevy-toolbox

A simple crate that provides macros for simplifying some common Bevy tasks.

Table of Contents:

- [spawn!](#spawn)

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