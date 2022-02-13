#[derive(Clone,Copy)]
pub struct Sound {
    is_running:bool,
}

impl Sound {
    pub fn pack()->Self{
        Self{
            is_running:false,
        }
    }
    pub fn play(&mut self) {
        self.is_running=true;
    }
    pub fn pause(&mut self) {
        self.is_running=false;
    }
    #[inline]
    pub fn is_playing(&self)->bool{
        self.is_running
    }
    pub fn safeplay(&mut self) {
        if !self.is_playing() {
            self.play()
        }
    }
    pub fn safepause(&mut self) {
        if self.is_playing() {
            self.pause()
        }
    }
}

