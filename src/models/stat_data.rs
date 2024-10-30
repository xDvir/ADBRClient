use std::error::Error;

#[derive(Debug)]
pub struct StatData {
    mode: u32,
    #[allow(dead_code)]
    size: u32,
    mtime: i64,
}

impl StatData {
    pub fn from_bytes(data: &[u8]) -> Result<Self, Box<dyn Error>> {
        if data.len() != 12 {
            println!("Invalid stat data length: expected 12, got {}", data.len());
            return Err("Invalid stat data length".into());
        }

        Ok(StatData {
            mode: u32::from_le_bytes(data[0..4].try_into()?),
            size: u32::from_le_bytes(data[4..8].try_into()?),
            mtime: u32::from_le_bytes(data[8..12].try_into()?) as i64,
        })
    }

    pub fn mode(&self) -> u32 {
        self.mode
    }

    #[allow(dead_code)]
    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn mtime(&self) -> i64 {
        self.mtime
    }
}