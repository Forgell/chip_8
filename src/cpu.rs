use std::fs::File;
use std::fs;
use std::io::Read;
//use std::thread;
use std::time::Duration;

pub struct CPU{
    pub memory : Memory,
    pub registers : Registers,
    pub stack : Stack,
    pub timers : Timers,
    pub display : Display,
    pub pc : u16
}

pub struct Memory{
    pub bytes : [u8 ; 4096]
}

pub struct Registers{
    pub v : [u8 ; 16],
    pub i : u16
}

pub struct Stack{
    pub bytes : [u16 ; 12],
    pub pop_index : u8,
    //push_index : u8
}

pub struct Display{
    pub v_memory : [u8 ; 64*32]
}

impl Stack {
    pub fn pop(&mut self) -> u16{
        if self.pop_index == 0{
            panic!("there are no elements!!");
        }

        let value = self.bytes[self.pop_index as usize - 1];
        // update pop_index
        self.bytes[self.pop_index as usize- 1] = 0;
        self.pop_index -= 1;
        return value;
    }

    pub fn push(&mut self, element : u16){
        self.bytes[self.pop_index as usize] = element;
        self.pop_index += 1;
    }
}

pub struct Timers{
    pub delay_timer: u8,
    pub sound_timer: u8,
    time_since_update : Duration
}

impl Timers{
    pub fn update(&mut self , duration : Duration){
        //thread::
        if duration + self.time_since_update > Duration::from_millis(10){
            if self.delay_timer > 0{
                self.delay_timer -= 1;
            }
            if self.sound_timer > 0{
                self.sound_timer -= 1;
            }else{
                //todo play sound
            }
            self.time_since_update = duration + self.time_since_update - Duration::from_millis(10);
        }else{
            self.time_since_update += duration;
        }

    }
}

pub fn init() -> CPU{
    let mut cpu = CPU{
        memory: Memory { bytes: [0 ; 4096] },
        registers: Registers { v: [0 ; 16], i: 0 },
        stack: Stack { bytes: [0 ; 12], pop_index: 0 },
        timers: Timers { delay_timer: 0, sound_timer: 0, time_since_update: Duration::from_millis(0) },
        display : Display{v_memory : [0 ; 64*32]},
        pc : 0
    };
    //
    cpu.add_font_to_memory();

    return cpu;
}

impl CPU{
    fn add_font_to_memory(&mut self){
        let font = [0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
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
            0xF0, 0x80, 0xF0, 0x80, 0x80 ]; // F

        for i in 0..font.len(){
            self.memory.bytes[i] = font[i];
        }
    }

    pub fn load_rom(&mut self , file_name : String ){
        let raw_bytes = get_file_as_byte_vec(&file_name);
        //let p_length = raw_bytes.len()/2;
        // let mut program = [ 0 as u16; 4096];
        // let mut counter = 0;
        // for byte in raw_bytes{
        //     if counter % 2 == 0{
        //         program[counter/2] = program[counter/2] | ((byte as u16) << 8);
        //     }else{
        //         program[counter/2] = program[counter/2] | byte as u16;
        //     }
        //     counter += 1;
        // }

        let mut current_memory_index = 0x200 as usize;
        for byte in raw_bytes{
            self.memory.bytes[current_memory_index] = byte;
            current_memory_index += 1;
        }

        self.pc = 0x200;

    }


}

fn get_file_as_byte_vec(filename: &String) -> Vec<u8> {
    let mut f = File::open(&filename).expect("no file found");
    let metadata = fs::metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    buffer
}

