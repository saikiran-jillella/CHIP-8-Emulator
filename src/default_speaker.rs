use rodio::{
    DeviceSinkBuilder, MixerDeviceSink, Player,
    source::{Source, SquareWave},
};

use crate::Speaker;

pub struct DefaultSpeaker {
    _stream: MixerDeviceSink,
    player: Player,
    active: bool,
}

impl DefaultSpeaker {
    pub fn new() -> Result<Self, String> {
        let stream = DeviceSinkBuilder::open_default_sink().map_err(|e| format!("Failed to open audio device: {e}"))?;
        let player = Player::connect_new(stream.mixer());
        let beep = SquareWave::new(600.0).amplify(0.12);

        player.append(beep);
        player.pause();

        Ok(Self { _stream: stream, player, active: false })
    }
}

impl Speaker for DefaultSpeaker {
    fn start(&mut self) {
        if self.active {
            return;
        }

        self.active = true;
        self.player.play();
    }

    fn stop(&mut self) {
        if !self.active {
            return;
        }

        self.active = false;
        self.player.pause();
    }
}
