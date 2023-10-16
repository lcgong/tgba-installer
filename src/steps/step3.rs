use fltk::{
    app::Sender,
    enums::{Align, Color},
    frame::Frame,
    group::{Flex, Group},
    misc::Progress,
    prelude::{GroupExt, WidgetBase, WidgetExt},
};

use super::super::myapp::{InstallerLogRecord, InstallerLogs};
use super::super::status::LoadingSpinner;
use super::super::status::{DownloadingStats, StatusUpdate};
use super::super::{myapp::Message, style::AppStyle};
use super::utils::format_scale;
use crate::pyenv::Installer;

#[derive(Debug)]
pub enum Step3Message {
    Enter(Installer),
    JobStart,
    JobSuccess,
    JobMessage(String),
    JobProgress(u32, u32),
    JobError(String),
    Downloading {
        title: String,
        total_size: u64,
        percentage: f64,
        speed: f64,
    },
    Done(Installer),
}

pub struct Step3Tab {
    // c_no: usize,
    installer: Option<Installer>,
    panel: Flex,
    sender: Sender<Message>,
    job_progress: Progress,
    job_message: Frame,
    job_percent: Frame,
    downloading_message: Frame,
    downloading_speed: Frame,
    downloading_progress: Progress,
    job_spinner: LoadingSpinner,
    logs: InstallerLogs,
}

static GREY_COLOR: Color = Color::from_rgb(200, 200, 200);
static MESSAGE_COLOR: Color = Color::from_rgb(10, 10, 10);

impl Step3Tab {
    pub fn new(
        logs: InstallerLogs,
        group: &mut Group,
        style: &AppStyle,
        sender: Sender<Message>,
    ) -> Self {
        let mut panel = Flex::default_fill().column();
        panel.resize(group.x(), group.y(), group.w(), group.h());
        group.add(&panel);
        panel.set_margins(40, 20, 40, 20);

        Frame::default();

        let job_spinner: LoadingSpinner;
        let mut job_progress: Progress;
        let mut job_message: Frame;
        let mut job_percent: Frame;
        let mut downloading_message: Frame;
        let downloading_speed: Frame;
        let mut downloading_progress: Progress;

        // ---------------- Job0 ------------------------------------------
        let mut job_flex = Flex::default_fill().row();
        panel.fixed(&job_flex, 32);
        {
            job_spinner = LoadingSpinner::new(36);
            job_flex.fixed(job_spinner.widget(), 36);

            let mut flex = Flex::default_fill().column();
            flex.set_margins(0, 0, 0, 0);
            flex.set_spacing(0);
            {
                let mut msg_flex = Flex::default_fill().row();
                {
                    job_message = Frame::default()
                        .with_label("下载Python程序包")
                        .with_align(Align::Inside | Align::Left);
                    job_message.set_label_size(16);
                    job_message.set_label_color(MESSAGE_COLOR);

                    job_percent = Frame::default()
                        .with_label("")
                        .with_align(Align::Inside | Align::Right);
                    job_percent.set_label_size(12);
                    job_percent.set_label_color(MESSAGE_COLOR);
                    msg_flex.fixed(&job_percent, 60);

                    msg_flex.end();
                }

                job_progress = Progress::default();
                job_progress.set_color(GREY_COLOR);
                job_progress.set_frame(fltk::enums::FrameType::FlatBox);
                job_progress.set_minimum(0.0);
                job_progress.set_maximum(100.0);
                job_progress.set_selection_color(style.tgu_color);

                flex.fixed(&job_progress, 4);

                flex.fixed(&Frame::default(), 1);

                flex.end();
            }
            job_flex.end();
        }

        panel.fixed(&mut Frame::default(), 10);

        let mut job_flex = Flex::default_fill().row();
        panel.fixed(&job_flex, 24);
        {
            job_flex.fixed(&Frame::default(), 36);

            let mut flex = Flex::default_fill().column();
            {
                let mut msg_flex = Flex::default_fill().row();
                {
                    downloading_message = Frame::default()
                        .with_label("")
                        .with_align(Align::Inside | Align::Left);
                    downloading_message.set_label_size(12);
                    downloading_message.set_label_color(GREY_COLOR);

                    downloading_speed = Frame::default()
                        .with_label("")
                        .with_align(Align::Inside | Align::Right);
                    downloading_message.set_label_size(12);
                    downloading_message.set_label_color(MESSAGE_COLOR);
                    msg_flex.fixed(&downloading_message, 80);

                    msg_flex.end();
                }

                downloading_progress = Progress::default();
                downloading_progress.set_color(GREY_COLOR);
                downloading_progress.set_frame(fltk::enums::FrameType::FlatBox);
                downloading_progress.set_minimum(0.0);
                downloading_progress.set_maximum(100.0);
                downloading_progress.set_selection_color(style.tgu_color);

                flex.fixed(&downloading_progress, 3);

                flex.end();
            }

            job_flex.end();
        }

        let frame = Frame::default();
        panel.fixed(&frame, 30);

        Frame::default();

        panel.end();

        Step3Tab {
            installer: None,
            logs,
            panel,
            sender,
            job_spinner,
            job_progress,
            job_message,
            job_percent,
            downloading_message,
            downloading_speed,
            downloading_progress,
        }
    }

