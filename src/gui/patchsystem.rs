use crate::midi;

use iced::widget::{button, row, column, text};
use iced::{Application, Command, Theme, Element, Alignment};
use iced::executor;
use iced::Subscription;
use iced_native::{window, mouse, Event, Length, alignment};
use iced::window::set_mode;
use std::time::Duration;
use iced::time;

pub struct PatchSystem {
    device: midi::ThruDevice,
    screen_height: u32,
    screen_width: u32,
    show_buttons: bool,
    mouse_down: bool,
    exit: bool
}

#[derive(Debug, Clone)]
pub enum Message {
    NextPatch,
    PreviousPatch,
    ResetPatch,
    QuitApplication,
    MouseHeld,
    EventOccurred(iced_native::Event)
}

impl Application for PatchSystem {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = midi::ThruDevice;
    type Theme = Theme;

    fn new(device: Self::Flags) -> (Self, Command<Message>) {
        (Self {
            device,
            screen_height: 100,
            screen_width: 100,
            show_buttons: false,
            exit: false,
            mouse_down: false
        }, set_mode(window::Mode::Fullscreen))
    }

    fn title(&self) -> String {
        String::from("MIDI Patch Changer") //TODO include input/output device names & patch list name?
    }

    fn view(&self) -> Element<Message> {
        let small = (self.screen_height / 10) as u16; // 1/10 of screen height
        let big = (self.screen_height as u16 - 2 * small) / 3; // space for 3 lines of text
        let top_text = if self.show_buttons {
            "Click to exit menu"
        } else {
            match self.device.previous_patch() { Some(patch) => &patch.name, None => "" }
        };
        let top = text(top_text)
            .size(small)
            .height(Length::Units(small))
            .horizontal_alignment(alignment::Horizontal::Center);
        let middle = text(match self.device.current_patch() { Some((number, patch)) => format!("#{} {}", number, patch.name), None => String::from("No Patches")})
            .size(big)
            .height(Length::Fill)
            .vertical_alignment(alignment::Vertical::Center)
            .horizontal_alignment(alignment::Horizontal::Center);
        let bottom = if self.show_buttons {
            let button_text = small / 2;
            row![
                button(text("Next Patch").size(button_text).horizontal_alignment(alignment::Horizontal::Center).vertical_alignment(alignment::Vertical::Center))
                    .on_press(Message::NextPatch)
                    .width(Length::Fill),
                button(text("Previous Patch").size(button_text).horizontal_alignment(alignment::Horizontal::Center).vertical_alignment(alignment::Vertical::Center))
                    .on_press(Message::PreviousPatch)
                    .width(Length::Fill),
                button(text("Reset").size(button_text).horizontal_alignment(alignment::Horizontal::Center).vertical_alignment(alignment::Vertical::Center))
                    .on_press(Message::ResetPatch)
                    .width(Length::Fill),
                button(text("QUIT").size(button_text).horizontal_alignment(alignment::Horizontal::Center).vertical_alignment(alignment::Vertical::Center))
                    .on_press(Message::QuitApplication)
                    .width(Length::Fill)
            ]
            .spacing(10)
            .height(Length::Units(small))
        } else {
            row![
                text(match self.device.next_patch() { Some(patch) => &patch.name, None => ""})
                    .size(small)
                    .height(Length::Units(small))
                    .width(Length::Fill)
                    .horizontal_alignment(alignment::Horizontal::Center)
            ]
        };
        column![top, middle, bottom]
            .padding(10)
            .align_items(Alignment::Fill)
            .into()
    }

    fn should_exit(&self) -> bool {
        self.exit
    }

    fn subscription(&self) -> Subscription<Message> {
        let events = iced::subscription::events().map(Message::EventOccurred);
        if self.mouse_down {
            let timeout = time::every(Duration::from_millis(1500)).map(|_| Message::MouseHeld);
            Subscription::batch([events, timeout])
        } else {
            events
        }
    }

    fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::NextPatch => {
                self.device.increment_patch(1);
            },
            Message::PreviousPatch => {
                self.device.increment_patch(-1);
            },
            Message::ResetPatch => {
                self.device.set_patch(0);
            },
            Message::QuitApplication => self.exit = true,
            Message::MouseHeld => {
                if self.mouse_down {
                    self.mouse_down = false;
                    self.show_buttons = !self.show_buttons;
                }
            },
            Message::EventOccurred(event) => {
                match event {
                    Event::Window(window::Event::Resized { width, height }) => {
                        self.screen_width = width;
                        self.screen_height = height;
                    },
                    Event::Mouse(mouse) => {
                        match mouse {
                            mouse::Event::ButtonPressed(_) => {
                                self.mouse_down = true;
                            },
                            mouse::Event::ButtonReleased(button) => {
                                if self.mouse_down {
                                    self.mouse_down = false;
                                    if self.show_buttons {
                                        self.show_buttons = false;
                                    } else {
                                        match button {
                                            mouse::Button::Left => {
                                                self.device.increment_patch(1);
                                            },
                                            mouse::Button::Right => {
                                                self.device.increment_patch(-1);
                                            },
                                            _ => self.show_buttons = true
                                        }
                                    }
                                }
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
            }
        }
        Command::none()
    }
}