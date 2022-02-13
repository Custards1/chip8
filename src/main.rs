use chip8::{chip8::{default_chip8_cpu,cosmic_chip8_cpu},graphics::PixMap,cpu::*,aux::*,keyboard::{Key,KeyEvent,KeyEventKind},errors::Error};


use sdl2::event::{Event,WindowEvent};
use sdl2::keyboard::{Scancode,Keycode};
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use clap::{App,AppSettings,Arg};
#[derive(Copy,Clone)]
struct WindowSize {
    x:u32,
    y:u32
}

fn norm_key(key:Scancode)->Option<Key> {
    Some(match key {
        
        Scancode::Num1=>Key::One,
        Scancode::Num2=>Key::Two,
        Scancode::Num3=>Key::Three,
        Scancode::Num4=>Key::C,
        Scancode::Q=>Key::Four,
        Scancode::W=>Key::Five,
        Scancode::E=>Key::Six,
        Scancode::R=>Key::D,
        Scancode::A=>Key::Seven,
        Scancode::S=>Key::Eight,
        Scancode::D=>Key::Nine,
        Scancode::F=>Key::E,

        Scancode::Z=>Key::A,
        Scancode::X=>Key::Zero,
        Scancode::C=>Key::B,
        Scancode::V=>Key::F,
        _=>return None  
    })
}
fn norm_key_event(key:Scancode,kind:KeyEventKind)->Option<KeyEvent> {
    Some(KeyEvent::new(norm_key(key)?,kind))
}
fn color(byte:u8)->u8 {
    match byte {
        0=>0,
        _=>255
    }
}
pub fn main() -> Result<(), String> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
    .about("A chip8 emulator to play some killer games!")
    .version(env!("CARGO_PKG_VERSION"))
    .author("Blake Brown")
    .arg(
        Arg::new("game")
        .required(true)
        .help("the path to the chip8 rom to run")
    )
    .arg(
        Arg::new("cosmic")
        .short('c')
        .long("cosmic")
        .takes_value(false)
        .help("run the emulator in cosmic vip mode (default: false)")
    )
    .get_matches();
    let game = match matches.value_of("game") {
        Some(a)=>Ok(a),
        _=>Err("No game provided")
    }?;
    let mut chip8 = match matches.is_present("cosmic"){
        true=>cosmic_chip8_cpu(),
        _=>default_chip8_cpu()
    };
    let rom = std::fs::read(game).map_err(|e|e.to_string())?;
    chip8.cpu.memory_mut().load_big_binary_instructions(0x200,&rom);
    chip8.cpu.memory().dump(0x200..0x220);
    let (sound,delay) = chip8.start().map_err(|e| e.to_string())?;
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let mut size = WindowSize{x:800,y:600};
    let window = video_subsystem
        .window("rust-sdl2 demo: Video", size.x, size.y)
        .resizable()
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 64, 32)
        .map_err(|e| e.to_string())?;
    let mut display = |pix:&mut PixMap,size:WindowSize|->Result<(),String>{
        if pix.ready() {
            texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for y in 0..32 {
                    for x in 0..64 {
                        let offset = y * pitch + x * 3;
                        let col:u8 = color(pix.get(x as u8,y as u8));
                        buffer[offset] = col;
                        buffer[offset + 1] = col;
                        buffer[offset + 2] = col;
                    }
                }
            })?;
            canvas.clear();
            canvas.copy(&texture, None, Some(Rect::new(0, 0, size.x, size.y)))?;
           pix.flush()
        }
        
        canvas.present();
        Ok(())
    };
    display(chip8.graphics_mut(),size)?;
    let mut event_pump = sdl_context.event_pump()?;
    
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyUp{scancode,..}|Event::KeyDown{scancode,..}=>{
                    match scancode{
                        Some(scancode)=>match norm_key_event(scancode, match event{
                            Event::KeyUp{..}=>KeyEventKind::KeyRelease,
                            _=>KeyEventKind::KeyPress,
                        }) {
                            Some(key)=>{
                                println!("Clackin {:?}",key);
                                chip8.keyboard_mut().action(key);
                            }
                            _=>{

                            }
                        }
                        _=>{

                        }
                    }
                }
                Event::Window { win_event, .. }=>{
                    if let WindowEvent::Resized(w, h) = win_event {
                        size.x=w as u32;
                        size.y=h as u32;
                    }
                }
                _ => {}
            }
        }
        // The rest of the game loop goes here...
        let err= chip8.execute_step();
        if err != Error::None {
            println!("{}",err);
            break
        }
        display(chip8.graphics_mut(),size)?;
        chip8.keyboard_mut().reset()
    }
    chip8.close(sound,delay);
    Ok(())
}