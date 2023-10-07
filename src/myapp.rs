use fltk::{
    app::{Receiver, Sender},
    button::Button,
    enums::Event,
    frame::Frame,
    group::{Flex, Group},
    image::IcoImage,
    prelude::{GroupExt, WidgetBase, WidgetExt, WindowExt},
    window::DoubleWindow,
};
use std::any::Any;

use super::{
    steps::{
        navbar::PhaseNavBar,
        step1::{Step1Message, Step1Tab},
        step2::{Step2Message, Step2Tab},
        step3::{Step3Message, Step3Tab},
        step4::{Step4Message, Step4Tab},
        step5::{Step5Message, Step5Tab},
        step6::{Step6Message, Step6Tab},
    },
    style::AppStyle,
};

pub struct MyApp {
    app: fltk::app::App,
    r: Receiver<Message>,
    s: Sender<Message>,
    navbar: PhaseNavBar,
    step_idx: usize,
    step_group: Group,
    step_objs: Vec<Box<dyn Any>>,
    // main_win: DoubleWindow,
    style: AppStyle,
}

#[derive(Debug)]
pub enum Message {
    Step1(Step1Message),
    Step2(Step2Message),
    Step3(Step3Message),
    Step4(Step4Message),
    Step5(Step5Message),
    Step6(Step6Message),
    Quit,
}

fn app_title(parent: &mut Flex, style: &AppStyle) {
    use fltk::enums::Align;

    let panel = Flex::default().column();
    parent.fixed(&panel, 42);

    let mut title_zh = Frame::default()
        .with_label("天工业务数据分析(TGBA)实验环境 - 自助安装")
        .with_align(Align::Inside | Align::Left);
    title_zh.set_label_font(style.font_bold_zh);
    title_zh.set_label_size(22);
    title_zh.set_label_color(style.tgu_color);

    let mut title_en = Frame::default()
        .with_label("TianGong Business Analytics (TGBA) Lab - Installer")
        .with_align(Align::Inside | Align::Left);
    title_en.set_label_font(style.font_bold_en);
    title_en.set_label_size(16);
    title_en.set_label_color(style.darkgrey);

    panel.end();
}

fn app_footer(s: &Sender<Message>, parent: &mut Flex, style: &AppStyle) {
    use fltk::enums::Align;

    let panel = Flex::default().row();
    parent.fixed(&panel, 24);

    // let mut btn = Button::default().with_label("退出");
    // panel.fixed(&btn, 40);
    // btn.set_callback({
    //     let s = s.clone();
    //     move |_| {
    //         s.send(Message::Quit);
    //     }
    // });

    // Frame::default();

    let mut footer = Frame::default()
        .with_label("天津工业大学经济与管理学院 © 2023")
        .with_align(Align::Inside | Align::Right);

    footer.set_label_font(style.font_zh);
    footer.set_label_size(12);
    footer.set_label_color(style.darkgrey);

    panel.end()

    // parent.fixed(&footer, 18);
}

impl MyApp {
    pub fn new() -> Self {
        let app = fltk::app::App::default().with_scheme(fltk::app::Scheme::Gtk);

        app.load_system_fonts();
        let style = AppStyle::default();

        fltk::app::background(255, 255, 255);
        fltk::app::set_visible_focus(false);
        fltk::app::set_font(style.font_zh);
        fltk::app::set_font_size(12);

        let (s, r) = fltk::app::channel::<Message>();

        let mut main_win = DoubleWindow::default()
            .with_size(700, 300)
            .center_screen()
            .with_label("TGBA安装程序");

        main_win.begin();
        let icon = IcoImage::load(&"resources/tgba-jupyterlab-48x48.ico").unwrap();
        main_win.set_icon(Some(icon));

        let mut main_flex = Flex::default_fill().column();
        main_flex.set_margins(10, 10, 10, 10);

        app_title(&mut main_flex, &style);

        let navbar = PhaseNavBar::new(&style);
        main_flex.fixed(navbar.navbar_row(), 40);

        let mut step_group = Group::default_fill();
        let step_objs: Vec<Box<dyn Any>> = vec![
            Box::new(Step1Tab::new(&mut step_group, &style, s.clone())),
            Box::new(Step2Tab::new(&mut step_group, &style, s.clone())),
            Box::new(Step3Tab::new(&mut step_group, &style, s.clone())),
            Box::new(Step4Tab::new(&mut step_group, &style, s.clone())),
            Box::new(Step5Tab::new(&mut step_group, &style, s.clone())),
            Box::new(Step6Tab::new(&mut step_group, &style, s.clone())),
        ];
        step_group.end();

        app_footer(&s, &mut main_flex, &style);

        main_win.end();
        main_win.show();

        s.send(Message::Step1(Step1Message::Enter));

        let myapp = MyApp {
            app,
            r,
            s,
            step_idx: 0,
            step_group,
            navbar,
            step_objs,
            style,
        };

        main_win.set_callback({
            let s = myapp.s.clone();
            move |_| {
                let event = fltk::app::event();
                if event == Event::Close {
                    s.send(Message::Quit);
                }
            }
        });

        main_win.handle({
            let mut x = 0;
            let mut y = 0;
            move |w, ev| {
                // 按住鼠标移动窗口
                match ev {
                    Event::Push => {
                        let (coord_x, coord_y) = fltk::app::event_coords();
                        x = coord_x;
                        y = coord_y;
                        true
                    }
                    Event::Drag => {
                        w.set_pos(fltk::app::event_x_root() - x, fltk::app::event_y_root() - y);
                        true
                    }
                    _ => false,
                }
            }
        });

        myapp
    }

