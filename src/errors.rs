// error1.rs
use std::fmt;
use std::error::Error as SError;

#[repr(u8)]
#[derive(Debug,PartialEq,PartialOrd,Clone,Copy)]
pub enum Error {
    None=0,
    StackUnderflow=1,
    ExecutionLocked=2,
    DelayTimerLocked=3,
    SoundTimerLocked=4,
    UnhookedKeyBoard=5,
    InvalidInstruction=6
}
impl Error {
    #[inline]
    pub fn locked(&self)->bool{
        match &self{
            Error::DelayTimerLocked|Error::ExecutionLocked|Error::SoundTimerLocked=>true,
            _=>false
        }
    }
}



impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self{
            Error::StackUnderflow=>write!(f,"Stack underflow! Attempted to pop non-existant element from stack! Last Call trace: 0xDEADBEEF"),
            Error::ExecutionLocked=>write!(f,"Exectution handle locked!"),
            Error::DelayTimerLocked=>write!(f,"Delay handle locked!"),
            Error::SoundTimerLocked=>write!(f,"Sound handle locked!"),
            Error::UnhookedKeyBoard=>write!(f,"Keyboard disconnected!"),
            Error::InvalidInstruction=>write!(f,"Invalid instruction: 0xDEADBEEF"),
            _=>write!(f,"No error")
        }
    }
}
impl SError for Error {
    fn description(&self) -> &str {
        match self {
            Error::None=>"No error",
            _=>"chip 8 error"
        }
    }
}
pub type Result<I> = std::result::Result<I,Error>;


