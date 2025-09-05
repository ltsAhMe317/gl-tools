use std::{
    ffi::c_void, path::Path, ptr::{null, null_mut}
};

use rusty_ffmpeg::ffi::*;

use crate::gl_unit::{define::{TextureParm, TextureType}, texture::{Texture, Texture2D, TextureWrapper}};

pub struct Video {
    pkt: *mut AVPacket,
    frame: *mut AVFrame,
    fmt_ctx: *mut AVFormatContext,
    decoder_ctx: *mut AVCodecContext,
    video_stream_id: i32,
    tex:Option<TextureWrapper<Texture2D>>,
    buffer:Option<*mut c_void>
}
impl Video {
    pub fn new(path: impl AsRef<Path>) -> Self {
        unsafe {
            let mut fmt_ctx = null_mut();
            let path_buf = path.as_ref().to_str().unwrap().as_bytes().as_ptr() as *const i8;
            if avformat_open_input(&mut fmt_ctx, path_buf, null(), null_mut()) < 0 {
                panic!("can't open video stream.");
            }
            if avformat_find_stream_info(fmt_ctx, null_mut()) < 0 {
                panic!("can't find stream info.");
            }

            //find decoder
            let decoder = null_mut();
            let fmt_ctx_now = *fmt_ctx;
            let mut video_stream_id = 0;
            //get video stream id
            for index in 0..fmt_ctx_now.nb_streams {
                if (*((*(*fmt_ctx_now.streams.wrapping_offset(index as isize))).codecpar))
                    .codec_type
                    == AVMEDIA_TYPE_VIDEO
                {
                    video_stream_id = index as isize;
                    let code_id = (*((*(*fmt_ctx_now.streams.wrapping_offset(video_stream_id)))
                        .codecpar))
                        .codec_id;
                    *decoder = *avcodec_find_decoder(code_id);
                    break;
                }
            }
            if decoder == null_mut() {
                panic!("no video stream");
            }
            let decoder_ctx = avcodec_alloc_context3(decoder);
            avcodec_parameters_to_context(
                decoder_ctx,
                (*(*fmt_ctx_now.streams.wrapping_offset(video_stream_id))).codecpar,
            );
            if avcodec_open2(decoder_ctx, decoder, null_mut()) < 0 {
                panic!("faild open decoder.");
            }

            let pkt = av_packet_alloc();
            let frame = av_frame_alloc();

            
            let mut temp =Self {
                pkt,
                frame,
                fmt_ctx,
                video_stream_id: video_stream_id as i32,
                decoder_ctx,
                tex: None,
                buffer: None,
            };
            temp.next_frame();
            temp
        }
    }
    pub fn next_frame(&mut self) {
        unsafe {
            av_read_frame(self.fmt_ctx, self.pkt);
            if (*self.pkt).stream_index == self.video_stream_id as i32 {
                if avcodec_send_packet(self.decoder_ctx, self.pkt) == 0 {
                    while avcodec_receive_frame(self.decoder_ctx, self.frame) == 0 {
                        
                    }
                }
            }
            // let frame = *self.frame;
            // let buffer=*self.buffer.get_or_insert(av_malloc(av_image_get_buffer_size(AV_PIX_FMT_RGB24, frame.width, frame.height, 32) as usize));
            // av_image_fill_arrays(frame.data.as_mut_ptr(), frame.linesize.as_mut_ptr(), buffer as *const u8,
            //          AV_PIX_FMT_RGB24, frame.width, frame.height, 32);

            // let tex =self.tex.get_or_insert(TextureWrapper(Texture2D::with_size(frame.width as u32, frame.height as u32,TextureType::RGB8, TextureParm::new())));
            // tex.send_date(TextureType::RGB8, 0, 0, frame.width, frame.height, *frame.data.as_ptr());
            todo!()
        }
        
    }
}
impl Drop for Video {
    fn drop(&mut self) {
        todo!()
    }
}
