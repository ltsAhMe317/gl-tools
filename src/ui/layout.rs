use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::{
    gl_unit::{FrameBuffer, window::Window},
    ui::{
        KeyStream, UIObject, UIlayout, UIrender, color, font,
        object::{UItext, rc_refcell},
    },
};

#[derive(PartialEq, Eq, Hash)]
pub enum LayoutPos {
    Bottom,
    Top,
    Left,
    Right,
    Center,
    Round,
}
pub struct FloatLayout<T: UIlayout + UIrender> {
    pub obj: T,
    float: (LayoutPos, LayoutPos),
}
impl<T> FloatLayout<T>
where
    T: UIlayout + UIrender,
{
    pub fn new(obj: T, float: (LayoutPos, LayoutPos)) -> Self {
        Self { obj, float }
    }
    pub fn add_obj(&mut self, obj: Box<dyn UIObject>) {
        self.obj = *Box::<dyn Any>::downcast::<T>(obj).unwrap();
    }
}
impl<T> UIrender for FloatLayout<T>
where
    T: UIlayout + UIrender,
{
    fn fast_draw(&self, window: &mut Window) {
        self.obj.fast_draw(window);
    }
    fn update(&mut self, window: &mut Window, key_stream: &mut KeyStream) {
        let (o_w, o_h) = self.obj.size();
        let (w, h) = window.window.get_size();
        let (w, h) = (w as f32, h as f32);
        let pos = match &self.float {
            (LayoutPos::Left, LayoutPos::Bottom) => (-w / 2f32, -h / 2f32),
            (LayoutPos::Left, LayoutPos::Top) => (-w / 2f32, h / 2f32 - o_h),
            (LayoutPos::Left, LayoutPos::Center) => (-w / 2f32, h / 2f32 - o_h),
            (LayoutPos::Right, LayoutPos::Bottom) => (w / 2f32 - o_w, -h / 2f32),
            (LayoutPos::Right, LayoutPos::Top) => (w / 2f32, h / 2f32 - o_h),
            (LayoutPos::Right, LayoutPos::Center) => (w / 2f32, h / 2f32 - o_h),
            (LayoutPos::Center, LayoutPos::Bottom) => (w / 2f32 - o_w, -h / 2f32),
            (LayoutPos::Center, LayoutPos::Top) => (w / 2f32, h / 2f32 - o_h),
            (LayoutPos::Center, LayoutPos::Center) => (-o_w / 2f32, -o_h / 2f32),

            _ => {
                panic!("only (x,y) allowed")
            }
        };

        self.obj.set_pos(pos);
        self.obj.update(window, key_stream);
    }
    fn draw(&self) -> Option<&FrameBuffer> {
        self.obj.draw()
    }
}

pub struct ListLayout {
    split: f32,
    pos: (f32, f32),
    group: Vec<Box<dyn UIObject>>,
    list_mode: LayoutPos,
}

impl UIlayout for ListLayout {
    fn size(&self) -> (f32, f32) {
        let mut size_count = (0.0, 0.0);
        for obj in self.group.iter() {
            match self.list_mode {
                LayoutPos::Bottom | LayoutPos::Top => {
                    let obj_size = obj.size();
                    if obj_size.0 > size_count.0 {
                        size_count.0 = obj_size.0;
                    }
                    size_count.1 += obj_size.1 + self.split;
                }
                LayoutPos::Left | LayoutPos::Right => {
                    let obj_size = obj.size();
                    if obj_size.1 > size_count.1 {
                        size_count.1 = obj_size.1;
                    }
                    size_count.0 += obj_size.0 + self.split;
                }
                LayoutPos::Center | LayoutPos::Round => panic!("center not support"),
            }
        }
        size_count
    }

    fn set_pos(&mut self, pos: (f32, f32)) {
        self.pos = pos;
    }
}

impl UIrender for ListLayout {
    fn draw(&self) -> Option<&FrameBuffer> {
        None
    }
    fn fast_draw(&self, window: &mut Window) {
        for obj in self.group.iter() {
            obj.fast_draw(window);
        }
    }

