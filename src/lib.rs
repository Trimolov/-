use horizon_mod_sdk::hud::{self, Corner, Line};
use horizon_mod_sdk::{export_mod, game, storage, Mod, Registration};

/// Screen corners a block of the HUD can sit in.
#[derive(Clone, Copy, PartialEq)]
enum Pos {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

const ALL: [Pos; 4] = [Pos::TopLeft, Pos::TopRight, Pos::BottomLeft, Pos::BottomRight];

impl Pos {
    fn corner(self) -> Corner {
        match self {
            Pos::TopLeft => Corner::TopLeft,
            Pos::TopRight => Corner::TopRight,
            Pos::BottomLeft => Corner::BottomLeft,
            Pos::BottomRight => Corner::BottomRight,
        }
    }

    fn index(self) -> usize {
        match self {
            Pos::TopLeft => 0,
            Pos::TopRight => 1,
            Pos::BottomLeft => 2,
            Pos::BottomRight => 3,
        }
    }

    fn code(self) -> &'static str {
        match self {
            Pos::TopLeft => "tl",
            Pos::TopRight => "tr",
            Pos::BottomLeft => "bl",
            Pos::BottomRight => "br",
        }
    }

    fn parse(s: &str) -> Option<Pos> {
        match s {
            "tl" => Some(Pos::TopLeft),
            "tr" => Some(Pos::TopRight),
            "bl" => Some(Pos::BottomLeft),
            "br" => Some(Pos::BottomRight),
            _ => None,
        }
    }
}

/// One switchable block of the HUD: on or off, and where it is drawn.
#[derive(Clone, Copy)]
struct Part {
    on: bool,
    pos: Pos,
}

impl Part {
    fn encode(self) -> String {
        format!("{}:{}", if self.on { 1 } else { 0 }, self.pos.code())
    }

    fn decode(s: &str, fallback: Part) -> Part {
        let Some((on, pos)) = s.split_once(':') else {
            return fallback;
        };
        Part {
            on: on == "1",
            pos: Pos::parse(pos).unwrap_or(fallback.pos),
        }
    }
}

/// Lets storage values be read whether the SDK hands back text or bytes.
trait Text {
    fn text(self) -> String;
}

impl Text for String {
    fn text(self) -> String {
        self
    }
}

impl Text for Vec<u8> {
    fn text(self) -> String {
        String::from_utf8_lossy(&self).into_owned()
    }
}

const DEFAULT_FPS: Part = Part { on: true, pos: Pos::TopLeft };
const DEFAULT_COORDS: Part = Part { on: true, pos: Pos::TopLeft };

struct FpsCoords {
    fps: Part,
    coords: Part,
    // FPS measurement: frames and seconds since the last update.
    seconds: f32,
    frames: u32,
    fps_now: u32,
    // What each corner showed last tick, so unchanged panels are not resent.
    last: [Vec<(String, u32)>; 4],
}

impl FpsCoords {
    fn load(&mut self) {
        let Some(raw) = storage::get("config") else {
            return;
        };
        let text = Text::text(raw);
        for item in text.split(';') {
            let Some((key, value)) = item.split_once('=') else {
                continue;
            };
            let part = match key {
                "fps" => &mut self.fps,
                "coords" => &mut self.coords,
                _ => continue,
            };
            *part = Part::decode(value, *part);
        }
    }

    fn save(&self) {
        let text = format!("fps={};coords={}", self.fps.encode(), self.coords.encode());
        let _ = storage::set("config", &text);
    }

    fn status(&self) -> String {
        let show = |p: Part| format!("{} ({})", if p.on { "ON" } else { "OFF" }, p.pos.code());
        format!(
            "FPS {}, Coords {}. Commands: /hud fps, /hud coords, /hud fps tl|tr|bl|br, /hud coords tl|tr|bl|br, /hud reset",
            show(self.fps),
            show(self.coords)
        )
    }
}

fn fps_color(fps: u32) -> u32 {
    // Minecraft chat colours: green, yellow, red.
    if fps >= 60 {
        0x55FF55
    } else if fps >= 30 {
        0xFFFF55
    } else {
        0xFF5555
    }
}

impl Mod for FpsCoords {
    fn init() -> Result<(Self, Registration), String> {
        let mut hud_mod = FpsCoords {
            fps: DEFAULT_FPS,
            coords: DEFAULT_COORDS,
            seconds: 0.0,
            frames: 0,
            fps_now: 0,
            last: Default::default(),
        };
        hud_mod.load();
        Ok((
            hud_mod,
            Registration::new().on_tick().on_frame().command("hud"),
        ))
    }

    // Called for every rendered frame with its duration in seconds.
    fn frame(&mut self, seconds: f32) {
        self.seconds += seconds;
        self.frames += 1;
        if self.seconds >= 0.5 {
            self.fps_now = (self.frames as f32 / self.seconds).round() as u32;
            self.seconds = 0.0;
            self.frames = 0;
        }
    }

    // Called 20 times a second: rebuild the panels and send the ones that changed.
    fn tick(&mut self, _tick: u64) {
        let mut lines: [Vec<(String, u32)>; 4] = Default::default();

        if self.fps.on {
            lines[self.fps.pos.index()].push((
                format!("FPS: {}", self.fps_now),
                fps_color(self.fps_now),
            ));
        }
        if self.coords.on {
            if let Some(player) = game::get_player() {
                lines[self.coords.pos.index()].push((
                    format!("XYZ: {:.1} / {:.1} / {:.1}", player.x, player.y, player.z),
                    0xFFFFFF,
                ));
            }
        }

        for pos in ALL {
            let i = pos.index();
            if lines[i] == self.last[i] {
                continue;
            }
            let panel: Vec<Line> = lines[i]
                .iter()
                .map(|(text, color)| Line {
                    text: text.clone(),
                    color: *color,
                })
                .collect();
            hud::set_panel(pos.corner(), &panel);
            self.last[i] = std::mem::take(&mut lines[i]);
        }
    }

    fn command(&mut self, name: &str, args: &str) -> Result<Option<String>, String> {
        if name != "hud" {
            return Ok(None);
        }
        let mut words = args.split_whitespace();
        let reply = match (words.next(), words.next()) {
            (None, _) => return Ok(Some(self.status())),
            (Some("reset"), _) => {
                self.fps = DEFAULT_FPS;
                self.coords = DEFAULT_COORDS;
                "HUD settings reset.".to_string()
            }
            (Some(which @ ("fps" | "coords")), arg) => {
                let part = if which == "fps" {
                    &mut self.fps
                } else {
                    &mut self.coords
                };
                match arg {
                    None => {
                        part.on = !part.on;
                        format!("{} {}", which, if part.on { "ON" } else { "OFF" })
                    }
                    Some(corner) => match Pos::parse(corner) {
                        Some(pos) => {
                            part.pos = pos;
                            part.on = true;
                            format!("{} moved to {}", which, pos.code())
                        }
                        None => return Err("Corner must be tl, tr, bl or br".to_string()),
                    },
                }
            }
            _ => return Err("Usage: /hud [fps|coords] [tl|tr|bl|br] | /hud reset".to_string()),
        };
        self.save();
        Ok(Some(reply))
    }
}

export_mod!(FpsCoords);
