use std::str::FromStr;

use iced_wgpu::Renderer;
use iced_winit::widget::{Column, Container, Text};
use iced_winit::{alignment, Color, Command, Element, Length, Program};
use palette::Srgb;
use tracing::error;

use crate::config::Configuration;
use crate::events::KaraEvent;

pub struct Controls {
    background_color: Color,
    foreground_color: Color,
    padding: u16,
    text: String,
    font_size: u16,
}

impl Controls {
    pub fn new(config: &Configuration) -> Controls {
        let (bg_r, bg_g, bg_b) = map_colour(&config.colours.background, ColourType::Background);
        let opacity = config.window.opacity;
        let (fg_r, fg_g, fg_b) = map_colour(&config.colours.foreground, ColourType::Foreground);

        Controls {
            background_color: Color {
                r: bg_r,
                g: bg_g,
                b: bg_b,
                a: opacity,
            },
            text: String::from("Hello there!"),
            foreground_color: Color {
                r: fg_r,
                g: fg_g,
                b: fg_b,
                a: 1.0,
            },
            padding: config.window.padding,
            font_size: config.window.font_size,
        }
    }

    pub fn background_colour(&self) -> Color {
        self.background_color
    }

    pub fn foreground_colour(&self) -> Color {
        self.foreground_color
    }
}

impl Program for Controls {
    type Renderer = Renderer;
    type Message = KaraEvent;

    fn update(&mut self, message: KaraEvent) -> Command<KaraEvent> {
        match message {
            KaraEvent::ReloadConfiguration(config) => {
                let (bg_r, bg_g, bg_b) =
                    map_colour(&config.colours.background, ColourType::Background);
                let opacity = config.window.opacity;
                let (fg_r, fg_g, fg_b) =
                    map_colour(&config.colours.foreground, ColourType::Foreground);

                self.background_color = Color {
                    r: bg_r,
                    g: bg_g,
                    b: bg_b,
                    a: opacity,
                };

                self.foreground_color = Color {
                    r: fg_r,
                    g: fg_g,
                    b: fg_b,
                    a: 1.0,
                };
                self.padding = config.window.padding;
                self.font_size = config.window.font_size;
            }
            KaraEvent::Close => {}
            _ => {}
        }
        Command::none()
    }

    fn view(&self) -> Element<KaraEvent, Renderer> {
        let content = Column::new().push(
            Text::new(&self.text)
                .style(self.foreground_colour())
                .size(self.font_size),
        );
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Bottom)
            .padding(self.padding)
            .into()
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ColourType {
    Background,
    Foreground,
}

pub(crate) fn map_colour(colours: &str, colour_type: ColourType) -> (f32, f32, f32) {
    match Srgb::from_str(colours) {
        Ok(rgb) => (to_float(rgb.red), to_float(rgb.green), to_float(rgb.blue)),
        Err(e) => {
            error!(value = colours, "{e}");
            match colour_type {
                ColourType::Background => (0.0, 0.0, 0.0),
                ColourType::Foreground => (1.0, 1.0, 1.0),
            }
        }
    }
}

fn to_float(val: u8) -> f32 {
    val as f32 / 255.0
}
