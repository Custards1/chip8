use crate::cpu::*;
use std::sync::{Arc,Mutex};
use crate::errors::*;
use crate::keyboard::{Keyboard,Key};
use crate::graphics::PixMap;
use crate::sound::Sound;
use crate::aux::Auxillary;
use crate::fonts::{FontPack,C8_CLASSIC_FONT};
pub const C8_TIMER_RATE:std::time::Duration = std::time::Duration::from_micros(16700);
pub const C8_CPU_RATE:std::time::Duration = std::time::Duration::from_micros(2000);

macro_rules! x_in_xkk {
    ($inst:expr) => {
        (($inst>>8)&0xF) as u8
    };
}
macro_rules! kk_in_xkk {
    ($inst:expr) => {
        ($inst&0xFF) as u8
    };
}
macro_rules! x_in_xy {
    ($inst:expr) => {
        x_in_xkk!($inst)
    };
}
macro_rules! y_in_xy {
    ($inst:expr) => {
        (($inst>>4)&0xF) as u8
    };
}
macro_rules! last_nibble {
    ($inst:expr) => {
        ($inst&0xF) as u8
    };
}





pub struct Chip8{
    pub cpu:DefaultCpu,
    state:CpuState,
    cosmic:bool,
    needs_key:bool,
    last_exec:std::time::Instant
}


#[inline]
pub fn chip8_cpu(cosmic:bool,graphics_pack:PixMap,sound_pack:Sound)->Chip8 {
    Chip8::new(
        DefaultCpu::new(MemoryStick::open(), 
                        Auxillary::pack(graphics_pack, sound_pack)),
                        cosmic)
}

#[inline]
pub fn custom_chip8(cosmic:bool,font:&str)->Chip8 {
    chip8_cpu(cosmic, PixMap::pack(),Sound::pack()).with_font(font)
}

#[inline]
pub fn default_chip8_cpu()->Chip8 {
    custom_chip8(false, C8_CLASSIC_FONT)
}

#[inline]
pub fn cosmic_chip8_cpu()->Chip8 {
    custom_chip8(true, C8_CLASSIC_FONT)
}



#[derive(Clone)]
pub struct CpuState {
    executing:Arc<Mutex<bool>>,
    delay_timer:Arc<Mutex<u8>>,
    sound_timer:Arc<Mutex<u8>>,
} 

impl CpuState {
    #[inline]
    pub fn new()->CpuState{
        Self{
            executing:Arc::new(Mutex::new(false)),
            delay_timer:Arc::new(Mutex::new(0)),
            sound_timer:Arc::new(Mutex::new(0)),
        }
    }
    pub fn stop(&self)->bool{
        match self.executing.lock(){
            Ok(mut a)=>{*a=false;true},
            _=>false
        }
    }
    pub fn start(&self)->bool{
        match self.executing.lock(){
            Ok(mut a)=>{*a=true;true},
            _=>false
        }
    }
    pub fn is_running(&self)->Option<bool>{
        match self.executing.lock(){
            Ok(a)=>Some(*a),
            _=>None
        }
    }
}

impl Chip8 {
    #[inline]
    pub fn new(cpu:DefaultCpu,cosmic:bool)->Self{
        Self{
            cpu:cpu,
            state:CpuState::new(),
            cosmic:cosmic,
            needs_key:false,
            last_exec:std::time::Instant::now()
        }
    }
    #[inline]
    pub fn keyboard(&self)->&Keyboard{
        return self.cpu.aux().keyboard()
    }
    #[inline]
    pub fn keyboard_mut(&mut self)->&mut Keyboard{
        return self.cpu.aux_mut().keyboard_mut()
    }
    