    fn update(&mut self, window: &mut Window, key_stream: &mut KeyStream) {
        let (mut x, mut y) = self.pos;
        match &self.list_mode {
            LayoutPos::Bottom | LayoutPos::Left => {
                for obj in self.group.iter_mut().rev() {
                    match &self.list_mode {
                        LayoutPos::Bottom => {
                            let (_, obj_h) = obj.size();
                            obj.set_pos((x, y));
                            y += obj_h;
                            y += self.split;
                        }
                        LayoutPos::Left => {
                            let (obj_w, obj_h) = obj.size();
                            obj.set_pos((x, y));
                            obj.fast_draw(window);
                            x += obj_w;
                            x += self.split;
                        }
                        _ => panic!(),
                    }
                    obj.update(window, key_stream);
                }
            }
            LayoutPos::Top | LayoutPos::Right => {
                for obj in self.group.iter_mut() {
                    match &self.list_mode {
                        LayoutPos::Top => {
                            let (obj_w, obj_h) = obj.size();
                            obj.set_pos((x, y));
                            obj.fast_draw(window);
                            y += obj_h;
                            y += self.split;
                        }
                        LayoutPos::Right => {
                            let (obj_w, obj_h) = obj.size();
                            obj.set_pos((x, y));
                            obj.fast_draw(window);
                            x += obj_w;
                            x += self.split;
                        }
                        _ => panic!(),
                    }
                    obj.update(window, key_stream);
                }
            }
            LayoutPos::Center | LayoutPos::Round => panic!("Center not support"),
        }
    }
}
impl ListLayout {
    pub fn new(list_mode: LayoutPos, split: f32, pos: (f32, f32)) -> Self {
        Self {
            split,
            pos,
            group: Vec::new(),
            list_mode,
        }
    }
    pub fn add_obj(&mut self, obj: Box<dyn UIObject>) {
        self.group.push(obj);
    }
    pub fn add<T: UIObject + 'static>(&mut self, obj: T) {
        self.group.push(Box::new(obj) as Box<dyn UIObject>);
    }
}

pub struct WindowLayout<T: UIObject> {
    pub title: BoundLayout<UItext>,
    pub pos: (f32, f32),
    pub obj: T,
    pub last_cursor_pos: Option<(f32, f32)>,
}
impl<T: UIObject> WindowLayout<T> {
    pub fn new(pos: (f32, f32), title: &str, title_size: i32, obj: T) -> Self {
        let text = UItext {
            color: (0f32, 0f32, 0f32, 1f32),
            pos: (0f32, 0f32),
            text_size: title_size,
            text: rc_refcell(title.to_string()),
        };
        let mut bound = BoundLayout {
            pos: (0f32, 0f32),
            bound: HashMap::new(),
            obj: text,
        };
        bound.add_bound(LayoutPos::Round, 15f32);
        Self {
            pos,
            obj,
            last_cursor_pos: None,
            title: bound,
        }
    }
    pub fn add_obj(&mut self, obj: Box<dyn UIObject>) {
        let obj = Box::<dyn Any>::downcast::<T>(obj).unwrap();
        self.obj = *obj;
    }
}
impl<T: UIObject> UIrender for WindowLayout<T> {
    fn draw(&self) -> Option<&FrameBuffer> {
        None
    }
    fn fast_draw(&self, window: &mut Window) {
        let window_size = window.window.get_size();
        let mut obj_size = self.obj.size();
        color(
            window_size,
            (100, 100, 100, 255),
            (self.pos.0, self.pos.1 + obj_size.1),
            (obj_size.0, obj_size.1),
            25,
        );
        self.obj.fast_draw(window);

        let title_size = self.title.size();
        let title_pos = {
            let mut size = self.pos;
            size.1 += obj_size.1 + title_size.1;
            size
        };
        let title_size = (
            if title_size.0 > obj_size.0 {
                title_size.0
            } else {
                obj_size.0
            },
            title_size.1,
        );
        color(window_size, (255, 255, 255, 255), title_pos, title_size, 25);
        self.title.fast_draw(window);
    }

