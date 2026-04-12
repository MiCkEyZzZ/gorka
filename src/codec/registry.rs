use crate::{
    BdsPrn, BitReader, CdmaCodec, CdmaState, DopplerCodec, FdmaCodec, FdmaState, GalSvn, GloSlot,
    GorkaError, GpsPrn, MilliHz, RawBitWriter,
};

#[derive(Debug, Clone, Copy)]
pub enum GnssSystem {
    Glonass(GloSlot),
    Gps(GpsPrn),
    Galileo(GalSvn),
    Beidou(BdsPrn),
}

pub struct DopplerRegistry {
    fdma: FdmaState,
    gps: [CdmaState; 32],
    galileo: [CdmaState; 36],
    beidou: [CdmaState; 63],
}

impl DopplerRegistry {
    pub fn new() -> Self {
        Self {
            fdma: FdmaState::new(),
            gps: [CdmaState::new(); 32],
            galileo: [CdmaState::new(); 36],
            beidou: [CdmaState::new(); 63],
        }
    }

    pub fn reset(&mut self) {
        self.fdma.reset();

        for s in &mut self.gps {
            s.reset();
        }

        for s in &mut self.galileo {
            s.reset();
        }

        for s in &mut self.beidou {
            s.reset();
        }
    }

    #[inline]
    pub fn gps_idx(prn: GpsPrn) -> usize {
        (prn.get() - 1) as usize
    }

    #[inline]
    pub fn gal_idx(svn: GalSvn) -> usize {
        (svn.get() - 1) as usize
    }

    #[inline]
    pub fn bds_idx(prn: BdsPrn) -> usize {
        (prn.get() - 1) as usize
    }

    pub fn encode(
        &mut self,
        writer: &mut RawBitWriter,
        system: GnssSystem,
        value: MilliHz,
    ) -> Result<(), GorkaError> {
        match system {
            GnssSystem::Glonass(slot) => FdmaCodec::encode(writer, &mut self.fdma, value, slot),
            GnssSystem::Gps(prn) => {
                let state = &mut self.gps[Self::gps_idx(prn)];

                CdmaCodec::encode(writer, state, value, ())
            }
            GnssSystem::Galileo(svn) => {
                let state = &mut self.galileo[Self::gal_idx(svn)];

                CdmaCodec::encode(writer, state, value, ())
            }
            GnssSystem::Beidou(prn) => {
                let state = &mut self.beidou[Self::bds_idx(prn)];

                CdmaCodec::encode(writer, state, value, ())
            }
        }
    }

    pub fn decode(
        &mut self,
        reader: &mut BitReader,
        system: GnssSystem,
    ) -> Result<MilliHz, GorkaError> {
        match system {
            GnssSystem::Glonass(slot) => FdmaCodec::decode(reader, &mut self.fdma, slot),
            GnssSystem::Gps(prn) => {
                let state = &mut self.gps[Self::gps_idx(prn)];

                CdmaCodec::decode(reader, state, ())
            }
            GnssSystem::Galileo(svn) => {
                let state = &mut self.galileo[Self::gal_idx(svn)];

                CdmaCodec::decode(reader, state, ())
            }
            GnssSystem::Beidou(prn) => {
                let state = &mut self.beidou[Self::bds_idx(prn)];

                CdmaCodec::decode(reader, state, ())
            }
        }
    }
}

impl Default for DopplerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
