# bevy-toolbox

A simple crate that provides macros for simplifying some common Bevy tasks.

## `spawn!`

This macro is used to simplify the entity creation of the bevy engine.

## Example

### To create a single entity with a transform component:

```rs
spawn! { commands // world, or anything that have `spawn` method which returns `EntityCommands`
  (Transform::default())
}
```

### To create a single entity with a button component with a text child:

```rs
spawn! { commands
  (Button)
    .[ // children
      (Text::new("Hello, World!"))
    ];
}
```

### To create a single entity with a button component with a text child, print `Hello, World!` when
clicked:

```rs
spawn! { commands
  (Button)
    .[ // children
      (Text::new("Hello, World!"))
    ]
    .observe(|_: Trigger<Pointer<Click>>| { println!("Hello, World!"); });
}
```

### Spawn multiple entities:

```rs
spawn! { commands
  (Button);
  (Button);
  (Button);
}
```

### Spawn children for existing entity:

* Assuming the parent `Entity` called `uwu` and already spawned.

```rs
spawn! { commands
  uwu > (Button);
}
```

### Only top level entity can have parent:

```rs
spawn! { commands
  uwu (Button);

  uwu > (Button); // ok

  (Button)
    .[
      uwu > (Button); // error
    ]
}
```

### Name a entity and reference it later:

```rs
spawn! { commands
  owo (Text::new("Hello, World!"));
  uwu (Button);

  uwu > (Button);  // another way of spawning children of `uwu`

  (Button)
    .observe(move |_: Trigger<Pointer<Click>>, mut commands: Commands| {
      commands.entity(owo).insert(Text::new("This is new text!"));
    });
}
```

### Reference current entity with `this`:

```rs
spawn! { commands
  (Button, BackgroundColor(Color::srgb(0.0, 0.0, 0.0)))
    .observe(move |_: Trigger<Pointer<Click>>, mut commands: Commands| {
       commands.entity(this).insert(BackgroundColor(Color::srgb(1.0, 1.0, 1.0)));
    });
}
```

### `observe` method name can be omitted:

```rs
spawn! { commands
  (Button)
    .(|_: Trigger<Pointer<Click>>| { println!("Hello, World!"); });
}
```

### Embed code block:

```rs
spawn! { commands
  // code block in top level
  entity_a (Button);

  {
    // injecting code block between the spawning of `entity_a` and `entity_b`
    println!("This is inside a code block!");
    println!("{entity_a:?}");
  };

  entity_b (Button); // order matters, previous code block can't access `entity_b`

  // code block as extension
  (Button)
    .{ /* you can also inject code block when defining entity, don't forget the `.` */ }
    .{ /* still, order does matter, this will be execute after the first one */ }

    .{
      // previously mentioned `this` exposes the current `Entity`
      // here `entity` exposes the current `EntityCommands`
    }

    // normal observe method
    .(|_: Trigger<Pointer<Click>>| { println!("Hello, World!"); })

    // normal children definitions
    .[ (Text::new("Hello, World!")) ];

  // code block in children extension
  (Button)
    .[
      // you can also inject code block here too
      // because children group is designed to be enclosed
      // and will not leak the children to the ancestors
      uwu (Text::new("Hello, World!"));

      // no problem
      { println!("{uwu:?}"); }
    ]
    .[
      // `uwu` does not accessible here
      { println!("{uwu:?}"); }
    ];

   // `uwu` also does not accessible here
   { println!("{uwu:?}"); }
}
```