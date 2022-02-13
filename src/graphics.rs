





pub struct PixMap {
    pub map:Box<[[u8;64];32]>,
    pub has_updates:bool
}

impl PixMap {
    pub fn pack() ->PixMap {
        Self {
            map:Box::new([[0;64];32]),
            has_updates:true
        }
    }
    #[inline]
    fn index(x:u8,y:u8)->(usize,usize) {
        ((x&63)as usize,(y&31)as usize)   
    }
    #[inline]
    fn set_map(&mut self,x:u8,y:u8) ->bool{
        self.has_updates = true;
        let (x,y) = Self::index(x, y);
        let b = self.map[y][x]>0; 
        self.map[y][x]^=1;
        b
    }
    #[inline]
    pub fn get(&self,x:u8,y:u8) ->u8{
        let (x,y) = Self::index(x, y);
        self.map[y as usize][x as usize]
    }
    #[inline]
    fn clear_map(&mut self,x:u8,y:u8)->bool {
        self.has_updates = true;
        let (x,y) = Self::index(x, y);
        let b = self.map[y][x] > 0;
        self.map[y][x] = 0;
        b
    }
    pub fn draw(&mut self,x:u8,y:u8)->bool {
        self.set_map(x,y)
    }
    pub fn clear(&mut self){
        self.has_updates = true;
        for i in &mut self.map.iter_mut() {
            for j in &mut i.iter_mut(){
                *j=0;
            }
        }
    }
    pub fn erase(&mut self,x:u8,y:u8) ->bool {
        let b = self.clear_map(x,y);
        b
    }
    pub fn decide(&mut self,x:u8,y:u8,bit:u8)->bool {
        if bit>0{
            self.draw(x, y)
        } else {
            //self.erase(x, y)
            false
        }
    }
    pub fn flush(&mut self){
        self.has_updates=false;
    }
    pub fn ready(&self)->bool{
        self.has_updates
    }
    pub fn force_redisplay(&mut self){
        self.has_updates = true
    }
}