    #[inline] 
    pub fn with_font_pack(mut self,font:FontPack)->Self{
        self.using_font_pack(font);
        self
    }
    #[inline] 
    pub fn with_font(mut self,font:&str)->Self{
        self.using_font(font);
        self
    }
    #[inline]
    pub fn using_font_pack(&mut self,font:FontPack) {
        self.cpu.memory_mut().fill(0x0,font.fonts.arena())
    }
    #[inline]
    pub fn using_font(&mut self,font:&str) {
        load_font(self.cpu.memory_mut(),font)
    }
    #[inline]
    pub fn set_char_font(&mut self,ch:u8,font:&str) {
        load_charfont(self.cpu.memory_mut(),ch,font)
    }
    #[inline]
    pub fn load_program(&mut self,hex:&str) {
        self.cpu.memory_mut().load_hex(0x200, hex);
    }
    #[inline]
    pub fn load_program_binary(&mut self,hex:&[u8]) {
        self.cpu.memory_mut().load_binary(0x200, hex);
    }
    #[inline]
    pub fn cpu_state(&self)->CpuState{
        self.state.clone()
    }
    #[inline]
    pub fn graphics(&self)->&PixMap{
        self.cpu.aux().graphics()
    }
    #[inline]
    pub fn graphics_mut(&mut self)->&mut PixMap{
        self.cpu.aux_mut().graphics_mut()
    }
    fn delay_timer(status:Arc<Mutex<bool>>,timer:Arc<Mutex<u8>>,wait_time:std::time::Duration) {
        loop {
            let now = std::time::Instant::now();
            let ok = match status.lock(){
                Ok(a)=>*a,
                _=>false
            };
            if !ok{
                break;
            }
            match timer.lock() {
                Ok(mut time)=>if *time>0{
                    *time-=1
                },
                _=>{}
            }
            let end = std::time::Instant::now()-now;
            if end < wait_time {
                std::thread::sleep(wait_time)
            }
        }
    }
    fn sound_timer(status:Arc<Mutex<bool>>,timer:Arc<Mutex<u8>>,mut sound_pack:Sound,wait_time:std::time::Duration) {
        loop {
            let now = std::time::Instant::now();
            let ok = match status.lock(){
                Ok(a)=>*a,
                _=>false
            };
            if !ok{
                break;
            }
            match timer.lock() {
                Ok(mut time)=>if *time>0{
                    *time-=1;
                    sound_pack.safeplay()
                    
                } else {
                    sound_pack.safepause()
                    
                },
                _=>{
                    sound_pack.safepause()
                    
                }
            }
            let end = std::time::Instant::now()-now;
            if end < wait_time {
                std::thread::sleep(wait_time)
            }
      }
    }
    fn open_sound(&self,clock_speed:std::time::Duration)->std::thread::JoinHandle<()> {
        let _st:Arc<Mutex<u8>> = self.state.sound_timer.clone();
        let _et:Arc<Mutex<bool>> = self.state.executing.clone();
        let sound = self.cpu.aux().sound();
        std::thread::spawn(move||{
            Self::sound_timer(_et, _st, sound, clock_speed);    
        })
    }
    fn open_delay(&self,clock_speed:std::time::Duration)->std::thread::JoinHandle<()> {
        let _st = self.state.delay_timer.clone();
        let _et = self.state.executing.clone();
        std::thread::spawn(move||{
            Self::delay_timer(_et, _st, clock_speed);
        })
    }
    fn wait_for_key_handler(&mut self,instruction:u16)->bool{
        let key = self.cpu.aux().keyboard().all_pressed().next();
        match key {
            Some(key)=>{
                *self.cpu.register_mut(x_in_xy!(instruction))=key.key.into_byte();
                self.needs_key=false;
                true
            }
            _=>{
                self.needs_key=true;
                false
            }
        }
    }
    pub fn execute_step(&mut self)->Error {
        match self.state.is_running(){
            Some(is_r)=>{
                if is_r{
                    match self.execute_instruction() {
                        Ok(_)=>{},
                        Err(e)=>return e
                    }
                    let end = std::time::Instant::now()-self.last_exec;
                    if C8_CPU_RATE > end {
                        std::thread::sleep(C8_CPU_RATE-end);
                    }
                    self.last_exec = std::time::Instant::now();
                }
                Error::None
            }
            _=>return Error::ExecutionLocked
        }
       
    }
    pub fn start(&mut self)->Result<(std::thread::JoinHandle<()>,std::thread::JoinHandle<()>)> {
        if !self.state.start() {
            return Err(Error::ExecutionLocked);
        }
        self.cpu.jump(0x200);
        let sound = self.open_sound(C8_TIMER_RATE.clone());
        let delay = self.open_delay(C8_TIMER_RATE.clone());
        Ok((sound,delay))
    }
    pub fn close(&mut self,sound:std::thread::JoinHandle<()>,delay:std::thread::JoinHandle<()>) {
        self.state.stop();
        let _ = sound.join();
        let _ = delay.join();
    }
    // pub fn execute(&mut self)->Error {
    //     if !self.state.start() {
    //         return Error::ExecutionLocked;
    //     }
    //     self.cpu.jump(0x200);
    //     let sound = self.open_sound(C8_TIMER_RATE.clone());
    //     let delay = self.open_delay(C8_TIMER_RATE.clone());
    //     let mut errors =None;
    //     loop {
    //         let now = std::time::Instant::now();
    //         match self.state.is_running(){
    //             Some(is_r)=>{
    //                 if !is_r{
    //                     break;
    //                 }
    //             }
    //             _=>{errors = Some(Error::ExecutionLocked);break}
    //         }
    //         match self.execute_instruction() {
    //             Ok(_)=>{},
    //             Err(e)=>{errors = Some(e);break}
    //         }
    //         let end = std::time::Instant::now()-now;
    //         if C8_CPU_RATE > end {
    //             std::thread::sleep(C8_CPU_RATE-end);
    //         }
    //     }
    //     self.state.stop();
    //     let _ = sound.join();
    //     let _ = delay.join();
    //     match errors {
    //         Some(err)=>err,
    //         _=>Error::None
    //     }
    // }
    pub fn execute_instruction(&mut self)->Result<()> {
        let instruction = self.cpu.instruction();
        self.cpu.inc_instruction();
        //println!("{:04X}: {:04X}",self.cpu.program_counter(),instruction);
        
        let _ = match instruction {
            0x00E0=>self.cpu.aux_mut().graphics_mut().clear(),
            0x00EE=>match self.cpu.ret(){
                true=>{},
                false=>return Err(Error::StackUnderflow)
            }
            0x00FD=>{},
            _=>match (instruction>>12)&0xF {
                0x0|0x1=>self.cpu.jump(instruction),
                0x2=>self.cpu.call(instruction),
                0x3=>{
                    if *self.cpu.register(x_in_xkk!(instruction)) == kk_in_xkk!(instruction) {
                        self.cpu.inc_instruction()
                    }
                }
                0x4=>{
                    if *self.cpu.register(x_in_xkk!(instruction)) != kk_in_xkk!(instruction) {
                        self.cpu.inc_instruction()
                    }
                }
                0x5=>{
                    if last_nibble!(instruction) != 0 {
                        return Err(Error::InvalidInstruction);
                    } else {
                        if *self.cpu.register(x_in_xy!(instruction)) == *self.cpu.register(y_in_xy!(instruction)) {
                            self.cpu.inc_instruction()
                        }
                    }
                }
                0x6=>{
                    *self.cpu.register_mut(x_in_xkk!(instruction))=kk_in_xkk!(instruction);
                }
                0x7=>{
                    *self.cpu.register_mut(x_in_xkk!(instruction))+=kk_in_xkk!(instruction);
                }
                0x8=>match last_nibble!(instruction){
                    0x0=>{
                        let byte = *self.cpu.register(y_in_xy!(instruction));
                        *self.cpu.register_mut(x_in_xy!(instruction))=byte;
                    }
                    0x1=>{
                        let byte = *self.cpu.register(y_in_xy!(instruction));
                        *self.cpu.register_mut(x_in_xy!(instruction))|=byte; 
                    }
                    0x2=>{
                        let byte = *self.cpu.register(y_in_xy!(instruction));
                        *self.cpu.register_mut(x_in_xy!(instruction))&=byte;
                    }
                    0x3=>{
                        let byte = *self.cpu.register(y_in_xy!(instruction));
                        *self.cpu.register_mut(x_in_xy!(instruction))^=byte;
                    }
                    0x4=>{
                        let res:u16 = *self.cpu.register(x_in_xy!(instruction)) as u16 +*self.cpu.register(y_in_xy!(instruction)) as u16;
                        *self.cpu.register_mut(x_in_xy!(instruction))=(res&0xFF) as u8;
                        if res&0xFF00 > 0 {
                            *self.cpu.register_mut(0xF)=1
                        } else {
                            *self.cpu.register_mut(0xF)=0
                        }
                    }
                    0x5=>{
                        let x = x_in_xy!(instruction);
                        let y = y_in_xy!(instruction);
                        if *self.cpu.register(x) > *self.cpu.register(y) {
                            *self.cpu.register_mut(0xF) =1
                        } else {
                            *self.cpu.register_mut(0xF) =0
                        }
                        let res:u16 = *self.cpu.register(x) as u16 -*self.cpu.register(y) as u16;
                        *self.cpu.register_mut(x_in_xy!(instruction))=(res&0xFF) as u8;
                    }
                    0x6=>{
                        let x = x_in_xy!(instruction);
                        let y = y_in_xy!(instruction);
                        if self.cosmic {
                            *self.cpu.register_mut(x)=*self.cpu.register(y);
                        }
                        if *self.cpu.register(x)&0x1 > 0 {
                            *self.cpu.register_mut(0xF) =1
                        } else {
                            *self.cpu.register_mut(0xF) =0
                        }
                        *self.cpu.register_mut(x)>>=1
                    }
                    0x7=>{
                        let x = x_in_xy!(instruction);
                        let y = y_in_xy!(instruction);
                        if *self.cpu.register(y) > *self.cpu.register(x) {
                            *self.cpu.register_mut(0xF) =1
                        } else {
                            *self.cpu.register_mut(0xF) =0
                        }
                        let res:u16 = *self.cpu.register(y) as u16 -*self.cpu.register(x) as u16;
                        *self.cpu.register_mut(x_in_xy!(instruction))=(res&0xFF) as u8;
                    }
                    0xE=>{
                        let x = x_in_xy!(instruction);
                        let y = y_in_xy!(instruction);
                        if self.cosmic {
                            *self.cpu.register_mut(x)=*self.cpu.register(y);
                        }
                        if *self.cpu.register(x)&0x80 > 0 {
                            *self.cpu.register_mut(0xF) =1
                        } else {
                            *self.cpu.register_mut(0xF) =0
                        }
                        *self.cpu.register_mut(x)<<=1
                    }
                    _=>{
                        return Err(Error::InvalidInstruction);
                    }
                }
                0x9=>{
                    if last_nibble!(instruction) != 0 {
                        return Err(Error::InvalidInstruction);
                    } else {
                        let x = x_in_xy!(instruction);
                        let y = y_in_xy!(instruction);
                        if *self.cpu.register(x) != *self.cpu.register(y) {
                            self.cpu.inc_instruction()
                        }
                    }
                }
                0xA=>{
                    //println!("OI: {:04X}, OV: {:04X}",self.cpu.registerI(),self.cpu.memory().derefrence(self.cpu.registerI()));
                    self.cpu.set_registerI(instruction&0xFFF);
                    //println!("I: {:04X}, V: {:04X}",self.cpu.registerI(),self.cpu.memory().derefrence(self.cpu.registerI()))
                }
                0xB=>{
                    if self.cosmic {
                        self.cpu.jump(instruction);
                        self.cpu.jumpn(*self.cpu.register(0x0) as u16);
                    } else {
                        self.cpu.jump(instruction);
                        self.cpu.jumpn(*self.cpu.register(x_in_xy!(instruction))as u16);
                    }
                }
                0xC=>{
                    *self.cpu.register_mut(x_in_xkk!(instruction))= self.cpu.random()&kk_in_xkk!(instruction);
                }
                0xD=>{
                    let height = last_nibble!(instruction);
                    let x = *self.cpu.register(x_in_xy!(instruction))&63;
                    let mut y = *self.cpu.register(y_in_xy!(instruction))&31;
                    let mut set = false;
                    let i = self.cpu.registerI();
                    let max = i + (height as u16);
                    for index in i..max {
                        let mut sprite = *self.cpu.memory().derefrence(index);
                        for xi in x..64 {
                            if self.cpu.aux_mut().graphics_mut().decide(xi,y,sprite&0x80) {
                                set = true;
                            }
                            sprite<<=1
                        }
                        y+=1;
                        if y >=32 {
                            break;
                        }
                    }
                    if set{
                        *self.cpu.register_mut(0xF)=1
                    } else {
                        *self.cpu.register_mut(0xF)=0
                    }
                }
                0xE=>{
                    match instruction&0xFF {
                        0x9E=>{
                           let byte = *self.cpu.register(x_in_xy!(instruction))&0xF;
                           if self.cpu.aux().keyboard().is_pressed(Key::from_byte(byte).unwrap()) {
                               self.cpu.inc_instruction()
                           }
                        }
                        0xA1=>{
                            let byte = *self.cpu.register(x_in_xy!(instruction))&0xF;
                            if !self.cpu.aux().keyboard().is_pressed(Key::from_byte(byte).unwrap()) {
                                self.cpu.inc_instruction()
                            }
                        }
                        _=>{
                            return Err(Error::InvalidInstruction);
                        }
                    }
                }
                0xF=> match instruction&0xFF{
                    0x07=>{
                        let wv = match self.state.delay_timer.lock() {
                            Ok(v)=>*v,
                            _=>{
                                return Err(Error::DelayTimerLocked);
                            }
                        };
                        *self.cpu.register_mut(x_in_xy!(instruction)) = wv;
                    }
                    0x0A=>{
                       if !self.wait_for_key_handler(instruction){
                           println!("Waitin");
                           self.cpu.dec_instruction()
                       }
                    }
                    0x15=>{
                        let byte = *self.cpu.register(x_in_xkk!(instruction));
                        match self.state.delay_timer.lock(){
                            Ok(mut dl)=>{
                                *dl = byte;
                            }
                            _=>{
                                return Err(Error::DelayTimerLocked);
                            }
                        }
                    }
                    0x18=>{
                        let byte = *self.cpu.register(x_in_xkk!(instruction));
                        match self.state.sound_timer.lock(){
                            Ok(mut dl)=>{
                                *dl = byte;
                            }
                            _=>{
                                return Err(Error::SoundTimerLocked);
                            }
                        }
                    }
                    0x1E=>{
                        let byte = *self.cpu.register(x_in_xkk!(instruction)) as u16 + self.cpu.registerI();
                        if !self.cosmic {
                            if byte > 0xFFF {
                                *self.cpu.register_mut(0xF)=1
                            } else {
                                *self.cpu.register_mut(0xF)=0
                            }
                        }
                        self.cpu.set_registerI(byte);
                    }
                    0x29=>{
                        let byte = (*self.cpu.register(x_in_xkk!(instruction))&0xF) as u16;
                        self.cpu.set_registerI(byte*5)
                    }
                    0x33=>{
                        let val = self.cpu.register(x_in_xkk!(instruction));
                        let i = self.cpu.registerI();
                        let temp_alloc = format!("{:03}",val);
                        let string = temp_alloc.as_bytes();
                        *self.cpu.memory_mut().derefrence_mut(i) =   string[0]-'0' as u8;
                        *self.cpu.memory_mut().derefrence_mut(i+1) = string[1]-'0' as u8;
                        *self.cpu.memory_mut().derefrence_mut(i+2) = string[2]-'0' as u8;
                    }
                    0x65=>{
                    
                        let x = x_in_xy!(instruction);
                        let addr = self.cpu.registerI();
                        self.cpu.fill_registers(addr, x);                  
                        if self.cosmic {
                            self.cpu.set_registerI(addr+(x as u16))
                        }
                    }
                    0x55=>{
                        let x = x_in_xy!(instruction);
                        let addr = self.cpu.registerI();
                        self.cpu.load_registers(addr, x);                   
                        if self.cosmic {
                            self.cpu.set_registerI(addr+(x as u16))
                        }
                    }
                    _=>{
                        return Err(Error::InvalidInstruction);
                    }
                }
                _=>{
                    return Err(Error::InvalidInstruction);
                }
            }
        };
        Ok(())     
    }
  
}





