use std::result::Result;

use bincode;
use serde_json;

use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;

use super::*;

impl<'a> Gba<'a> {
    pub(super) fn check_save(&mut self, key: Scancode, _ctrl: bool) {
        use self::Scancode::*;
        let index = match key {
            Num0 => 0,
            Num1 => 1,
            Num2 => 2,
            Num3 => 3,
            Num4 => 4,
            Num5 => 5,
            Num6 => 6,
            Num7 => 7,
            Num8 => 8,
            Num9 => 9,
            _ => 10,
        };
        if index == 10 {
            return
        }
        let mut path = self.opts.save_file.to_os_string();
        let ext = if self.opts.json_save { "json" } else { "sav" };
        path.push(format!("{}.{}", index, ext));
        match File::create(Path::new(&path)) {
            Ok(file) => {
                if self.opts.json_save {
                    serde_json::to_writer_pretty(&file, self).unwrap();
                } else {
                    bincode::serialize_into(&file, self).unwrap();
                }
                info!("Saved file {:?}", path);
            },
            Err(err) => error!("Failed to create save state: {}", err),
        }
    }
}

impl<'a> Serialize for Gba<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("gba_rs::Gba", 4)?;
        s.serialize_field("cpu", &self.cpu)?;
        s.serialize_field("mmu", &self.mmu)?;
        s.serialize_field("io", &self.io)?;
        s.serialize_field("ppu", &self.ppu)?;
        s.end()
    }
}
