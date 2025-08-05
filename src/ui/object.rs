use std::{cell::RefCell, ops::DerefMut, rc::Rc};

use glfw::Action;

use crate::{gl_unit::{window::Window, FrameBuffer}, ui::{color, font, KeyStream, UIlayout, UIrender}};

pub fn rc_refcell<T>(value:T)->Rc<RefCell<T>>{
    Rc::new(RefCell::new(value))
}
pub struct UItext{
    pub text_color:(f32,f32,f32,f32),
    pub pos:(f32,f32),
    pub size:i32,
    pub text:Rc<RefCell<String>>,
}
impl UIrender for UItext{
    fn draw(&self)->Option<&FrameBuffer> {
        None
    }
    fn fast_draw(&self, window: &mut Window) {
        font::font(|font|{
            font.draw(self.text.borrow().as_str(),window.window.get_size(), self.pos.0, self.pos.1,self.size, self.text_color);
        });
    }

    fn update(&mut self,window: &mut Window,key_stream:&mut KeyStream) {
    }
}
impl UIlayout for UItext{
    fn size(&self)->(f32,f32) {
        (font::font(|font|{font.size(self.text.borrow().as_str(), self.size)}) as f32,self.size as f32)
    }
    fn set_pos(&mut self,pos:(f32,f32)) {
        self.pos = pos;
    }
}

pub struct UIbutton{
    pub check_click:bool,
    pub text:UItext,
    pub action:Box<dyn FnMut()>
}
impl UIlayout for UIbutton{
    fn size(&self)->(f32,f32) {
        self.text.size()
    }

    fn set_pos(&mut self,pos:(f32,f32)) {
        self.text.set_pos(pos);
    }
}
impl UIrender for UIbutton{
    fn fast_draw(&self, window: &mut Window) {
        let window_size = window.window.get_size();
        self.text.fast_draw(window);
        let (text_w,_) = self.text.size();        
        color(window_size, (255,255,255,255), self.text.pos, (text_w,1f32), 1);
        
    }

    fn draw(&self)->Option<&FrameBuffer> {
        None
    }

    fn update(&mut self,window: &mut Window,key_stream:&mut KeyStream) {
        let (text_w,_) = self.text.size();        
        let (x,y) = window.window.get_cursor_pos();
        
        let window_size = window.window.get_size();
        let (x,y) = (x-window_size.0 as f64/2f64,y-window_size.1 as f64/2f64);
        let (x,y) = (x as f32,-y as f32);
        
        if x>self.text.pos.0&&x<self.text.pos.0+text_w&&y>self.text.pos.1&&y<self.text.pos.1+self.text.size as f32&&key_stream.cursor_close(){
            
            self.text.text_color = (1f32,1f32,0f32,1f32);
            if window.window.get_mouse_button(glfw::MouseButton::Button1) == Action::Press&&!self.check_click&&key_stream.use_mouse_button(glfw::MouseButton::Button1){
                self.check_click = true;
            }
            if window.window.get_mouse_button(glfw::MouseButton::Button1) == Action::Release&&self.check_click{
                self.check_click = false;
                (self.action)();
            }
        }else{
            self.text.text_color = (1f32,1f32,1f32,1f32);
        }
    }
}


pub struct UIinput{
    str_buffer:Rc<RefCell<String>>
}
impl UIrender for UIinput{
    fn draw(&self) -> Option<&FrameBuffer> {
        todo!()
    }

    fn fast_draw(&self, window: &mut Window) {
        todo!()
    }

    fn update(&mut self,window: &mut Window,key_stream:&mut KeyStream) {
        todo!()
    }
}
impl UIlayout for UIinput{
    fn size(&self) -> (f32, f32) {
        todo!()
    }

    fn set_pos(&mut self, pos: (f32, f32)) {
        todo!()
    }
}
