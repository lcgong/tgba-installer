use fltk::{
    app::Sender,
    enums::Align,
    frame::Frame,
    group::{Flex, Group},
    misc::Progress,
    prelude::{GroupExt, WidgetBase, WidgetExt},
    window::DoubleWindow,
};

use super::super::status::LoadingSpinner;
use super::super::status::{DownloadingStats, StatusUpdate};
use super::super::{myapp::Message, style};
use super::utils::format_scale;
use crate::pyenv::Installer;

use pep508_rs::Requirement;

#[derive(Debug)]
pub enum Step3Message {
    Enter(Installer),
    // JobStart,
    // JobSuccess,
    JobMessage(String),
    // JobProgress(u32, u32),
    JobError(String),
    Downloading {
        title: String,
        total_size: u64,
        percentage: f64,
        speed: f64,
    },
    DownloadingStart(Installer, Vec<Requirement>, usize),
    DownloadingError(Installer, Vec<Requirement>, usize, String),
    DownloadingDone(Installer, Vec<Requirement>, usize),
    Done(Installer),
}

pub struct Step3Tab {
    // c_no: usize,
    // installer: Option<Installer>,
    main_win: DoubleWindow,
    panel: Flex,
    sender: Sender<Message>,
    job_progress: Progress,
    job_message: Frame,
    job_percent: Frame,
    downloading_message: Frame,
    downloading_speed: Frame,
    downloading_progress: Progress,
    job_spinner: LoadingSpinner,
}

impl Step3Tab {
    pub fn new(
        main_win: DoubleWindow,
        group: &mut Group,
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
        let mut downloading_speed: Frame;
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
                    job_message.set_label_color(style::COLOR_MESSAGE);

                    job_percent = Frame::default()
                        .with_label("")
                        .with_align(Align::Inside | Align::Right);
                    job_percent.set_label_size(12);
                    job_percent.set_label_color(style::COLOR_MESSAGE);
                    msg_flex.fixed(&job_percent, 60);

                    msg_flex.end();
                }

                job_progress = Progress::default();
                job_progress.set_color(style::COLOR_GREY);
                job_progress.set_frame(fltk::enums::FrameType::FlatBox);
                job_progress.set_minimum(0.0);
                job_progress.set_maximum(100.0);
                job_progress.set_selection_color(style::COLOR_TGU);

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
                    downloading_message.set_label_color(style::COLOR_GREY);

                    downloading_speed = Frame::default()
                        .with_label("")
                        .with_align(Align::Inside | Align::Right);
                    downloading_speed.set_label_size(12);
                    downloading_speed.set_label_color(style::COLOR_MESSAGE);
                    msg_flex.fixed(&downloading_speed, 80);

