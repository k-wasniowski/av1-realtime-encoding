mod camera;
mod video_frame;
mod encoder;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::SystemTime;
use log::info;

fn main() {
    let (sender, receiver): (Sender<video_frame::VideoFrame>, Receiver<video_frame::VideoFrame>) = mpsc::channel();

    let camera_info = camera::run(sender);

    info!("Camera width: {}", camera_info.width);
    info!("Camera height: {}", camera_info.height);
    info!("Camera framerate: {}", camera_info.framerate);

    let (encoded_video_sender, encoded_video_receiver): (Sender<video_frame::EncodedVideoFrame>, Receiver<video_frame::EncodedVideoFrame>) = mpsc::channel();

    encoder::run(camera_info, receiver, encoded_video_sender);

    loop {
        let encoded_video_frame = match encoded_video_receiver.recv() {
            Ok(encoded_video_frame) => encoded_video_frame,
            Err(e) => {
                println!("Failed to receive encoded video frame - {}", e);
                continue;
            }
        };

        let current_time = SystemTime::now();

        let age = current_time.duration_since(encoded_video_frame.timestamp).unwrap();

        println!("Received encoded video frame with age: {}, type: {:?}", age.as_millis(), encoded_video_frame.frame_type);
    }
}