    pub fn widget(&self) -> &Flex {
        &self.panel
    }

    pub fn start(&mut self, installer: Installer) {
        let handle = tokio::runtime::Handle::current();
        let collector = Step4Collector::new(self.logs.clone(), self.sender.clone());

        std::thread::spawn(move || {
            // 在新线程内运行异步代码
            handle.block_on(run(installer, collector));
        });
    }

    pub fn handle_message(&mut self, msg: Step3Message) {
        match msg {
            Step3Message::JobStart => {
                self.job_spinner.start();
            }
            Step3Message::JobSuccess => {
                self.job_spinner.success();
            }
            Step3Message::JobProgress(num, max_num) => {
                let percent = num as f64 / max_num as f64 * 100.0;

                self.job_percent.set_label(&format!("{num}/{max_num}"));
                self.job_progress.set_value(percent);
                self.job_progress.redraw();
            }
            Step3Message::JobMessage(msg) => {
                self.job_message.set_label(&msg);
            }
            Step3Message::Downloading {
                title,
                total_size,
                percentage,
                speed,
            } => {
                let total_size = format_scale(total_size as f64, 1);
                let speed = format_scale(speed as f64, 2);

                let msg = format!("{title}, {total_size}");
                self.downloading_message.set_label(&msg);
                self.downloading_message.set_label_color(MESSAGE_COLOR);
                self.downloading_speed.set_label(&format!("{speed}/s"));
                self.downloading_progress.set_value(percentage);
            }
            _ => {
                unimplemented!()
            }
        }
    }
}

pub struct Step4Collector {
    sender: Sender<Message>,
    logs: InstallerLogs,
}

impl Step4Collector {
    pub fn new(logs: InstallerLogs, sender: Sender<Message>) -> Self {
        Step4Collector { logs, sender }
    }

    pub fn done(&mut self, installer: Installer) {
        self.send(Step3Message::Done(installer));
    }

    pub fn job_start(&mut self) {
        self.send(Step3Message::JobStart);
    }

    pub fn job_success(&mut self) {
        self.send(Step3Message::JobSuccess);
    }

    pub fn job_error(&mut self, err: String) {
        self.send(Step3Message::JobError(err));
    }

    fn send(&self, msg: Step3Message) {
        self.sender.send(Message::Step3(msg));
    }

    fn write(&self, record: InstallerLogRecord) {
        if let Ok(mut records) = self.logs.lock() {
            records.push(record);
        } else {
            fltk::app::awake_callback(move || {
                fltk::dialog::alert_default("无法获取日志锁");
            });
        }
    }
}

impl StatusUpdate for Step4Collector {
    fn alert(&self, err: &str) {
        println!("ERROR: {err}");
    }

    fn message(&self, msg: &str) {
        self.send(Step3Message::JobMessage(msg.to_string()));
    }

    fn update_progress(&self, num: u32, max_num: u32) {
        self.send(Step3Message::JobProgress(num, max_num));
    }

    fn update_downloading(&self, status: &DownloadingStats) {
        self.send(Step3Message::Downloading {
            title: status.title().to_string(),
            total_size: status.total_size(),
            percentage: status.percentage(),
            speed: status.speed(),
        });
    }

    fn log_debug(&self, msg: String) {
        println!("{}", msg);
        self.write(InstallerLogRecord::Debug(msg));
    }

    fn log_error(&self, msg: String) {
        eprintln!("{}", msg);
        self.write(InstallerLogRecord::Error(msg));
    }
}

pub async fn run(mut installer: Installer, mut collector: Step4Collector) {
    use super::super::pyenv::{download_requirements, set_platform_info};

    collector.job_start();

    if let Err(err) = set_platform_info(&mut installer, &collector) {
        collector.job_error(format!("获取系统平台信息中发生错误: {err}"));
        return;
    }

    if let Err(err) = download_requirements(&installer, &collector).await {
        collector.job_error(format!("下载安装软件包中发生错误: {err}"));
        return;
    }

    collector.job_success();

    collector.done(installer);
}
