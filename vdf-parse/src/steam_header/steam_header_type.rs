#[derive(Default)]
pub struct SteamHeader {
    magic: u32,
    version: u32,
}


pub struct SteamHeaderBuilder {
    magic: u32,
    version: u32,
}

impl SteamHeader {
    fn new(magic: u32, version: u32) -> Self {
        Self {
            magic,
            version,
        }
    }
}

impl SteamHeaderBuilder {
    pub fn new() -> Self {
        Self {
            magic: 0,
            version: 1,
        }
    }

    pub fn magic(&mut self, magic: u32) -> &mut Self {
        self.magic = magic;
        self
    }

    pub fn version(&mut self, version: u32) -> &mut Self {
        self.version = version;
        self
    }

    pub fn build(&self) -> SteamHeader {
        SteamHeader::new(self.magic, self.version)
    }
    
}