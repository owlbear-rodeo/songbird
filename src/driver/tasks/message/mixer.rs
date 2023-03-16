#![allow(missing_docs)]

use super::{Interconnect, UdpRxMessage, UdpTxMessage, WsMessage};

use crate::{
    driver::{Bitrate, Config, CryptoState},
    tracks::Track,
};
use flume::Sender;
use xsalsa20poly1305::XSalsa20Poly1305 as Cipher;

pub struct MixerConnection {
    pub cipher: Cipher,
    pub crypto_state: CryptoState,
    pub udp_rx: Sender<UdpRxMessage>,
    pub udp_tx: Sender<UdpTxMessage>,
}

impl Drop for MixerConnection {
    fn drop(&mut self) {
        let _ = self.udp_rx.send(UdpRxMessage::Poison);
        let _ = self.udp_tx.send(UdpTxMessage::Poison);
    }
}

pub enum MixerMessage {
    AddTrack(Track),
    SetTrack(Option<Track>),

    SetBitrate(Bitrate),
    SetConfig(Config),
    SetMute(bool),
    SetDisabled(bool),

    SetConn(MixerConnection, u32),
    Ws(Option<Sender<WsMessage>>),
    DropConn,

    ReplaceInterconnect(Interconnect),
    RebuildEncoder,

    Poison,
}
