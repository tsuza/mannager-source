pub mod depotdownloader;
pub mod metamod;
pub mod sourcemod;

#[derive(Debug, Clone, PartialEq)]
pub enum SourceEngineVersion {
    Source1,
    Source2,
}

impl From<SourceEngineVersion> for u32 {
    fn from(value: SourceEngineVersion) -> Self {
        match value {
            SourceEngineVersion::Source1 => 1,
            SourceEngineVersion::Source2 => 2,
        }
    }
}
