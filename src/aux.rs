use crate::{graphics::{PixMap},sound::Sound,keyboard::*};

pub struct Auxillary {
    graphics_pack:PixMap,
    sound_pack:Sound,
    keyboard:Keyboard
}

impl Auxillary {
    pub fn pack(graphics_pack:PixMap,sound_pack:Sound) ->Self{
        Self{
            graphics_pack:graphics_pack,
            sound_pack:sound_pack,
            keyboard:Keyboard::new(),
        }
    }
    #[inline]
    pub fn keyboard(&self)->&Keyboard {
        &self.keyboard
    }
    #[inline]
    pub fn keyboard_mut(&mut self)->&mut Keyboard {
        &mut self.keyboard
    }
    
    #[inline]
    pub(crate) fn graphics(&self)->&PixMap{
        &self.graphics_pack
    }
    #[inline]
    pub(crate) fn graphics_mut(&mut self)->&mut PixMap{
        &mut self.graphics_pack
    }
    #[inline]
    pub(crate) fn sound(&self)->Sound {
        self.sound_pack
    }
}