pub mod data;
pub(crate) mod internal_data;

use std::fmt;

use super::*;
use crate::{
    driver::tasks::message::{UdpTxMessage, WsMessage},
    model::payload::{ClientDisconnect, Speaking},
    tracks::{TrackHandle, TrackState},
};
pub use data as context_data;
use data::*;
use flume::Sender;
use internal_data::*;
use xsalsa20poly1305::XSalsa20Poly1305 as Cipher;

pub struct CipherWrapper(Cipher);

impl fmt::Debug for CipherWrapper {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("CipherWrapper").finish()
    }
}

impl std::ops::Deref for CipherWrapper {
    type Target = Cipher;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Information about which tracks or data fired an event.
///
/// [`Track`] events may be local or global, and have no tracks
/// if fired on the global context via [`Driver::add_global_event`].
///
/// [`Track`]: crate::tracks::Track
/// [`Driver::add_global_event`]: crate::driver::Driver::add_global_event
#[derive(Debug)]
#[non_exhaustive]
pub enum EventContext<'a> {
    /// Track event context, passed to events created via [`TrackHandle::add_event`],
    /// [`EventStore::add_event`], or relevant global events.
    ///
    /// [`EventStore::add_event`]: EventStore::add_event
    /// [`TrackHandle::add_event`]: TrackHandle::add_event
    Track(&'a [(&'a TrackState, &'a TrackHandle)]),
    /// Speaking state update, typically describing how another voice
    /// user is transmitting audio data. Clients must send at least one such
    /// packet to allow SSRC/UserID matching.
    SpeakingStateUpdate(Speaking),
    /// Speaking state transition, describing whether a given source has started/stopped
    /// transmitting. This fires in response to a silent burst, or the first packet
    /// breaking such a burst.
    SpeakingUpdate(SpeakingUpdateData),
    /// Opus audio packet, received from another stream.
    VoicePacket(VoiceData<'a>),
    /// Telemetry/statistics packet, received from another stream.
    RtcpPacket(RtcpData<'a>),
    /// Fired whenever a client disconnects.
    ClientDisconnect(ClientDisconnect),
    /// Fires when this driver successfully connects to a voice channel.
    DriverConnect(
        ConnectData<'a>,
        Sender<UdpTxMessage>,
        Sender<WsMessage>,
        CipherWrapper,
    ),
    /// Fires when this driver successfully reconnects after a network error.
    DriverReconnect(
        ConnectData<'a>,
        Sender<UdpTxMessage>,
        Sender<WsMessage>,
        CipherWrapper,
    ),
    /// Fires when this driver fails to connect to, or drops from, a voice channel.
    DriverDisconnect(DisconnectData<'a>),
}

pub enum CoreContext {
    SpeakingStateUpdate(Speaking),
    SpeakingUpdate(InternalSpeakingUpdate),
    VoicePacket(InternalVoicePacket),
    RtcpPacket(InternalRtcpPacket),
    ClientDisconnect(ClientDisconnect),
    DriverConnect(
        InternalConnect,
        Sender<UdpTxMessage>,
        Sender<WsMessage>,
        Cipher,
    ),
    DriverReconnect(
        InternalConnect,
        Sender<UdpTxMessage>,
        Sender<WsMessage>,
        Cipher,
    ),
    DriverDisconnect(InternalDisconnect),
}

impl<'a> CoreContext {
    pub(crate) fn to_user_context(&'a self) -> EventContext<'a> {
        use CoreContext::*;

        match self {
            SpeakingStateUpdate(evt) => EventContext::SpeakingStateUpdate(*evt),
            SpeakingUpdate(evt) => EventContext::SpeakingUpdate(SpeakingUpdateData::from(evt)),
            VoicePacket(evt) => EventContext::VoicePacket(VoiceData::from(evt)),
            RtcpPacket(evt) => EventContext::RtcpPacket(RtcpData::from(evt)),
            ClientDisconnect(evt) => EventContext::ClientDisconnect(*evt),
            DriverConnect(evt, tx, ws, cipher) => EventContext::DriverConnect(
                ConnectData::from(evt),
                tx.clone(),
                ws.clone(),
                CipherWrapper(cipher.clone()),
            ),
            DriverReconnect(evt, tx, ws, cipher) => EventContext::DriverReconnect(
                ConnectData::from(evt),
                tx.clone(),
                ws.clone(),
                CipherWrapper(cipher.clone()),
            ),
            DriverDisconnect(evt) => EventContext::DriverDisconnect(DisconnectData::from(evt)),
        }
    }
}

impl EventContext<'_> {
    /// Retreive the event class for an event (i.e., when matching)
    /// an event against the registered listeners.
    pub fn to_core_event(&self) -> Option<CoreEvent> {
        use EventContext::*;

        match self {
            SpeakingStateUpdate(_) => Some(CoreEvent::SpeakingStateUpdate),
            SpeakingUpdate(_) => Some(CoreEvent::SpeakingUpdate),
            VoicePacket(_) => Some(CoreEvent::VoicePacket),
            RtcpPacket(_) => Some(CoreEvent::RtcpPacket),
            ClientDisconnect(_) => Some(CoreEvent::ClientDisconnect),
            DriverConnect(_, _, _, _) => Some(CoreEvent::DriverConnect),
            DriverReconnect(_, _, _, _) => Some(CoreEvent::DriverReconnect),
            DriverDisconnect(_) => Some(CoreEvent::DriverDisconnect),
            _ => None,
        }
    }
}
