use tokio::sync::mpsc;

#[derive(Debug)]
pub struct ChannelPacket<T> {
    pub sender: mpsc::Sender<ChannelPacket<T>>, // I don't know why this works
    pub data: T
}