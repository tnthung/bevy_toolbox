use bevy::prelude::*;
use bevy_toolbox::*;


fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_systems(Startup, setup)
    .run();
}


fn setup(mut commands: Commands) {
  spawn! { commands
    // Add camera
    (Camera2d);

    // Create a container to center the grid
    container (Node {
      width          : v!(100vw),
      height         : v!(100vh),
      justify_content: JustifyContent::Center,
      align_items    : AlignItems::Center,
      ..Default::default()
    });

    // Create a grid container
    container > grid (Node {
      flex_direction : FlexDirection::Column,
      justify_content: JustifyContent::Center,
      align_items    : AlignItems::Center,
      row_gap        : v!(10px),
      ..Default::default()
    });

    // Create 4 rows
    for row in 0..4 {
      // Create a row container
      grid > (Node {
        flex_direction: FlexDirection::Row,
        column_gap    : v!(10px),
        ..Default::default()
      }).[
        // Create 10 buttons in this row
        for col in 0..10 {
          ().{ create_stylish_button(entity, row * 10 + col); };
        }
      ];
    }
  }
}


fn create_stylish_button(mut entity: EntityCommands, index: usize) {
  let base = entity.id();

  spawn! { [entity.commands()]
    base + (
      Button,
      BorderRadius::all(v!(5px)),
      BackgroundColor(c!(#0477BF)),
      Node {
        width          : v!(8.5vw),
        padding        : e!(10px),
        justify_content: JustifyContent::Center,
        align_items    : AlignItems::Center,
        ..Default::default()
      },
    )
      // Add a click event to the button
      .(move |_: Trigger<Pointer<Click>>| {
        println!("Button {index} is clicked");
      })
      // Some fancy button styling
      .(change_background_color::<Pointer<Over>>(c!(#049DD9)))
      .(change_background_color::<Pointer<Out >>(c!(#0477BF)))
      .(change_background_color::<Pointer<Down>>(c!(#04B2D9)))
      .(change_background_color::<Pointer<Up  >>(c!(#049DD9)))
      // Add text to the button
      .[(
        Text::new(format!("Button {index}")),
        TextFont::default().with_font_size(16.0),
        TextLayout::new_with_justify(JustifyText::Center),
      )];
  }
}

fn change_background_color<E: Event>(color: Color) -> impl FnMut(Trigger<E>, Commands) {
  move |t, mut cmds| { cmds.entity(t.entity()).insert(BackgroundColor(color)); }
}
