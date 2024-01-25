// Source:
/*
#define SNDRV_PROTOCOL_VERSION(major, minor, subminor) (((major)<<16)|((minor)<<8)|(subminor))
#define SNDRV_PROTOCOL_MAJOR(version) (((version)>>16)&0xffff)
#define SNDRV_PROTOCOL_MINOR(version) (((version)>>8)&0xff)
#define SNDRV_PROTOCOL_MICRO(version) ((version)&0xff)
#define SNDRV_PROTOCOL_INCOMPATIBLE(kversion, uversion) \
    (SNDRV_PROTOCOL_MAJOR(kversion) != SNDRV_PROTOCOL_MAJOR(uversion) || \
     (SNDRV_PROTOCOL_MAJOR(kversion) == SNDRV_PROTOCOL_MAJOR(uversion) && \
       SNDRV_PROTOCOL_MINOR(kversion) != SNDRV_PROTOCOL_MINOR(uversion)))
 */

#[repr(transparent)]
pub struct Version(u32);

impl Version {
    pub fn new(major: u32, minor: u32, subminor: u32) -> Self {
        let val = ((major) << 16) | ((minor) << 8) | (subminor);
        Self(val)
    }

    pub fn from_raw(val: u32) -> Self {
        Self(val)
    }

    pub fn raw(&self) -> u32 {
        self.0
    }

    pub fn major(&self) -> u32 {
        (self.0 >> 16) & 0xffff
    }

    pub fn minor(&self) -> u32 {
        (self.0 >> 8) & 0xff
    }

    pub fn subminor(&self) -> u32 {
        self.0 & 0xff
    }

    pub fn check_protocol_incompatible(&self, version: Self) -> bool {
        let kmajor = self.major();
        let kminor = self.minor();

        let umajor = version.major();
        let uminor = version.minor();

        kmajor != umajor || kminor != uminor
    }
}

impl std::fmt::Debug for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Version")
            .field("major", &self.major())
            .field("minor", &self.minor())
            .field("subminor", &self.subminor())
            .finish()
    }
}
