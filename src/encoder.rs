use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use log::{debug, error};
use rav1e::config::SpeedSettings;
use rav1e::*;
use crate::camera::CameraInfo;
use crate::video_frame;
use crate::video_frame::FrameType;

fn convert_frame_type(frame_type: rav1e::prelude::FrameType) -> video_frame::FrameType {
    match frame_type {
        rav1e::prelude::FrameType::KEY => FrameType::Key,
        rav1e::prelude::FrameType::INTER => FrameType::Inter,
        rav1e::prelude::FrameType::INTRA_ONLY => FrameType::IntraOnly,
        rav1e::prelude::FrameType::SWITCH => FrameType::Switch,
    }
}

pub fn run(
    camera_info: CameraInfo,
    video_source: Receiver<video_frame::VideoFrame>,
    encoded_video_sender: Sender<video_frame::EncodedVideoFrame>,
) {
    thread::spawn(move || {
        let width = camera_info.width;
        let height = camera_info.height;
        let speed_settings = SpeedSettings::from_preset(10);
        let encoder_config = EncoderConfig {
            width,
            height,
            low_latency: true,
            still_picture: false,
            speed_settings,
            ..Default::default()
        };

        let config = Config::new()
            .with_encoder_config(encoder_config.clone())
            .with_threads(16);

        let mut ctx: Context<u8> = match config.new_context() {
            Ok(ctx) => ctx,
            Err(e) => panic!("Failed to create rav1e context - {}", e),
        };

        loop {
            let frame = match video_source.recv() {
                Ok(frame) => frame,
                Err(e) => {
                    error!("Failed to receive video frame - {}", e);
                    continue;
                }
            };

            let mut frame_to_encode = ctx.new_frame();

            for p in &mut frame_to_encode.planes {
                let stride = (encoder_config.width + p.cfg.xdec) >> p.cfg.xdec;
                p.copy_from_raw_u8(&frame.buffer, stride, 1);
            }

            match ctx.send_frame(frame_to_encode) {
                Ok(_) => debug!("Successfully sent frame to the rav1e encoder"),
                Err(e) => error!("Failed to send frame to the rav1e encoder - {}", e)
            }

            let pkt = match ctx.receive_packet() {
                Ok(pkt) => pkt,
                Err(_) => {
                    continue;
                }
            };

            let encoded_video_frame = video_frame::EncodedVideoFrame {
                timestamp: frame.timestamp,
                buffer: pkt.data.to_vec(),
                frame_type: convert_frame_type(pkt.frame_type),
            };

            match encoded_video_sender.send(encoded_video_frame) {
                Ok(_) => debug!("Successfully sent frame to the encoded video sink"),
                Err(e) => error!("Failed to send frame to the encoded video sink - {}", e)
            }
        }
    });
}