use gl::types::GLenum;
use paste::paste;

use crate::TypeGL;
macro_rules! gl_enum {
    (Zero) => {
        gl::ZERO
    };
    (One) => {
        gl::ONE
    };
    (SrcColor) => {
        gl::SRC_COLOR
    };
    (DstColor) => {
        gl::DST_COLOR
    };
    (OneMinusSrcColor) => {
        gl::ONE_MINUS_SRC_COLOR
    };
    (OneMinusDstColor) => {
        gl::ONE_MINUS_DST_COLOR
    };
    (SrcAlpha) => {
        gl::SRC_ALPHA
    };
    (DstAlpha) => {
        gl::DST_ALPHA
    };
    (ConstColor) => {
        gl::CONSTANT_COLOR
    };
    (ConstAlpha) => {
        gl::CONSTANT_ALPHA
    };
    (OneMinusSrcAlpha) => {
        gl::ONE_MINUS_SRC_ALPHA
    };
    (OneMinusDstAlpha) => {
        gl::ONE_MINUS_DST_ALPHA
    };
    (Line) => {
        gl::LINE
    };
    (Point) => {
        gl::POINT
    };
    (Fill) => {
        gl::FILL
    };
    (Front) => {
        gl::FRONT
    };
    (Back) => {
        gl::BACK
    };
    (Texture2D) => {
        gl::TEXTURE_2D
    };
    (Texture1D) => {
        gl::TEXTURE_1D
    };
    (AttachmentColor,$id:tt) => {
        (gl::COLOR_ATTACHMENT0 + $id)
    };
    (TextureUnit,$id:tt) => {
        (gl::TEXTURE0 + $id)
    };

    (RGBA8) => {
        (gl::RGBA, gl::UNSIGNED_BYTE)
    };
    (RGB8) => {
        (gl::RGB, gl::UNSIGNED_BYTE)
    };
    (RED8) => {
        (gl::RED, gl::UNSIGNED_BYTE)
    };
    (RGBA16) => {
        (gl::RGBA, gl::UNSIGNED_SHORT)
    };
    (RGB16) => {
        (gl::RGB, gl::UNSIGNED_SHORT)
    };
    (RED16) => {
        (gl::RED, gl::UNSIGNED_SHORT)
    };
    (RGBA32) => {
        (gl::RGBA, gl::FLOAT)
    };
    (RGB32) => {
        (gl::RGB, gl::FLOAT)
    };
    (RED32) => {
        (gl::RED, gl::FLOAT)
    };
    (Vertex) => {
        gl::ARRAY_BUFFER
    };

    (Element) => {
        gl::ELEMENT_ARRAY_BUFFER
    };
    (Static) => {
        gl::STATIC_DRAW
    };
    (Stream) => {
        gl::STREAM_DRAW
    };
    (Dynamic) => {
        gl::DYNAMIC_DRAW
    };
    (Nearest) => {
        gl::NEAREST
    };
    (Linear) => {
        gl::LINEAR
    };
    (Repeat) => {
        gl::REPEAT
    };
    (MirroredRepeat) => {
        gl::MIRRORED_REPEAT
    };
    (ClampEdge) => {
        gl::CLAMP_TO_EDGE
    };
    (ClampBorder) => {
        gl::CLAMP_TO_BORDER
    };
}

macro_rules! enums_creater {
        ($($name:ident {$($var:ident),* $(,)?})*)=>{
        $(
            #[derive(Clone,Copy,PartialEq)]
            pub enum $name{
                $($var,)*
            }
            impl $name{
                pub const fn as_gl(self)->GLenum{
                    match self{
                        $(Self::$var => gl_enum!($var),)*
                    }
                }
            }
        )*
    };
}
macro_rules! two_enums_creater {
        ($($name:ident {$($var:ident),* $(,)?})*)=>{
        $(
            pub enum $name{
                $($var,)*
            }
            impl $name{
                pub const fn as_gl(self)->(GLenum,GLenum){
                    match self{
                        $(Self::$var => gl_enum!($var),)*
                    }
                }
            }
        )*
    };
}

macro_rules! enums_index_creater {
    ($($name:ident { $($var:ident),* $(,)?})*) => {
        $(
            pub enum $name {
                $($var(u32),)*
            }
        paste!{
            impl $name {
               pub const fn as_gl(self) ->GLenum{
                    match self {
                        $(Self::$var(size) => gl_enum!([<$name $var>],size))*
                    }
                }
            }
        }
        )*

    };
}
macro_rules! setter_gen {
    ($name:ident{$($var:ident:$var_type:ty),* }) => {
        pub struct $name {
            $(pub $var:$var_type,)*
        }
        impl $name {
            $(
                pub fn $var(mut self,value:$var_type)->Self{
                    self.$var = value;
                    self
                }
            )*
        }
    };
}
enums_creater! {
    Blend {
        Zero,
        One,
        SrcColor,
        DstColor,
        OneMinusSrcColor,
        OneMinusDstColor,
        OneMinusSrcAlpha,
        OneMinusDstAlpha,
        SrcAlpha,
        DstAlpha,
        ConstColor,
        ConstAlpha,
    }
    PolygonMode {
        Fill,
        Line,
        Point,
    }
    Face{
        Front,
        Back
    }
    BufferTarget{
        Vertex,
        Element
    }
    BufferUsage{
        Dynamic,
        Stream,
        Static
    }
    Filter{
        Linear,
        Nearest
    }
    TextureWarpMode{
        Repeat,
        MirroredRepeat,
        ClampBorder,
        ClampEdge
    }
}
two_enums_creater! {
    TextureType {
        RGBA8,
        RGB8,
        RED8,
        RGBA16,
        RGB16,
        RED16,
        RGBA32,
        RGB32,
        RED32,
    }
}
enums_index_creater! {
    Attachment{
        Color
    }
    Texture{
        Unit
    }

}
setter_gen! {
    VertexArrayAttribPointerGen {
        index: u32,
        once_size: i32,
        is_normalized: bool,
        stride: i32,
        pointer: usize
    }
}
impl VertexArrayAttribPointerGen {
    pub const fn new<T: TypeGL>(index: u32, once_size: i32) -> Self {
        Self {
            index,
            once_size,
            is_normalized: false,
            stride: once_size * size_of::<T>() as i32,
            pointer: 0,
        }
    }
}

setter_gen! {
TextureParm {
    min_filter: Filter,
    mag_filter: Filter,
    wrap_s: TextureWarpMode,
    wrap_t: TextureWarpMode,
    once_load_size: i32
}
}

impl TextureParm {
    pub const fn new() -> Self {
        Self {
            min_filter: Filter::Nearest,
            mag_filter: Filter::Nearest,
            wrap_s: TextureWarpMode::ClampBorder,
            wrap_t: TextureWarpMode::ClampBorder,
            once_load_size: 4,
        }
    }
}
