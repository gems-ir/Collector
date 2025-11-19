use crate::app::utils::{containerized, titled};
use crate::config::Config;
use iced::widget::{checkbox, column, row, text_input};
use iced::{Center, Element, Fill, FillPortion, Task};

#[derive(Default, Debug)]
pub(crate) struct Output {
    is_zip: bool,
    is_zip_pass: bool,
    password: String,
}

#[derive(Debug, Clone)]
pub(crate) enum OutputMsg {
    OnToggledZip(bool),
    OnToggledZipPass(bool),
    InputPassword(String),
}

impl Output {
    pub(crate) fn update(&mut self, action: OutputMsg) -> Task<OutputMsg> {
        match action {
            OutputMsg::OnToggledZip(is_checked) => {
                self.is_zip = is_checked;
                Task::none()
            }
            OutputMsg::OnToggledZipPass(is_checked) => {
                self.is_zip_pass = is_checked;
                Task::none()
            }
            OutputMsg::InputPassword(input) => {
                self.password = input;
                Task::none()
            }
        }
    }
    pub(crate) fn view(&self, _config: &Config) -> Element<'_, OutputMsg> {
        let mut row_zip_checkbox = row![
            checkbox("Zip the destination folder", self.is_zip).on_toggle(OutputMsg::OnToggledZip),
        ];

        if self.is_zip {
            row_zip_checkbox = row_zip_checkbox.push(
                checkbox("Add zip password", self.is_zip_pass).on_toggle(OutputMsg::OnToggledZipPass),
            );
        }

        let mut column_zip = column![
            row_zip_checkbox
                .spacing(10),
        ]
            .align_x(Center)
            .spacing(10);


        if self.is_zip_pass && self.is_zip {
            column_zip = column_zip.push(
                text_input("Enter the zip password...", &self.password)
                    .on_input(OutputMsg::InputPassword)
                    .width(FillPortion(2))
            );
        }

        let lines = column![
            titled("Output"),
            column_zip,
        ]
            .width(Fill)
            .align_x(Center)
            .spacing(10);

        containerized(lines)
    }
}