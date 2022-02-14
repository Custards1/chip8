#[derive(Copy, Clone,PartialEq,PartialOrd,Debug)]
#[repr(u8)]
pub enum KeyEventKind {
    KeyRelease=0,
    KeyPress=1,
    KeyDefault=2,
}


#[derive(Copy, Clone,Debug)]
#[repr(u8)]
pub enum Key {
    A=0,
    B=1,
    C=2,
    D=3,
    E=4,
    F=5,
    Zero=6,
    One=7,
    Two=8,
    Three=9,
    Four=10,
    Five=11,
    Six=12,
    Seven=13,
    Eight=14,
    Nine=15
}
impl Key {
    pub fn from_byte(byte:u8)->Option<Self>{
        if byte <16{
            Some(unsafe{std::mem::transmute(byte)})
        }else{
            None
        }
    }
    pub fn into_byte(&self)->u8 {
        *self as u8
    }
}
#[derive(Clone,Copy,Debug)]
pub struct KeyEvent {
    pub key:Key,
    pub kind:KeyEventKind
}
impl KeyEvent {
    #[inline]
    pub fn new(key:Key,kind:KeyEventKind)->Self {
        KeyEvent{
            key,
            kind
        }
    }
    #[inline]
    pub fn from_keyd(keys:u8,kind:KeyEventKind)->Self {
        KeyEvent{
            key:Key::from_byte(keys&0xF).unwrap(),
            kind:kind
        }
    }
    #[inline]
    pub fn raw(keys:u8,kind:u8)->Self {
        Self::from_keyd(keys&0xF, if kind == 0 {
            KeyEventKind::KeyPress
        } else {
            KeyEventKind::KeyRelease
        })
    }
}

#[derive(Clone,Copy)]
pub struct Keyboard([KeyEventKind;16]);
impl From<[KeyEventKind;16]> for Keyboard {
    #[inline]
    fn from(a:[KeyEventKind;16])->Self{
        return Self(a)
    }
}
impl Keyboard {
    pub fn new()->Self{
        return Self([KeyEventKind::KeyDefault;16])
    }
    #[inline]
    pub fn action(&mut self,event:KeyEvent){
        self.0[event.key.into_byte() as usize] = event.kind
    }
    #[inline]
    pub fn commit(&mut self,event:impl std::iter::Iterator<Item=KeyEvent>){
        for act in event {
            self.action(act)
        }
    }
    #[inline]
    pub fn press(&mut self,key:Key){
       self.action(KeyEvent::new(key, KeyEventKind::KeyPress))
    }
    #[inline]
    pub fn is_pressed(&self,key:Key) ->bool{
       return self.0[key.into_byte() as usize] == KeyEventKind::KeyPress;
    }
    #[inline]
    pub fn is_currenly_released(&self,key:Key) ->bool{
       return self.0[key.into_byte() as usize] == KeyEventKind::KeyRelease;
    }
    #[inline]
    pub fn is_released(&self,key:Key) ->bool{
       return match self.0[key.into_byte() as usize]{
           KeyEventKind::KeyRelease|KeyEventKind::KeyDefault=>true,
           _=>false
        }
    }
    #[inline]
    pub fn all_events(&self) ->impl std::iter::Iterator<Item=KeyEvent>+'_ {
        return self.0.iter().enumerate().map(|(i,x)| KeyEvent::from_keyd(i as u8,*x))
    }
    
    #[inline]
    pub fn all_pressed(&self) ->impl std::iter::Iterator<Item=KeyEvent>+'_ {
        return self.all_events().filter(|x|x.kind == KeyEventKind::KeyPress)
    }
    #[inline]
    pub fn all_currenly_released(&self) ->impl std::iter::Iterator<Item=KeyEvent>+'_ {
        return self.all_events().filter(|x|x.kind == KeyEventKind::KeyRelease)
    }
    #[inline]
    pub fn all_released(&self) ->impl std::iter::Iterator<Item=KeyEvent>+'_ {
        return self.all_events().filter(|x|x.kind == KeyEventKind::KeyRelease ||
                                         x.kind == KeyEventKind::KeyDefault)
    }
    
    

    
    #[inline]
    pub fn release(&mut self,key:Key){
       self.action(KeyEvent::new(key, KeyEventKind::KeyRelease))
    }
    #[inline]
    pub fn reset(&mut self){
       for i in &mut self.0 {
           if *i == KeyEventKind::KeyRelease {
               *i = KeyEventKind::KeyDefault
           }
       }
    }
    
    
    
}

