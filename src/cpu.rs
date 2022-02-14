use crate::aux::Auxillary;
use byteorder::{BigEndian,ReadBytesExt};
use oorandom as rand;
pub const CHIP8_MEM_SIZE:usize = 0xFFF;
pub const CHIP8_REGISTER_COUNT:usize = 0xF+1;
pub const CHIP8_PROGRAM_START:u16=0x200;
macro_rules! sized_rptr {
    ($ptr:expr) => {
        ($ptr&0xF) as usize
    };
}
macro_rules! sized_ptr {
    ($ptr:expr) => {
        ($ptr&0xFFF)
    };
}

pub struct MemoryStick{
    memory:Vec<u8>,
    callstack:Vec<u16>,
    size:u16
}

impl MemoryStick {
    #[inline]
    pub fn open()->Self{
        Self::alloced(CHIP8_MEM_SIZE)
    }
    #[inline]
    pub fn alloced(max:usize)->Self{
        Self {
            memory:vec![0u8;max],
            callstack:vec![],
            size:max as u16
        }
    }
    fn _load_hex(&mut self,addr:&mut u16,iaddr:&mut usize,hex:&str)->bool{
        let mut ic:u8=0;
        let mut first =false;
        let mut done=false;
        for ch in hex.chars().nth(*iaddr){
            *iaddr+=1;
            if ch >='0' && ch <='9'{
                if !first {
                    ic = ((ch as u8 -'0' as u8)&0xF)<<4;
                    first=true;
                } else {
                    ic |= (ch as u8 -'0' as u8)&0xF;
                    *self.derefrence_mut(*addr) = ic;
                    done=true;
                    break;
                }
            }
            else if ch>='a' && ch<='z' {
                if !first {
                    ic = ((ch as u8 -'a' as u8)&0xF)<<4;
                    first=true;
                } else {
                    ic |= (ch as u8 -'a' as u8)&0xF;
                    *self.derefrence_mut(*addr) = ic;
                    done=true;
                    break;
                }
            }
            else if ch >='A' && ch<='Z' {
                if !first {
                    ic = ((ch as u8 -'A' as u8)&0xF)<<4;
                    first=true;
                } else {
                    ic |= (ch as u8 -'A' as u8)&0xF;
                    *self.derefrence_mut(*addr) = ic;
                    done=true;
                    break;
                }
            }
            
        }
        if done {
            *addr+=1;
        }
        done
    }
    pub fn load_memory(&mut self,addr:u16,other:MemoryStick) {
        self.load_binary(addr, &other.memory)
    }
    pub fn dump(&self,rng:std::ops::Range<u16>) {
        let mut skip = false;
        for idx in rng {
            if !skip{
                println!("{:04X}\t{:04X}",idx,self.u16(idx))
            }
            skip=!skip
        }
    }
    #[inline]
    pub (crate) fn arena(&self)->&[u8] {
        &self.memory
    }
    
    #[inline]
    pub fn u16(&self,addr:u16)->u16 {
        ((*self.derefrence(addr) as u16)<<8)|(*self.derefrence(addr+1) as u16)
    }
    #[inline]
    pub fn load_hex(&mut self,addr:u16,hex:&str){
        let mut addr = addr;
        let mut iaddr = 0;
        while self._load_hex(&mut addr, &mut iaddr, hex){

        }
    }
    pub fn loadn_hex(&mut self,addr:u16,hex:&str,n:u16){
        let mut addr = addr;
        let mut iaddr = 0;
        let mut count =0;
        while self._load_hex(&mut addr, &mut iaddr, hex){
            count+=1;
            if count >=n {
                break;
            }
        }
    }
    pub fn load_binary(&mut self,addr:u16,binary:&[u8]){
        let mut addr = addr;
        for byte in binary{
            *self.derefrence_mut(addr)=*byte;
            addr+=1;
        }
    }
    pub fn load_big_binary_instructions(&mut self,addr:u16,binary:&[u8]){
        let mut addr = addr;
        let mut cursor = std::io::Cursor::new(binary);
        while let Ok(byte) = cursor.read_u16::<BigEndian>(){
            *self.derefrence_mut(addr)=(byte>>8) as u8;
            *self.derefrence_mut(addr+1)=(byte&0xFF) as u8;
            addr+=2;
        }
    }
    pub fn load_little_binary_instructions(&mut self,addr:u16,binary:&[u8]){
        let mut addr = addr;
        let mut cursor = std::io::Cursor::new(binary);
        for byte in cursor.read_u16::<BigEndian>(){
            *self.derefrence_mut(addr)=(byte>>8) as u8;
            *self.derefrence_mut(addr+1)=(byte&0xFF) as u8;
            addr+=2;
        }
    }
    pub (crate)fn derefrence(&self,pointer:u16)->&u8 {
        self.memory.get((pointer&self.size) as usize).unwrap()
    }
    pub(crate) fn derefrence_mut(&mut self,pointer:u16)->&mut u8{
        self.memory.get_mut((pointer&self.size) as usize).unwrap()
    }
    pub fn fill(&mut self,base_ptr:u16,vals:&[u8]){
        let mut n =0;
        for val in vals{
            self.memory[(((base_ptr+n)&self.size)) as usize]=*val;
            n+=1;
        }
    }
    pub fn push(&mut self,ptr:u16){
        self.callstack.push(ptr)
    }
    pub fn pop(&mut self)->Option<u16>{
        self.callstack.pop()
    } 
}

