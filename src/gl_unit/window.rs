use std::{
    cell::{OnceCell, RefCell, UnsafeCell},
    hash::{Hash, Hasher},
    sync::{Mutex, OnceLock},
};

use glfw::{
    Context, Glfw, GlfwReceiver, PWindow, SwapInterval, WindowEvent, WindowHint, WindowMode,
};

pub struct Timer {
    pub update: f64,
    pub delta: f64,
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

impl Timer {
    pub const fn new() -> Self {
        Self {
            delta: 0f64,
            update: 0f64,
        }
    }
    pub const fn update(&mut self, delta: f64) {
        self.update = delta - self.delta;
        self.delta = delta;
    }

    pub const fn fps(&self) -> f64 {
        1f64 / self.update
    }
}

pub struct Window {
    pub window: PWindow,
    pub events: GlfwReceiver<(f64, WindowEvent)>,
    pub delta_count: Timer,
}

impl PartialEq<Self> for Window {
    fn eq(&self, other: &Self) -> bool {
        self.window.window_ptr().eq(&other.window.window_ptr())
    }
}
impl Eq for Window {}
impl Hash for Window {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.window.window_ptr().hash(state)
    }
}

unsafe impl Sync for GLFWwrapper {}
unsafe impl Send for GLFWwrapper {}
struct GLFWwrapper(Glfw);

thread_local! {
    static GLFW: OnceCell<RefCell<GLFWwrapper>> = OnceCell::new();
}
unsafe impl Send for Window {}
unsafe impl Sync for Window {}
impl Window {
    pub fn new(w: usize, h: usize, name: &str, is_full: bool) -> Window {
        GLFW.with(|glfw| {
            let glfw_lock = &mut glfw
                .get_or_init(|| RefCell::new(GLFWwrapper(glfw::init_no_callbacks().unwrap())))
                .borrow_mut()
                .0;

            glfw_lock.window_hint(WindowHint::Visible(false));
            let mut window: (PWindow, GlfwReceiver<(f64, WindowEvent)>);
            match is_full {
                true => {
                    window = glfw_lock.with_primary_monitor(|glfw_lock, sc| {
                        let scs = sc.unwrap();
                        let show = scs.get_video_mode().unwrap();
                        glfw_lock
                            .create_window(
                                show.width,
                                show.height,
                                name,
                                WindowMode::FullScreen(scs),
                            )
                            .unwrap()
                    });
                }
                false => {
                    window = glfw_lock
                        .create_window(w as u32, h as u32, name, WindowMode::Windowed)
                        .unwrap();
                    let vid_mode = glfw_lock
                        .with_primary_monitor(|_glfw, sc| sc.unwrap().get_video_mode().unwrap());
                    window.0.set_pos(
                        ((vid_mode.width as f32 / 2f32) - w as f32 / 2f32) as i32,
                        ((vid_mode.height as f32 / 2f32) - h as f32 / 2f32) as i32,
                    );
                }
            }
            window.0.make_current();
            glfw_lock.set_swap_interval(SwapInterval::None);
            window.0.glfw.make_context_current(None);
            window.0.set_char_polling(true);
            window.0.set_key_polling(true);
            Self {
                events: window.1,
                delta_count: Timer::new(),
                window: window.0,
            }
        })
    }
    pub fn update(&mut self) -> bool {
        // let now_time = self.window.0.glfw.get_time();
        //
        // if now_time - self.last_update >= 1f64/self.fps as f64 {
        self.delta_count.update(self.window.glfw.get_time());
        self.window.glfw.poll_events();
        self.window.swap_buffers();
        // self.last_update = now_time;
        self.window.should_close()
        // }
    }
    pub fn current(&mut self) {
        self.window.make_current();
    }
    pub fn view_port(&self) {
        unsafe {
            let size = self.window.get_framebuffer_size();
            gl::Viewport(0, 0, size.0, size.1);
        }
    }
    pub fn get_char(&self, char: char) -> bool {
        if let Some(event) = self.window_event() {
            return event == WindowEvent::Char(char);
        }
        false
    }
    pub fn is_resize(&self) -> bool {
        match self.window_event() {
            Some(WindowEvent::Size(_, _)) => true,
            _ => false,
        }
    }
    pub fn window_event(&self) -> Option<WindowEvent> {
        Some(self.events.receive()?.1)
    }
}
