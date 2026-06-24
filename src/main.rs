use iced::font;
use iced::widget::{center_x, column, container, row, scrollable, table, text};
use iced::{Center, Element, Fill, Font, Task, Theme};

pub fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .theme(Theme::CatppuccinMocha)
        .run()
}

struct App {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    file_name: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    OpenFile,
    FileLoaded(Option<(String, Vec<String>, Vec<Vec<String>>)>),
}

fn pick_and_load_csv() -> Option<(String, Vec<String>, Vec<Vec<String>>)> {
    let path = rfd::FileDialog::new()
        .add_filter("CSV", &["csv", "tsv"])
        .pick_file()?;

    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let file = std::fs::File::open(&path).ok()?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let headers: Vec<String> = rdr
        .headers()
        .map(|h| h.iter().map(|s| s.to_string()).collect())
        .unwrap_or_default();

    let rows: Vec<Vec<String>> = rdr
        .records()
        .filter_map(|r| r.ok())
        .map(|r| r.iter().map(|s| s.to_string()).collect())
        .collect();

    Some((file_name, headers, rows))
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                headers: Vec::new(),
                rows: Vec::new(),
                file_name: None,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenFile => {
                // 用 std::future::ready 包装同步结果
                Task::perform(load_csv_task(), Message::FileLoaded)
            }
            Message::FileLoaded(Some((name, headers, rows))) => {
                self.file_name = Some(name);
                self.headers = headers;
                self.rows = rows;
                Task::none()
            }
            Message::FileLoaded(None) => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let toolbar = container(
            row![
                iced::widget::button("Open CSV").on_press(Message::OpenFile),
                text(self.file_name.as_deref().unwrap_or("No file loaded")),
            ]
            .spacing(20)
            .align_y(Center),
        )
        .padding(10)
        .width(Fill);

        let statusbar = container(
            text(format!(
                "Rows: {} | Columns: {}",
                self.rows.len(),
                self.headers.len()
            ))
            .size(12),
        )
        .padding(8)
        .width(Fill)
        .style(container::dark);

        let content: Element<'_, Message> = if self.headers.is_empty() {
            center_x(
                column![
                    text("No CSV loaded").size(24),
                    text("Click \"Open CSV\" to get started"),
                ]
                .spacing(10)
                .align_x(Center),
            )
            .center_y(Fill)
            .into()
        } else {
            let columns: Vec<_> = self
                .headers
                .iter()
                .enumerate()
                .map(|(i, header)| {
                    let header_text = text(header.clone()).font(Font {
                        weight: font::Weight::Bold,
                        ..Font::DEFAULT
                    });
                    table::column(header_text, move |row: &Vec<String>| {
                        text(row.get(i).cloned().unwrap_or_default())
                    })
                })
                .collect();

            let csv_table = table(columns, &self.rows)
                .padding_x(10.0)
                .padding_y(5.0)
                .separator_x(1.0)
                .separator_y(1.0);

            center_x(scrollable(center_x(csv_table)).spacing(10)).into()
        };

        column![toolbar, content, statusbar].into()
    }
}

async fn load_csv_task() -> Option<(String, Vec<String>, Vec<Vec<String>>)> {
    pick_and_load_csv()
}
