use horizon_mod_sdk::hud::{self, Corner, Line};
use horizon_mod_sdk::{export_mod, game, Mod, Registration};

/// Shows "XYZ: x / y / z" in the top-left corner while a world is loaded.
struct XyzCoords;

impl Mod for XyzCoords {
    fn init() -> Result<(Self, Registration), String> {
        // Ask the game to call `tick` every game tick.
        Ok((XyzCoords, Registration::new().on_tick()))
    }

    fn tick(&mut self, _tick: u64) {
        match game::get_player() {
            Some(player) => {
                let text = format!(
                    "XYZ: {:.1} / {:.1} / {:.1}",
                    player.x, player.y, player.z
                );
                hud::set_panel(Corner::TopLeft, &[Line { text, color: 0xFFFFFF }]);
            }
            // No player (main menu, loading): hide the panel.
            None => hud::set_panel(Corner::TopLeft, &[]),
        }
    }
}

export_mod!(XyzCoords);
