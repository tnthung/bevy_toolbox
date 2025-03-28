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
    (Camera2d);

    // Create a container that will center the button
    container (Node {
      width: v!(100vw),
      height: v!(100vh),
      align_items: AlignItems::Center,
      justify_content: JustifyContent::Center,
      ..Default::default()
    });

    // Create a button as a child of the container
    container > simple_button (
      Button,
      BorderRadius::all(v!(5px)),
      BackgroundColor(c!(#0477BF)),
      Node { padding: e!(10px), ..Default::default() },
    )
      // Add a click event to the button
      .(|_: Trigger<Pointer<Click>>| { println!("Hello, World!"); })
      // Some fancy button styling
      .(change_background_color::<Pointer<Over>>(c!(#049DD9)))
      .(change_background_color::<Pointer<Out >>(c!(#0477BF)))
      .(change_background_color::<Pointer<Down>>(c!(#04B2D9)))
      .(change_background_color::<Pointer<Up  >>(c!(#049DD9)))
      // Add a text to the button
      .[(Text::new("Click me!"))];

    { // Acknowledge the button has been spawned
      println!("Button {simple_button:?} spawned!");
    };
  }
}


fn change_background_color<E: Event>(color: Color) -> impl FnMut(Trigger<E>, Commands) {
  move |t, mut cmds| { cmds.entity(t.entity()).insert(BackgroundColor(color)); }
}
