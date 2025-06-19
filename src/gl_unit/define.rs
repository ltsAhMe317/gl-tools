use gl::types::GLenum;
use paste::paste;
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
        gl::COLOR_ATTACHMENT0 + $id
    };
    (TextureUnit,$id:tt) => {
        gl::TEXTURE + $id
    };
}

macro_rules! enums_creater {
    ($($name:ident { $($var:ident),* $(,)?})*)=>{
        $(
            pub enum $name {
                $($var,)*
            }

        impl $name {
           pub const fn as_gl(self) ->GLenum{
                match self {
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

}

enums_index_creater! {
    Attachment{
        Color
    }
    Texture{
        Unit
    }
}
