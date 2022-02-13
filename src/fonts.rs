use crate::cpu::*;
pub const C8_CLASSIC_FONT:&'static str = 
concat!(
    "F0909090F0",
    "2060202070",
    "F010F080F0",
    "F010F010F0",
    "9090F01010",
    "F080F010F0",
    "F080F090F0",
    "F010204040",
    "F090F090F0",
    "F090F010F0",
    "F090F09090",
    "E090E090E0",
    "F0808080F0",
    "E0909090E0",
    "F080F080F0",
    "F080F08080");


pub struct FontPack{
    pub (crate)fonts:MemoryStick
}
impl Default for FontPack {
    #[inline]
    fn default()->Self {
        Self{
            fonts:MemoryStick::alloced(0xFF)
        }
    }
}

impl FontPack {
    #[inline]
    pub fn empty() ->Self {
        Self::default()
    }
    #[inline]
    pub fn with_font(font:&str)->Self{
        let mut font_pack = Self::default();
        font_pack.load_font(font);
        font_pack
    }
    #[inline]
    pub fn load_font(&mut self,hex:&str){
        load_font(&mut self.fonts, hex)
    }
    #[inline]
    pub fn load_charfont(&mut self,ch:u8,hex:&str){
        load_charfont(&mut self.fonts, ch, hex)
    }
    #[inline]
    pub fn classic()->Self{
        Self::with_font(C8_CLASSIC_FONT)
    }
}

