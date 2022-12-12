use std::str::FromStr;

use iced_wgpu::{Renderer, Theme};
use iced_winit::{
    alignment,
    theme::ProgressBar,
    widget::{progress_bar::StyleSheet, Column, Container, Text},
    Color, Command, Element, Length, Program,
};
use palette::Srgb;
use tracing::error;

use crate::{config::Configuration, events::KaraEvent};

pub struct Controls {
    background_color: Color,
    foreground_color: Color,
    padding: u16,
    text: String,
    font_size: u16,
    progress_bar: ProgressBarData,
}

struct ProgressBarData {
    progress: f32,
    background_color: Color,
    foreground_color: Color,
    corner_radius: f32,
    height: u16,
}

impl ProgressBarData {
    fn new(
        background_color: &str,
        foreground_color: &str,
        corner_radius: f32,
        height: u16,
    ) -> Self {
        let (bg_r, bg_g, bg_b) = map_colour(background_color, ColourType::Background);
        let (fg_r, fg_g, fg_b) = map_colour(foreground_color, ColourType::Foreground);
        Self {
            progress: 100.0,
            background_color: Color {
                r: bg_r,
                g: bg_g,
                b: bg_b,
                a: 1.0,
            },
            foreground_color: Color {
                r: fg_r,
                g: fg_g,
                b: fg_b,
                a: 1.0,
            },
            corner_radius,
            height,
        }
    }

    fn update_progress(&mut self, new_value: f32) {
        self.progress = new_value;
    }

    fn update_styles(
        &mut self,
        background_color: &str,
        foreground_color: &str,
        corner_radius: f32,
        height: u16,
    ) {
        let (bg_r, bg_g, bg_b) = map_colour(background_color, ColourType::Background);
        let (fg_r, fg_g, fg_b) = map_colour(foreground_color, ColourType::Foreground);
        self.background_color = Color {
            r: bg_r,
            g: bg_g,
            b: bg_b,
            a: 1.0,
        };
        self.foreground_color = Color {
            r: fg_r,
            g: fg_g,
            b: fg_b,
            a: 1.0,
        };
        self.corner_radius = corner_radius;
        self.height = height;
    }
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
            progress_bar: ProgressBarData::new(
                &config.colours.progressbar_background,
                &config.colours.progressbar_foreground,
                config.window.progress_bar.corner_radius,
                config.window.progress_bar.height,
            ),
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
                self.progress_bar.update_styles(
                    &config.colours.progressbar_background,
                    &config.colours.progressbar_foreground,
                    config.window.progress_bar.corner_radius,
                    config.window.progress_bar.height,
                );
            }
            KaraEvent::ReadingSpeech(text) | KaraEvent::FinalisedSpeech(text) => self.text = text,
            KaraEvent::UpdateProgressBar(new_progress) => {
                self.progress_bar.update_progress(new_progress);
            }
            _ => {}
        }
        Command::none()
    }

    fn view(&self) -> Element<KaraEvent, Renderer> {
        let content = Column::new()
            .spacing(100)
            .push(
                Text::new(&self.text)
                    .style(self.foreground_colour())
                    .size(self.font_size),
            )
            .align_items(iced_winit::Alignment::Center);
        let style: Box<dyn StyleSheet<Style = Theme>> = Box::new(MyProgressbarStyle {
            background: self.progress_bar.background_color,
            bar: self.progress_bar.foreground_color,
            radius: self.progress_bar.corner_radius,
        });
        Container::new(if self.progress_bar.progress < 100.0 {
            content.push(
                Column::new()
                    .push("Downloading resourses. Please wait...")
                    .spacing(2)
                    .push(
                        iced_winit::widget::progress_bar(0.0..=100.0, self.progress_bar.progress)
                            .height(Length::Units(self.progress_bar.height))
                            .style(ProgressBar::Custom(style)),
                    ),
            )
        } else {
            content
        })
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

struct MyProgressbarStyle {
    background: Color,
    bar: Color,
    radius: f32,
}

impl StyleSheet for MyProgressbarStyle {
    type Style = iced_winit::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced_winit::widget::progress_bar::Appearance {
        iced_winit::widget::progress_bar::Appearance {
            background: iced_winit::Background::Color(self.background),
            bar: iced_winit::Background::Color(self.bar),
            border_radius: self.radius,
        }
    }
}
