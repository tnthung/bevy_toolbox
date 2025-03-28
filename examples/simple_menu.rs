use bevy::prelude::*;
use bevy_toolbox::*;
use bevy::{ecs::system::IntoObserverSystem, window::PrimaryWindow, winit::cursor::CursorIcon};


fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_systems(Startup, setup)
    .run();
}


fn setup(mut commands: Commands) {
  spawn! { [commands.reborrow()]
    (Camera2d);

    // Create a container that will center everything
    container (Node {
      width          : v!(100vw),
      height         : v!(100vh),
      align_items    : AlignItems::Center,
      justify_content: JustifyContent::Center,
      ..Default::default()
    });

    // Create a container for the buttons
    container > (
      Node {
        width          : v!(250px),
        padding        : e!(10px),
        flex_direction : FlexDirection::Column,
        row_gap        : v!(10px),
        align_items    : AlignItems::Stretch,
        justify_content: JustifyContent::Center,
        ..Default::default()
      },
      BorderRadius::all(v!(15px)),
      BackgroundColor(c!(#006A71)),
    ).[
      new_game  ();
      load_game ();
      settings  ();

      // spacer
      (Node {
        height: v!(20px),
        ..Default::default()
      });

      exit_game ();

      {
        type T<'w> = Trigger<'w, Pointer<Click>>;

        new_button(commands.reborrow(), new_game , "New Game" , |_: T| { println!("New Game" ); });
        new_button(commands.reborrow(), load_game, "Load Game", |_: T| { println!("Load Game"); });
        new_button(commands.reborrow(), settings , "Settings" , |_: T| { println!("Settings" ); });
        new_button(commands.reborrow(), exit_game, "Exit Game", |_: T, mut ew: EventWriter<AppExit>| {
          println!("Exit Game");
          ew.send(AppExit::Success);
        });
      };
    ];
  }
}


fn new_button<B: Bundle, M>(mut commands: Commands, entity: Entity, text: impl AsRef<str>, cb: impl IntoObserverSystem<Pointer<Click>, B, M>) {
  const ICON_DEFAULT : CursorIcon = CursorIcon::System(bevy::window::SystemCursorIcon::Default);
  const ICON_HOVER   : CursorIcon = CursorIcon::System(bevy::window::SystemCursorIcon::Pointer);
  const COLOR_DEFAULT: Color = c!( #48A6A7);
  const COLOR_HOVER  : Color = c!( #9ACBD0);
  const COLOR_ACTIVE : Color = c!( #48A6A7);

  fn change_background_color<E: Event>(color: Color) -> impl FnMut(Trigger<E>, Commands) {
    move |t, mut cmds| { cmds.entity(t.entity()).insert(BackgroundColor(color)); }
  }

  fn change_cursor_icon<E: Event>(cursor: CursorIcon) -> impl FnMut(Trigger<E>, Commands, Single<Entity, With<PrimaryWindow>>) {
    move |_, mut cmds, win| { let cursor = cursor.clone(); cmds.entity(win.into_inner()).insert(cursor); }
  }

  spawn! {commands
    entity + (
      Button,
      BorderRadius::all(v!(5px)),
      BackgroundColor(COLOR_DEFAULT),
      Node {
        padding        : e!(10px),
        justify_content: JustifyContent::Center,
        ..Default::default()
      },
    )
      // Add a click event to the button
      .(cb)
      // Some fancy button styling
      .(change_cursor_icon     ::<Pointer<Over>>(ICON_HOVER   ))
      .(change_cursor_icon     ::<Pointer<Out >>(ICON_DEFAULT ))
      .(change_background_color::<Pointer<Over>>(COLOR_HOVER  ))
      .(change_background_color::<Pointer<Out >>(COLOR_DEFAULT))
      .(change_background_color::<Pointer<Down>>(COLOR_ACTIVE ))
      .(change_background_color::<Pointer<Up  >>(COLOR_HOVER  ))
      // Add a text to the button
      .[(Text::new(text.as_ref()))];
  }
}
