use std::fmt;
use std::result::Result;

use bincode;
use zstd;

use serde::de;
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
            return;
        }
        let mut path = self.opts.save_file.to_os_string();
        path.push(format!("{}.sav", index));
        match File::create(Path::new(&path)) {
            Ok(file) => {
                let mut writer = zstd::Encoder::new(&file, 1).unwrap();
                bincode::serialize_into(&mut writer, self).unwrap();
                info!("Saved file {:?}", path);
            }
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

impl<'de> Deserialize<'de> for Gba<'static> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct GbaVisitor;
        impl<'de> Visitor<'de> for GbaVisitor {
            type Value = Gba<'static>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Gba")
            }

            fn visit_seq<V: SeqAccess<'de>>(self, mut seq: V) -> Result<Gba<'static>, V::Error> {
                let cpu = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let mmu = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let io = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let ppu = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;

                unsafe {
                    let mut gba: Gba<'static> = mem::uninitialized();
                    ptr::write(&mut gba.cpu, cpu);
                    ptr::write(&mut gba.mmu, mmu);
                    ptr::write(&mut gba.io, io);
                    ptr::write(&mut gba.ppu, ppu);
                    Ok(gba)
                }
            }
        }

        const FIELDS: &'static [&'static str] = &["cpu", "mmu", "io", "ppu"];
        deserializer.deserialize_struct("gba_rs::Gba", FIELDS, GbaVisitor)
    }
}
