use std::{cell::RefCell, collections::HashMap, ops::DerefMut, rc::Rc};

use glfw::Action;

use crate::{
    gl_unit::{FrameBuffer, window::Window},
    setter_gen,
    ui::{KeyStream, UIlayout, UIrender, color, font, layout::BoundLayout},
};

pub fn rc_refcell<T>(value: T) -> Rc<RefCell<T>> {
    Rc::new(RefCell::new(value))
}

pub struct UItext {
    pub color: (f32, f32, f32, f32),
    pub pos: (f32, f32),
    pub text_size: i32,
    pub text: Rc<RefCell<String>>,
}
setter_gen! {
    impl UItext{
        color: (f32, f32, f32, f32),
        pos: (f32, f32),
        text_size: i32
    }
}
impl UItext {
    pub fn new(text: &str) -> Self {
        Self {
            color: (1f32, 1f32, 1f32, 1f32),
            pos: (0f32, 0f32),
            text_size: 25,
            text: rc_refcell(text.to_string()),
        }
    }
    pub fn text(self, text: &str) -> Self {
        *self.text.borrow_mut() = text.to_string();
        self
    }
    pub fn get_text(&self) -> Rc<RefCell<String>> {
        self.text.clone()
    }
}
impl UIrender for UItext {
    fn draw(&self) -> Option<&FrameBuffer> {
        None
    }
    fn fast_draw(&self, window: &mut Window) {
        font::font(|font| {
            font.draw(
                self.text.borrow().as_str(),
                window.window.get_size(),
                self.pos.0,
                self.pos.1,
                self.text_size,
                self.color,
            );
        });
    }

    fn update(&mut self, window: &mut Window, key_stream: &mut KeyStream) {}
}
impl UIlayout for UItext {
    fn size(&self) -> (f32, f32) {
        (
            font::font(|font| font.size(self.text.borrow().as_str(), self.text_size)) as f32,
            self.text_size as f32,
        )
    }
    fn set_pos(&mut self, pos: (f32, f32)) {
        self.pos = pos;
    }
}
pub struct UIkeep {
    text: UIbutton,
    pub enable: Rc<RefCell<bool>>,
}
impl UIkeep {
    pub fn new(str: &str) -> (Self,Rc<RefCell<bool>>) {
        let enable = Rc::new(RefCell::new(false));
        let enable_clone = enable.clone();
        let enable_clone_again = enable.clone();
        (Self {
            text: UIbutton {
                check_click: false,
                text: UItext::new(str),
                action: Box::new(move || {
                    let bool = *enable_clone.borrow();
                    *enable_clone.borrow_mut() = !bool;
                }),
            },
            enable,
        },enable_clone_again)
    }
}
impl UIlayout for UIkeep {
    fn size(&self) -> (f32, f32) {
        self.text.size()
    }

    fn set_pos(&mut self, pos: (f32, f32)) {
        self.text.set_pos(pos);
    }
}
impl UIrender for UIkeep {
    fn draw(&self) -> Option<&FrameBuffer> {
        None
    }

    fn fast_draw(&self, window: &mut Window) {
        self.text.fast_draw(window);
    }

    fn update(&mut self, window: &mut Window, key_stream: &mut KeyStream) {
        self.text.update(window, key_stream);
        if *self.enable.borrow(){
            self.text.text.color = (1f32,0f32,0f32,1f32);
        }
    }
}
pub struct UIbutton {
    pub check_click: bool,
    pub text: UItext,
    pub action: Box<dyn FnMut()>,
}
impl UIlayout for UIbutton {
    fn size(&self) -> (f32, f32) {
        self.text.size()
    }
    fn set_pos(&mut self, pos: (f32, f32)) {
        self.text.set_pos(pos);
    }
}
impl UIrender for UIbutton {
    fn fast_draw(&self, window: &mut Window) {
        let window_size = window.window.get_size();
        self.text.fast_draw(window);
        let (text_w, _) = self.text.size();
        color(
            window_size,
            (255, 255, 255, 255),
            self.text.pos,
            (text_w, 1f32),
            1,
        );
    }

    fn draw(&self) -> Option<&FrameBuffer> {
        None
    }

    fn update(&mut self, window: &mut Window, key_stream: &mut KeyStream) {
        let (text_w, _) = self.text.size();
        let (x, y) = window.window.get_cursor_pos();

        let window_size = window.window.get_size();
        let (x, y) = (
            x - window_size.0 as f64 / 2f64,
            y - window_size.1 as f64 / 2f64,
        );
        let (x, y) = (x as f32, -y as f32);

        if x > self.text.pos.0
            && x < self.text.pos.0 + text_w
            && y > self.text.pos.1
            && y < self.text.pos.1 + self.text.text_size as f32
            && key_stream.cursor_close()
        {
            self.text.color = (1f32, 1f32, 0f32, 1f32);
            if window.window.get_mouse_button(glfw::MouseButton::Button1) == Action::Press
                && !self.check_click
                && key_stream.use_mouse_button(glfw::MouseButton::Button1)
            {
                self.check_click = true;
            }
            if window.window.get_mouse_button(glfw::MouseButton::Button1) == Action::Release
                && self.check_click
            {
                self.check_click = false;
                (self.action)();
            }
        } else {
            self.text.color = (1f32, 1f32, 1f32, 1f32);
        }
    }
}