    fn set_step(&mut self, step_idx: usize) {
        self.step_idx = step_idx;
        self.navbar.set_activate(step_idx as i32);

        let step_idx = step_idx as i32;
        for idx in 0..self.step_group.children() {
            if idx != step_idx {
                self.step_group.child(idx).unwrap().hide();
            } else {
                self.step_group.child(idx).unwrap().show();
            }
        }
    }

    #[inline]
    fn get_step_mut<T: Any>(&mut self) -> &mut T {
        self.step_objs[self.step_idx].downcast_mut::<T>().unwrap()
    }

    pub fn run(&mut self) {
        while self.app.wait() {
            let Some(msg) = self.r.recv() else {
                continue;
            };

            let s = self.s.clone();

            use Message::*;
            match msg {
                Step1(msg @ Step1Message::Enter) => {
                    self.set_step(0);
                    let d = self.get_step_mut::<Step1Tab>();
                    d.handle_message(msg);
                }
                Step1(Step1Message::Done) => {
                    println!("step1: done");
                    s.send(Step2(Step2Message::Enter));
                }
                Step1(msg) => {
                    let d = self.get_step_mut::<Step1Tab>();
                    d.handle_message(msg);
                }
                //
                Step2(msg @ Step2Message::Enter) => {
                    self.set_step(1);
                    let d = self.get_step_mut::<Step2Tab>();
                    d.handle_message(msg);
                }
                Step2(Step2Message::Done) => {
                    println!("step2: done");
                    s.send(Step3(Step3Message::Enter));
                }
                Step2(msg) => {
                    let d = self.get_step_mut::<Step2Tab>();
                    d.handle_message(msg);
                }
                //
                Step3(msg @ Step3Message::Enter) => {
                    self.set_step(2);
                    let d = self.get_step_mut::<Step3Tab>();
                    d.c();
                    d.handle_message(msg);
                }
                Step3(Step3Message::Done) => {
                    println!("step3: done");
                    s.send(Step4(Step4Message::Enter));
                }
                Step3(msg) => {
                    let d = self.get_step_mut::<Step3Tab>();
                    d.handle_message(msg);
                }

                //
                Step4(msg @ Step4Message::Enter) => {
                    self.set_step(3);
                    let d = self.get_step_mut::<Step4Tab>();
                    d.c();
                    d.handle_message(msg);
                }
                Step4(Step4Message::Done) => {
                    println!("step4: done");
                    s.send(Step5(Step5Message::Enter));
                }
                Step4(msg) => {
                    let d = self.get_step_mut::<Step4Tab>();
                    d.handle_message(msg);
                }
                //
                Step5(msg @ Step5Message::Enter) => {
                    self.set_step(4);
                    let d = self.get_step_mut::<Step5Tab>();
                    d.c();
                    d.handle_message(msg);
                }
                Step5(Step5Message::Done) => {
                    println!("step5: done");
                    s.send(Step6(Step6Message::Enter));
                }
                Step5(msg) => {
                    let d = self.get_step_mut::<Step5Tab>();
                    d.handle_message(msg);
                }
                //
                Step6(msg @ Step6Message::Enter) => {
                    self.set_step(5);
                    let d = self.get_step_mut::<Step6Tab>();
                    d.c();
                    d.handle_message(msg);
                }
                Step6(Step6Message::Done) => {
                    println!("step6: done");
                    // s.send(Step6(Step6Message::Enter));
                }
                Step6(msg) => {
                    let d = self.get_step_mut::<Step6Tab>();
                    d.handle_message(msg);
                }
                Quit => {
                    self.handle_quit();
                }
            }
        }
        // self.app.run();
    }

    fn handle_quit(&self) {
        // let (vw, vh) = fltk::app::screen_size();
        let (x, y) = fltk::app::event_coords();

        let x = fltk::app::event_x_root() - 150;
        let y = fltk::app::event_y_root() - 50;
        // let d = show_dialog();
        // // MyDialog::default();

        fltk::dialog::message_set_hotspot(false);
        fltk::dialog::message_set_font(self.style.font_zh, 16);
        fltk::dialog::message_icon_label("?");
        fltk::dialog::message_title("请确认：");

        let choice = fltk::dialog::choice2(
            x,
            y,
            "是否中断现在的安装？",
            "中断安装离开",
            "继续安装",
            "",
        );

        if let Some(0) = choice {
            self.app.quit();
        }
    }
}