                    msg_flex.end();
                }

                downloading_progress = Progress::default();
                downloading_progress.set_color(style::COLOR_GREY);
                downloading_progress.set_frame(fltk::enums::FrameType::FlatBox);
                downloading_progress.set_minimum(0.0);
                downloading_progress.set_maximum(100.0);
                downloading_progress.set_selection_color(style::COLOR_TGU);

                flex.fixed(&downloading_progress, 3);

                flex.end();
            }

            job_flex.end();
        }

        let frame = Frame::default();
        panel.fixed(&frame, 30);

        Frame::default();

        panel.end();

        log::info!("step3 panel created");

        Step3Tab {
            main_win,
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
        let collector = Step3Collector::new(self.sender.clone());

        self.job_spinner.start();

        std::thread::spawn(move || {
            // 在新线程内运行异步代码
            handle.block_on(prepare_downloading(installer, collector));
        });
    }

    pub fn on_downloading_start(
        &mut self,
        installer: Installer,
        requirements: Vec<Requirement>,
        requirement_idx: usize,
    ) {
        let handle = tokio::runtime::Handle::current();
        let collector = Step3Collector::new(self.sender.clone());

        std::thread::spawn(move || {
            // 在新线程内运行异步代码
            handle.block_on(download_worker(
                installer,
                collector,
                requirements,
                requirement_idx,
            ));
        });
    }

    fn on_downloading_done(
        &mut self,
        installer: Installer,
        requirements: Vec<Requirement>,
        requirement_idx: usize,
    ) {
        let n_requirement = requirements.len();
        let next_idx = requirement_idx + 1;

        {
            // 显示进度比例
            let percent = next_idx as f64 / n_requirement as f64 * 100.0;

            self.job_percent
                .set_label(&format!("{next_idx}/{n_requirement}"));
            self.job_progress.set_value(percent);
            self.job_progress.redraw();
        }

        if next_idx < n_requirement {
            self.sender
                .send(Message::Step3(Step3Message::DownloadingStart(
                    installer,
                    requirements,
                    requirement_idx + 1,
                )));
        } else {
            self.job_spinner.success(); // 最后一个下载结束

            self.sender
                .send(Message::Step3(Step3Message::Done(installer)));
        }
    }

    fn on_downloading_error(
        &self,
        installer: Installer,
        requirements: Vec<Requirement>,
        requirement_idx: usize,
        errmsg: String,
    ) {
        log::error!("下载中遇到错误: \n{errmsg}");
        use super::super::dialog::error_confirm;

        let result = error_confirm(&self.main_win, "下载失败", &errmsg);
        if result == 1 {
            log::info!("用户选择再次尝试下载{}", requirements[requirement_idx]);
            self.sender
                .send(Message::Step3(Step3Message::DownloadingStart(
                    installer,
                    requirements,
                    requirement_idx,
                )));
        } else {
            log::info!("用户选择放弃安装，退出程序！");
        }
    }

    fn on_downloading_status(
        &mut self,
        title: String,
        total_size: u64,
        percentage: f64,
        speed: f64,
    ) {
        let total_size = format_scale(total_size as f64, 1);
        let speed = format_scale(speed as f64, 2);

        let msg = format!("{title}, {total_size}");
        self.downloading_message.set_label(&msg);
        self.downloading_message.set_label_color(style::COLOR_MESSAGE);
        self.downloading_speed.set_label(&format!("{speed}/s"));
        self.downloading_progress.set_value(percentage);
    }

    pub fn handle_message(&mut self, msg: Step3Message) {
        match msg {
            Step3Message::JobMessage(msg) => {
                self.job_message.set_label(&msg);
            }
            Step3Message::DownloadingStart(installer, requirements, requirement_idx) => {
                self.on_downloading_start(installer, requirements, requirement_idx)
            }
            Step3Message::DownloadingDone(installer, requirements, requirement_idx) => {
                self.on_downloading_done(installer, requirements, requirement_idx)
            }
            Step3Message::DownloadingError(installer, requirements, requirement_idx, errmsg) => {
                self.on_downloading_error(installer, requirements, requirement_idx, errmsg)
            }
            Step3Message::Downloading {
                title,
                total_size,
                percentage,
                speed,
            } => {
                self.on_downloading_status(title, total_size, percentage, speed);
            }
            _ => {
                unimplemented!("unknow msg: {:?}", msg)
            }
        }
    }
}

pub struct Step3Collector {
    sender: Sender<Message>,
}

impl Step3Collector {
    pub fn new(sender: Sender<Message>) -> Self {
        Step3Collector { sender }
    }

    pub fn job_error(&mut self, err: String) {
        self.send(Step3Message::JobError(err));
    }

    fn start_downloading(&self, installer: Installer, requirements: Vec<Requirement>, idx: usize) {
        self.send(Step3Message::DownloadingStart(installer, requirements, idx));
    }

    fn send(&self, msg: Step3Message) {
        self.sender.send(Message::Step3(msg));
    }
}

impl StatusUpdate for Step3Collector {
    fn message(&self, msg: &str) {
        self.send(Step3Message::JobMessage(msg.to_string()));
    }

    fn update_downloading(&self, status: &DownloadingStats) {
        self.send(Step3Message::Downloading {
            title: status.title().to_string(),
            total_size: status.total_size(),
            percentage: status.percentage(),
            speed: status.speed(),
        });
    }
}

pub async fn prepare_downloading(mut installer: Installer, mut collector: Step3Collector) {
    use super::super::pyenv::{prepare_requirements, set_platform_info};

    if let Err(err) = set_platform_info(&mut installer) {
        collector.job_error(format!("获取系统平台信息中发生错误: {err}"));
        return;
    }

    let requirements: Vec<pep508_rs::Requirement> = match prepare_requirements(&installer).await {
        Ok(requirements) => requirements,
        Err(err) => {
            collector.job_error(format!("下载安装软件包中发生错误: {err}"));
            return;
        }
    };

    collector.start_downloading(installer, requirements, 0);
}

pub async fn download_worker(
    installer: Installer,
    collector: Step3Collector,
    requirements: Vec<Requirement>,
    requirement_idx: usize,
) {
    use super::super::pyenv::retry_download_requirement;

    let requirement = &requirements[requirement_idx];

    if let Err(err) = retry_download_requirement(&installer, &collector, requirement).await {
        collector.send(Step3Message::DownloadingError(
            installer,
            requirements,
            requirement_idx,
            err.to_string(),
        ));
        return;
    } else {
        collector.send(Step3Message::DownloadingDone(
            installer,
            requirements,
            requirement_idx,
        ));
    }
}