    fn update(&mut self, window: &mut Window, key_stream: &mut KeyStream) {
        let window_size = window.window.get_size();
        let obj_size = self.obj.size();

        let title_size = self.title.size();
        let title_pos = {
            let mut size = self.pos;
            size.1 += obj_size.1;
            size
        };
        let title_size = (
            if title_size.0 > obj_size.0 {
                title_size.0
            } else {
                obj_size.0
            },
            title_size.1,
        );

        let (cursor_x, cursor_y) = window.window.get_cursor_pos();
        let (cursor_x, cursor_y) = (cursor_x as f32, cursor_y as f32);
        let (cursor_x, cursor_y) = (
            cursor_x - window_size.0 as f32 / 2f32,
            -(cursor_y - window_size.1 as f32 / 2f32),
        );
        if cursor_x > title_pos.0
            && cursor_x < title_pos.0 + title_size.0
            && cursor_y > title_pos.1
            && cursor_y < title_pos.1 as f32 + title_size.1
            && key_stream.cursor_close()
        {
            if window.window.get_mouse_button(glfw::MouseButton::Button1) == glfw::Action::Press
                && key_stream.use_mouse_button(glfw::MouseButton::Button1)
            {
                if let Some(last_cursor_pos) = self.last_cursor_pos {
                    self.pos.0 -= last_cursor_pos.0 - cursor_x;
                    self.pos.1 -= last_cursor_pos.1 - cursor_y;
                }
                self.last_cursor_pos = Some((cursor_x, cursor_y));
            } else {
                self.last_cursor_pos = None;
            }
        }
        self.obj.set_pos(self.pos);
        self.obj.update(window, key_stream);
        self.title.set_pos(title_pos);
        self.title.update(window, key_stream);
    }
}
impl<T: UIObject> UIlayout for WindowLayout<T> {
    fn size(&self) -> (f32, f32) {
        let mut size = self.obj.size();
        size.1 += self.title.size().1;
        size
    }

    fn set_pos(&mut self, pos: (f32, f32)) {
        self.pos = pos;
    }
}

pub struct BoundLayout<T: UIObject> {
    pub pos: (f32, f32),
    pub bound: HashMap<LayoutPos, f32>,
    pub obj: T,
}
impl<T: UIObject> BoundLayout<T> {
    pub fn add_bound(&mut self, pos: LayoutPos, value: f32) {
        self.bound.insert(pos, value);
    }
    pub fn add_obj(&mut self, obj: Box<dyn UIObject>) {
        self.obj = *Box::<dyn Any>::downcast(obj).unwrap();
    }
}
impl<T: UIObject> UIrender for BoundLayout<T> {
    fn draw(&self) -> Option<&FrameBuffer> {
        None
    }
    fn fast_draw(&self, window: &mut Window) {
        self.obj.fast_draw(window);
    }

    fn update(&mut self, window: &mut Window, key_stream: &mut KeyStream) {
        let mut pos = self.pos;
        let (obj_w, obj_h) = self.obj.size();
        let (w, h) = self.size();
        if let Some(value) = self.bound.get(&LayoutPos::Left) {
            pos.0 = pos.0 + value;
        } else if let Some(value) = self.bound.get(&LayoutPos::Right) {
            pos.0 = pos.0 + w - value - obj_w;
        }
        if let Some(value) = self.bound.get(&LayoutPos::Top) {
            pos.1 = pos.1 + h - obj_h - value;
        } else if let Some(value) = self.bound.get(&LayoutPos::Bottom) {
            pos.1 = pos.1 + value;
        }
        if let Some(value) = self.bound.get(&LayoutPos::Round) {
            let half_obj_w = obj_w / 2f32;
            let half_obj_h = obj_h / 2f32;
            pos.0 = pos.0 + (w / 2f32) - half_obj_w;
            pos.1 = pos.1 + (h / 2f32) - half_obj_h;
        }
        self.obj.set_pos(pos);
        self.obj.update(window, key_stream);
    }

    
}
impl<T: UIObject> UIlayout for BoundLayout<T> {
    fn size(&self) -> (f32, f32) {
        let (mut w, mut h) = self.obj.size();

        if let Some(value) = self.bound.get(&LayoutPos::Left) {
            w += value;
        }
        if let Some(value) = self.bound.get(&LayoutPos::Right) {
            w += value;
        }
        if let Some(value) = self.bound.get(&LayoutPos::Top) {
            h += value;
        }
        if let Some(value) = self.bound.get(&LayoutPos::Bottom) {
            h += value;
        }
        if let Some(value) = self.bound.get(&LayoutPos::Round) {
            let value = value * 2f32;
            w += value;
            h += value;
        }

        (w, h)
    }

    fn set_pos(&mut self, pos: (f32, f32)) {
        self.pos = pos;
    }
}
