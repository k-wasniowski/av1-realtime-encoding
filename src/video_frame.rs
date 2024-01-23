use std::time::SystemTime;

pub struct VideoFrame {
    pub timestamp: SystemTime,
    pub buffer: Vec<u8>,
}

impl VideoFrame {
    pub fn new(timestamp: SystemTime, buffer: Vec<u8>) -> Self {
        Self {
            timestamp,
            buffer,
        }
    }
}

#[derive(Debug)]
pub enum FrameType {
    Key,
    Inter,
    IntraOnly,
    Switch,
}

pub struct EncodedVideoFrame {
    pub timestamp: SystemTime,
    pub buffer: Vec<u8>,
    pub frame_type: FrameType,
}