pub struct UIinput {
    pos: (f32, f32),
    str_buffer: BoundLayout<UItext>,
    is_input: bool,
}
impl UIinput {
    pub fn new(pos: (f32, f32)) -> (Self, Rc<RefCell<String>>) {
        let buffer = rc_refcell(String::new());
        let text = UItext {
            color: (0f32, 0f32, 0f32, 1f32),
            pos: (0f32, 0f32),
            text_size: 25,
            text: buffer.clone(),
        };
        let mut bound = BoundLayout {
            pos: (0f32, 0f32),
            bound: HashMap::new(),
            obj: text,
        };
        bound.add_bound(super::layout::LayoutPos::Round, 20f32);
        (
            Self {
                str_buffer: bound,
                pos,
                is_input: false,
            },
            buffer,
        )
    }
}
impl UIrender for UIinput {
    fn draw(&self) -> Option<&FrameBuffer> {
        None
    }

    fn fast_draw(&self, window: &mut Window) {
        let size = self.str_buffer.size();
        color(
            window.window.get_size(),
            (255, 255, 255, 255),
            {
                let mut pos = self.pos;
                pos.1 += size.1;
                pos
            },
            size,
            20,
        );
        self.str_buffer.fast_draw(window);
    }

    fn update(&mut self, window: &mut Window, key_stream: &mut KeyStream) {
        let (x, y) = window.window.get_cursor_pos();

        let window_size = window.window.get_size();
        let (x, y) = (
            x - window_size.0 as f64 / 2f64,
            y - window_size.1 as f64 / 2f64,
        );
        let (x, y) = (x as f32, -y as f32);
        let size = self.size();
        if x > self.pos.0
            && x < self.pos.0 + size.0
            && y > self.pos.1
            && y < self.pos.1 + size.1
            && key_stream.cursor_close()
        {
            if window.window.get_mouse_button(glfw::MouseButton::Button1) == Action::Press
                && key_stream.use_mouse_button(glfw::MouseButton::Button1)
                && !self.is_input
            {
                window.window.set_cursor_mode(glfw::CursorMode::Disabled);
                self.is_input = true;
                let str = self.str_buffer.obj.text.clone();
                window
                    .window
                    .set_key_callback(move |window, key, num, action, _| {
                        if key == glfw::Key::Backspace
                            && (action == Action::Repeat || action == Action::Press)
                        {
                            str.borrow_mut().pop();
                        }
                    });
            }
        }
        if self.is_input
            && window.window.get_key(glfw::Key::Enter) == Action::Press
            && key_stream.use_key(glfw::Key::Enter)
        {
            window.window.set_cursor_mode(glfw::CursorMode::Normal);
            self.is_input = false;
            window.window.unset_key_callback();
        }
        let mem = self.str_buffer.obj.text.borrow().to_string();
        if self.is_input
            && let Some(window_event) = window.window_event()
        {
            match window_event {
                glfw::WindowEvent::Char(char) => {
                    self.str_buffer.obj.text.borrow_mut().push(char);
                }
                _ => {}
            }
        }

        self.str_buffer.set_pos(self.pos);
        self.str_buffer.update(window, key_stream);
    }
}
impl UIlayout for UIinput {
    fn size(&self) -> (f32, f32) {
        self.str_buffer.size()
    }

    fn set_pos(&mut self, pos: (f32, f32)) {
        self.pos = pos;
    }
}

pub struct UIenable<T: UIrender> {
    enable: Rc<RefCell<bool>>,
    obj: T,
}
impl<T: UIrender> UIenable<T> {
    pub fn new(obj: T) -> (Self, Rc<RefCell<bool>>) {
        let button = Rc::new(RefCell::new(false));
        (
            Self {
                enable: button.clone(),
                obj,
            },
            button,
        )
    }
}
impl<T: UIrender> UIrender for UIenable<T> {
    fn draw(&self) -> Option<&FrameBuffer> {
        self.obj.draw()
    }

    fn fast_draw(&self, window: &mut Window) {
        if *self.enable.borrow() {
            self.obj.fast_draw(window);
        }
    }

    fn update(&mut self, window: &mut Window, key_stream: &mut KeyStream) {
        if *self.enable.borrow() {
            self.obj.update(window, key_stream);
        }
    }
}
impl<T: UIrender + UIlayout> UIlayout for UIenable<T> {
    fn size(&self) -> (f32, f32) {
        if *self.enable.borrow() {
            return self.obj.size();
        }
        (0f32, 0f32)
    }

    fn set_pos(&mut self, pos: (f32, f32)) {
        if *self.enable.borrow() {
            self.obj.set_pos(pos);
        }
    }
}
