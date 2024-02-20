use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use rand::prelude::*;
use sdl2::sys::SDL_CreateTexture;
use sdl2::rect::Rect;

const SCALE: i32 = 10;
const VIDEO_WIDTH: u8 = 64;
const VIDEO_HEIGHT: u8 = 32;

struct Chip8 {
    registers: [u8; 16],
    memory: [u8; 4096],
    index: u16,
    pc: u16,
    stack: [u16; 16],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [u8; 16],
    display: [u16; 64 * 32],
    opcode: u16,
}

impl Chip8 {
    const start_addr: u16 = 0x200;
    const fontset_addr: u16 = 0x50;

    fn init() -> Self {
        let mut memory = [0; 4096];
        let fontset: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];

        for (i, &byte) in fontset.iter().enumerate() {
            memory[Chip8::fontset_addr as usize + i] = byte;
        }

        Chip8 {
            registers: [0; 16],
            memory,
            index: 0,
            pc: Chip8::start_addr,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; 16],
            display: [0; 64 * 32],
            opcode: 0,
        }
    }
    fn rand_byte() -> u8 {
        rand::random()
    }

    fn load_rom(&mut self, rom_path: &str) {
        let rom = std::fs::read(rom_path).unwrap();
        for (i, byte) in rom.iter().enumerate() {
            self.memory[Chip8::start_addr as usize + i] = *byte;
            println!("byte: {}", byte);
        }
        println!("End of rom");
        println!("memory: {:?}", self.memory);
    }

    // CPU Instructions 
    fn op_00e0(&mut self) {
        self.display = [0; 64 * 32];
    }
    fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }
    fn op_1nnn(&mut self) {
        self.pc = self.opcode & 0x0FFF;
    }
    fn op_2nnn(&mut self) {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = self.opcode & 0x0FFF;
    }
    fn op_3xkk(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let byte = self.opcode & 0x00FF;
        if self.registers[vx as usize] == byte as u8 {
            self.pc += 2;
        }
    }
    fn op_4xkk(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let byte = self.opcode & 0x00FF;
        if self.registers[vx as usize] != byte as u8 {
            self.pc += 2;
        }
    }
    // SE vx, vy
    fn op_5xy0(&mut self) {
        let vx = self.opcode & 0x0F00 >> 8;
        let vy = self.opcode & 0x00F0 >> 4;

        if self.registers[vx as usize] == self.registers[vy as usize] {
            self.pc += 2;
        }
    }
    // LD vx, byte
    fn op_6xkk(&mut self) {
        let vx = self.opcode & 0x0F00 >> 8;
        let byte = self.opcode & 0x00FF;

        self.registers[vx as usize] = byte as u8;
    }
    // ADD vx, byte
    fn op_7xkk(&mut self) {
        let vx = self.opcode & 0x0F00 >> 8;
        let byte = self.opcode & 0x00FF;

        let (answer, overflow) = self.registers[vx as usize].overflowing_add(byte as u8);
        self.registers[vx as usize] = answer;
    }
    // LD vx, vy
    fn op_8xy0(&mut self) {
        let vx = self.opcode & 0x0F00 >> 8;
        let vy = self.opcode & 0x00F0 >> 4;

        self.registers[vx as usize] = self.registers[vy as usize];
    }
    // OR vx, vy
    fn op_8xy1(&mut self) {
        let vx = self.opcode & 0x0F00 >> 8;
        let vy = self.opcode & 0x00F0 >> 4;

        self.registers[vx as usize] |= self.registers[vy as usize];
    }
    // AND vx, vy
    fn op_8xy2(&mut self) {
        let vx = self.opcode & 0x0F00 >> 8;
        let vy = self.opcode & 0x00F0 >> 4;

        self.registers[vx as usize] &= self.registers[vy as usize];
    }
    // XOR vx, vy
    fn op_8xy3(&mut self) {
        let vx = self.opcode & 0x0F00 >> 8;
        let vy = self.opcode & 0x00F0 >> 4;

        self.registers[vx as usize] ^= self.registers[vy as usize];
    }
    // ADD vx, vy carry
    fn op_8xy4(&mut self) {
        let vx = self.opcode & 0x0F00 >> 8;
        let vy = self.opcode & 0x00F0 >> 4;

        let (result, overflow) = self.registers[vx as usize].overflowing_add(self.registers[vy as usize]);
        self.registers[vx as usize] = result;
        self.registers[0xF] = overflow as u8;
    }
    // SUB vx, vy
    fn op_8xy5(&mut self) {
        let vx = self.opcode & 0x0F00 >> 8;
        let vy = self.opcode & 0x00F0 >> 4;

        let (result, overflow) = self.registers[vx as usize].overflowing_sub(self.registers[vy as usize]);
        self.registers[vx as usize] = result;
        self.registers[0xF] = overflow as u8;
    }
    // SHR vx
    fn op_8xy6(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        
        self.registers[0xF] = self.registers[vx as usize] & 0x1;
        self.registers[vx as usize] >>= 1;
    }
    // SUBN vy, vx
    fn op_8xy7(&mut self){
        let vx = self.opcode & 0x0F00 >> 8;
        let vy = self.opcode & 0x00F0 >> 4;

        let (result, overflow) = self.registers[vy as usize].overflowing_sub(self.registers[vx as usize]);
        self.registers[vy as usize] = result;
        self.registers[0xF] = overflow as u8;
    }
    // SHL vx {vy,}
    fn op_8xye(&mut self) {
        let vx = self.opcode & 0x0F00 >> 8;

        self.registers[0xF] = (self.registers[vx as usize] & 0x80) >> 7;
        self.registers[vx as usize] <<= 1;
    }
    // SNE vx, vy
    fn op_9xy0(&mut self) {
        let vx = self.opcode & 0x0F00 >> 8;
        let vy = self.opcode & 0x00F0 >> 4;

        if self.registers[vx as usize] != self.registers[vy as usize] {
            self.pc += 2;
        }
    }
    // LD I, addr
    fn op_annn(&mut self) {
        let address: u16 = (self.opcode & 0x0FFF) as u16;

        self.index = address;
    }
    // JP v0, addr
    fn op_bnnn(&mut self) {
        let address: u16 = (self.opcode & 0x0FFF) as u16;

        self.pc = self.registers[0] as u16 + address;
    }
    // RND vx, byte
    fn op_cxkk(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let byte = (self.opcode & 0x00FF) as u8;

        self.registers[vx as usize] = Self::rand_byte() & byte;
    }
    // DRW vx, vy, nibble
    fn op_dxyn(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let vy = (self.opcode & 0x00F0) >> 4;
        let height = self.opcode & 0x000F;

        let x_pos = self.registers[vx as usize] % VIDEO_WIDTH;
        let y_pos = self.registers[vy as usize] % VIDEO_HEIGHT;
        println!("x_pos: {}, y_pos: {}", x_pos, y_pos);

        self.registers[0xF] = 0;

        for row in 0..height {
            let sprite = self.memory[(self.index + row) as usize];
            println!("sprite: {:x}", sprite);
            for col in 0..8 {
                let sprite_pixel = sprite & (0x80 >> col);
                let screen_pixel = self.display[(((y_pos as u16 + row) * VIDEO_WIDTH as u16) + (x_pos as u16 + col)) as usize];

                if sprite_pixel != 0 {
                    if screen_pixel == 0xFFFF {
                        self.registers[0xF] = 1;
                    }
                    self.display[(((y_pos as u16 + row) * VIDEO_WIDTH as u16) + (x_pos as u16 + col)) as usize] ^= 0xFFFF;
                }
            }
        }
    }
    
    // SKP vx
    fn op_ex9e(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let key = self.registers[vx as usize];

        if self.keypad[key as usize] != 0 {
            self.pc += 2;
        }
    }
    
    // SKNP vx
    fn op_exa1(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let key = self.registers[vx as usize];

        if self.keypad[key as usize] == 0 {
            self.pc += 2;
        }
    }

    // LD vx, DT
    fn op_fx07(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;

        self.registers[vx as usize] = self.delay_timer;
    }

    // LD vx, k
    fn op_fx0a(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;

        for i in 0..16 {
            if self.keypad[i] != 0 {
                self.registers[vx as usize] = i as u8;
                return;
            }
            else {
                self.pc -= 2;
            }
        }
    }

    // LD DT, vx
    fn op_fx15(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;

        self.delay_timer = self.registers[vx as usize];
    }

    // LD ST, vx
    fn op_fx18(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;

        self.sound_timer = self.registers[vx as usize];
    }

    // ADD I, vx
    fn op_fx1e(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;

        self.index += self.registers[vx as usize] as u16;
    }

    // LD F, vx
    fn op_fx29(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let digit = self.registers[vx as usize];

        self.index = Chip8::fontset_addr + (digit * 5) as u16;
    }

    // LD B, vx
    fn op_fx33(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let mut digit = self.registers[vx as usize];

        self.memory[(self.index + 2) as usize] = digit % 10;
        digit /= 10;

        self.memory[(self.index + 1) as usize] = digit % 10;
        digit /= 10;


        self.memory[(self.index) as usize] = digit % 10;
    }

    // LD [I], vx
    fn op_fx55(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;

        for i in 0..vx {
            self.memory[(self.index + i) as usize] = self.registers[i as usize];
        }
    }

    // LD vx, [I]
    fn op_fx66(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;

        for i in 0..vx {
            self.registers[i as usize] = self.memory[(self.index + i) as usize];
        }
    }
    
    fn run_opcode(&mut self) {

        match self.opcode {
            0xe0 => self.op_00e0(),
            0xee => self.op_00ee(),
            0x1000..=0x1FFF => self.op_1nnn(),
            0x2000..=0x2FFF => self.op_2nnn(),
            0x3000..=0x3FFF => self.op_3xkk(),
            0x4000..=0x4FFF => self.op_4xkk(),
            0x5000..=0x5FFF => self.op_5xy0(),
            0x6000..=0x6FFF => self.op_6xkk(),
            0x7000..=0x7FFF => self.op_7xkk(),
            0x8000..=0x8FFF => {
                match self.opcode & 0x000F {
                    0x0 => self.op_8xy0(),
                    0x1 => self.op_8xy1(),
                    0x2 => self.op_8xy2(),
                    0x3 => self.op_8xy3(),
                    0x4 => self.op_8xy4(),
                    0x5 => self.op_8xy5(),
                    0x6 => self.op_8xy6(),
                    0x7 => self.op_8xy7(),
                    0xE => self.op_8xye(),
                    _ => (),
                }
            },
            0x9000..=0x9FFF => self.op_9xy0(),
            0xA000..=0xAFFF => self.op_annn(),
            0xB000..=0xBFFF => self.op_bnnn(),
            0xC000..=0xCFFF => self.op_cxkk(),
            0xD000..=0xDFFF => self.op_dxyn(),
            0xE000..=0xEFFF => {
                match self.opcode & 0x00FF {
                    0x9E => self.op_ex9e(),
                    0xA1 => self.op_exa1(),
                    _ => (),
                }
            },
            0xF000..=0xFFFF => {
                match self.opcode & 0x00FF {
                    0x07 => self.op_fx07(),
                    0x0A => self.op_fx0a(),
                    0x15 => self.op_fx15(),
                    0x18 => self.op_fx18(),
                    0x1E => self.op_fx1e(),
                    0x29 => self.op_fx29(),
                    0x33 => self.op_fx33(),
                    0x55 => self.op_fx55(),
                    0x66 => self.op_fx66(),
                    _ => (),
                }
            },
            _ => println!("Unknown opcode: {:x}", self.opcode)
        }
    }

    fn cycle(&mut self) {
        self.opcode = (self.memory[self.pc as usize] as u16) << 8 | (self.memory[(self.pc + 1) as usize] as u16);
        self.pc += 2;
        self.run_opcode();

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("BEEP");
            }
            self.sound_timer -= 1;
        }
    }

}


fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("Chip8 Emulator", 640, 320)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();


    let mut chip8 = Chip8::init();
    chip8.load_rom("src/Chip8LogoTest.ch8");
    
    println!("opcode: {:x}", chip8.opcode);

    for i in 0..chip8.display.len() {
        if chip8.display[i] != 0 {
            println!("display: {}", chip8.display[i]);

        }
    }

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        canvas.clear();
        for (i, pixel) in chip8.display.iter().enumerate() {
            let x = (i as i32 % 64) * SCALE;
            let y = (i as i32 / 64)* 5;
            let color = if *pixel != 0 {
                //println!("x: {}, y: {}", x, y);
                sdl2::pixels::Color::RGB(255, 255, 255)
            } else {
                sdl2::pixels::Color::RGB(0, 0, 0)
            };
            let rect = Rect::new(x, y, SCALE as u32, SCALE as u32);
            canvas.set_draw_color(color);
            canvas.fill_rect(rect).unwrap();
        }
        canvas.present();
        println!("opcode: {:x}", chip8.opcode);
        println!("pc: {:x}", chip8.pc);
        
        chip8.cycle();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