#[inline]
pub fn load_font(memory:&mut MemoryStick,hex:&str){
    memory.loadn_hex(0x0, hex,16*5)
}
#[inline]
pub fn load_charfont(memory:&mut MemoryStick,ch:u8,hex:&str){
    memory.loadn_hex(((ch&0xF) as u16)*5, hex,5)
}




pub struct DenseCpu {
    pc:u16,
    regi:u16,
    registers:[u8;CHIP8_REGISTER_COUNT],
    aux_pack: Auxillary,
    memory:MemoryStick,
    rgen:rand::Rand32
}

impl DenseCpu {
    pub fn new(memory:MemoryStick,aux:Auxillary)-> Self {
        Self {
            pc:CHIP8_PROGRAM_START,
            regi:0,
            registers:[0u8;CHIP8_REGISTER_COUNT],
            memory:memory,
            aux_pack:aux,
            rgen:rand::Rand32::new(0xFCFB)
        }
    }
    #[inline]
    fn _fill_regestry(memory:&mut MemoryStick,addr:u16,registers:&[u8]){
        memory.fill(addr, registers)
    }
    #[inline]
    pub fn fill_registers(&mut self,mut addr:u16,reg:u8) {
        for regi in 0..=reg {
            //println!("{} regi reads {:04X} {:04X}",regi,*self.memory().derefrence(addr),addr);
            *self.register_mut(regi) = *self.memory().derefrence(addr);
            addr+=1
        }
    }
    #[inline]
    pub fn load_registers(&mut self,mut addr:u16,reg:u8) {
        for reg in 0..=reg {
            let b = *self.register(reg);
            *self.memory_mut().derefrence_mut(addr)=b;
            addr+=1;
        }
    }
    #[inline]
    pub fn registery(&self) ->&[u8] {
        &self.registers
    }
    #[inline]
    pub fn register(&self,ptr:u8) ->&u8 {
        self.registers.get(sized_rptr!(ptr)).unwrap()
    }
    #[inline]
    pub fn register_i(&self)->u16{
        self.regi
    }
    #[inline]
    pub fn set_register_i(&mut self,i:u16){
        self.regi = sized_ptr!(i)
    }
    #[inline]
    pub fn deref_i(&self)->&u8 {
        self.memory.derefrence(self.regi)
    }
    #[inline]
    pub fn deref_i_mut(&mut self)->&mut u8{
        self.memory.derefrence_mut(self.regi)
    }
    #[inline]
    pub fn program_counter(&self)->u16{
        self.pc
    }
    #[inline]
    pub fn instruction(&self)->u16 {
        return self.memory().u16(self.pc)
    }
    #[inline]
    pub fn inc_instruction(&mut self) {
       self.jump(self.pc+2)
    }
    #[inline]
    pub fn dec_instruction(&mut self) {
       self.jump(self.pc-2)
    }
    #[inline]
    pub(crate) fn register_mut(&mut self,ptr:u8) ->&mut u8 {
        self.registers.get_mut(sized_rptr!(ptr)).unwrap()
    }
    #[inline]
    pub fn memory(&self)->&MemoryStick{
        &self.memory
    }
    #[inline]
    pub fn memory_mut(&mut self)->&mut MemoryStick{
        &mut self.memory
    }
    #[inline]
    pub(crate) fn aux(&self)->&Auxillary{
        &self.aux_pack
    }
    #[inline]
    pub(crate) fn aux_mut(&mut self)->&mut Auxillary{
        &mut self.aux_pack
    }
    #[inline]
    pub fn inc(&mut self){
        self.jump(self.pc+1)
    }
    #[inline]
    pub fn dec(&mut self){
        self.jump(self.pc-1)
    }
    #[inline]
    pub fn jump(&mut self,addr:u16){
        self.pc=sized_ptr!(addr)
    }
    #[inline]
    pub fn jumpn(&mut self,n:u16){
        self.jump(self.pc+sized_ptr!(n))
    }
    #[inline]
    pub fn call(&mut self,addr:u16){
        self.memory.push(sized_ptr!(self.pc));
        self.jump(addr)
    }
    #[inline]
    pub fn ret(&mut self)->bool{
        match self.memory.pop(){
            Some(addr)=>{self.jump(addr);true},
            _=>false
        }
    }
    #[inline]
    pub fn random(&mut self)->u8 {
        let a = (((self.rgen.rand_u32()& 0xF)%4)*8)%25;
        (self.rgen.rand_u32()>>a) as u8
    }
    

}

pub type DefaultCpu = DenseCpu;