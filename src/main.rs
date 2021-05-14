mod cpu;
use cpu::*;
use console_engine::{pixel, ConsoleEngine};
//use console_engine::Color;
use console_engine::KeyCode;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use rand::Rng;
// use std::thread;
// use std::time::Duration;


const DISPLAY_RESOLUTION: [u32 ; 2] = [64 , 32];
fn main() {

    //const MIL_SEC: f32 = 16.7;
    let mut engine = console_engine::ConsoleEngine::init(64 + 2 , 32 + 2, 60);
    let mut keyboard_input = [false ; 16];
    let system_start =  SystemTime::now();

    //let since_start = system_start.duration_since(UNIX_EPOCH);


    let mut cpu = cpu::init();

    let file_name = String::from("roms/pong.rom");
    // load program
    cpu.load_rom(file_name);
    let time = system_start.duration_since(UNIX_EPOCH);
    let mut total_time : Duration = Duration::from_millis(0);
    if let Ok(time) = time{
        total_time = time;
    }
    loop{
        engine.wait_frame(); // wait for next frame + capture inputs
        engine.clear_screen(); // reset the screen
        // draw border
        draw_border(&mut engine , DISPLAY_RESOLUTION, &keyboard_input );
        // draw screen

        let index = cpu.pc;
        let opcode = ((cpu.memory.bytes[index as usize] as u16) << 8) | (cpu.memory.bytes[index as usize + 1] as u16) ;

        accept_opcode(&mut cpu, opcode, keyboard_input);

        draw_the_screen(&mut engine ,&mut cpu);



        let elapsed_time = system_start.duration_since(UNIX_EPOCH);
        if let Ok(elapsed_time) = elapsed_time{
            cpu.timers.update(elapsed_time - total_time);
            total_time = elapsed_time;
        }


        update_inputs(&mut keyboard_input ,&engine);
        if engine.is_key_pressed(KeyCode::Char('q')) { // if the user presses 'q' :
            break; // exits app
        }


        engine.draw(); // draw the screen
    }
}


fn draw_border(engine: &mut ConsoleEngine, display_res : [u32 ; 2], keyboard_input : &[bool ; 16]){
    let x_max = display_res[0] as i32 + 1;
    let y_max = display_res[1] as i32 + 1;

    engine.line(0 , 0 , x_max, 0 , pixel::pxl('▒') );
    engine.line(0 , 0 , 0, y_max, pixel::pxl('▒') );
    engine.line(x_max , 0 , x_max , y_max, pixel::pxl('▒'));
    //engine.line(0 , y_max, x_max , y_max , pixel::pxl('▒'));

    // the last row will also be a representatoin of the input
    for i in 0..keyboard_input.len(){
        if keyboard_input[i]{
            engine.set_pxl(i as i32 + 1, y_max, pixel::pxl('▒'));
        }
    }


}

fn set_pixel(engine:&mut ConsoleEngine, x: i32 , y : i32){
    let pixel_solid : char = '█';
    engine.set_pxl(x + 1, y + 1, pixel::pxl(pixel_solid)); // this is to offset for the border
}


fn accept_opcode(cpu :&mut CPU, opcode: u16, keyboard_input : [bool ; 16]){
    let first_four_bits = (opcode & 0xF000)  >> 12;
    match first_four_bits {
        0x0 => {
            // there are 3 possible opcodes
            let secound_4_bits = opcode & 0x0F00;

            //0NNN Calls machine code routine (RCA 1802 for COSMAC VIP) at address NNN. Not necessary for most ROMs.
            if secound_4_bits != 0{
                panic!("this is not a file we can run {:x}" , opcode)
            }
            //00E0 Clears the screen.

            else if secound_4_bits == 0{
                let last_four_bits = opcode & 0x000F;
                if last_four_bits == 0{
                    clear_display(cpu);
                    cpu.pc += 2;
                }else//00EE Returns from a subroutine.
                {
                    cpu.pc = cpu.stack.pop();
                    assert!(cpu.pc != 0, "opcode : {}" , opcode);
                    assert!(cpu.pc > 0x200, "opcode : {}" , opcode);
                    //println!("{} next opcode" , cpu.memory.bytes[cpu.pc as usize]);
                    //panic!("subroutine at 00EE : {}" , opcode)
                }

            }

        }
        0x1 => { // only 1NNN 	Jumps to address NNN.
            cpu.pc = 0x0FFF & opcode;
        }
        0x2 => { //2NNN	Calls subroutine at NNN.
            cpu.stack.push(cpu.pc + 2); // push the next address
            cpu.pc = 0x0FFF & opcode;
            assert!(cpu.pc != 0, "opcode : {}" , opcode);
        }
        0x3 => { // 3XNN Skips the next instruction if VX equals NN. (Usually the next instruction is a jump to skip a code block)
            let x = ((0x0F00 & opcode) >> 8) as usize;
            let nn = 0x00FF & opcode;
            if cpu.registers.v[x] == nn as u8{
                // then skip the next opcode
                cpu.pc += 4;
            }else{ // then don't
                cpu.pc += 2;
            }
        }
        0x4 => { // 4XNN Skips the next instruction if VX does not equal NN. (Usually the next instruction is a jump to skip a code block)
            let x = ((0x0F00 & opcode) >> 8) as usize;
            let nn = 0x00FF & opcode;
            if cpu.registers.v[x] != nn as u8{
                // then skip the next opcode
                cpu.pc += 4;
            }else{
                cpu.pc += 2;
            }

        }
        0x5 => { // 5XY0 Skips the next instruction if VX equals VY. (Usually the next instruction is a jump to skip a code block)
            let x = ((0x0F00 & opcode) >> 8) as usize;
            let y =  ((0x00F0 & opcode) >> 4) as usize;
            if cpu.registers.v[x] == cpu.registers.v[y]{
                // skip next opcode
                cpu.pc += 4;
            }else {
                cpu.pc += 2;
            }
        }
        0x6 => { // 6XNN Sets VX to NN.
            let x = ((0x0F00 & opcode) >> 8) as usize;
            let nn = 0x00FF & opcode;
            cpu.registers.v[x] = nn as u8;
            cpu.pc += 2;
        }
        0x7 => {//7XNN Adds NN to VX. (Carry flag is not changed)
            let x = ((0x0F00 & opcode) >> 8) as usize;
            let nn = 0x00FF & opcode;
            cpu.registers.v[x] = ((nn as u16) + cpu.registers.v[x] as u16) as u8;
            cpu.pc += 2;
        }
        0x8 => {
            let last_four_bits = 0x000F & opcode;
            let x  = ((0x0F00 & opcode) >> 8) as usize;
            let y =  ((0x00F0 & opcode) >> 4) as usize;
            match last_four_bits{

                0x0 => { // 8XY0 Sets VX to the value of VY.
                    cpu.registers.v[x] = cpu.registers.v[y];
                    cpu.pc += 2;
                }
                0x1 => { // 8XY1 	Sets VX to VX or VY. (Bitwise OR operation)
                    cpu.registers.v[x] = cpu.registers.v[x] | cpu.registers.v[y];
                    cpu.pc += 2;
                }
                0x2 => { // 8XY2 Sets VX to VX and VY. (Bitwise AND operation)
                    cpu.registers.v[x] = cpu.registers.v[x] & cpu.registers.v[y];
                    cpu.pc += 2;
                }
                0x3 => { // 8XY3  Sets VX to VX xor VY.
                    cpu.registers.v[x] = cpu.registers.v[x] ^ cpu.registers.v[y];
                    cpu.pc += 2;
                }
                0x4 => { // 8XY4 Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there is not.
                    // check if carry will happen
                    if cpu.registers.v[x] as u32 + cpu.registers.v[y] as u32> 0xFF  { // if sum is more than 255 carry will occur
                        cpu.registers.v[0xF as usize] = 1;
                    }else {
                        cpu.registers.v[0xF as usize] = 0;
                    }
                    cpu.registers.v[x] = (cpu.registers.v[y] as u16 + cpu.registers.v[x] as u16) as u8;
                    cpu.pc += 2;
                }
                0x5 => { // 8XY5 VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there is not.
                    // check if borrow will happen
                    if cpu.registers.v[x] < cpu.registers.v[y]{ // borrow will occur
                        cpu.registers.v[0xF as usize] = 0;
                        cpu.registers.v[x] = ((cpu.registers.v[x] as u16 + 0x100) - cpu.registers.v[y] as u16) as u8;
                    }else{
                        cpu.registers.v[0xF as usize] = 1;
                        cpu.registers.v[x] = (cpu.registers.v[x] as u16 - cpu.registers.v[y] as u16) as u8;
                    }


                    cpu.pc += 2;
                }
                0x6 => { //8XY6 Stores the least significant bit of VX in VF and then shifts VX to the right by 1.
                    let least_sing_bit = 0x01 & cpu.registers.v[x];
                    cpu.registers.v[0xF as usize] = least_sing_bit as u8;
                    cpu.registers.v[x] = cpu.registers.v[x] >> 1;
                    cpu.pc += 2;
                }
                0x7 => { // 8XY7 Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there is not.
                    // check if borrow will happen
                    if cpu.registers.v[x] > cpu.registers.v[y]{ // borrow will occur
                        cpu.registers.v[0xF as usize] = 0;
                    }else{
                        cpu.registers.v[0xF as usize] = 1;
                    }
                    cpu.registers.v[x] = cpu.registers.v[y] - cpu.registers.v[x];
                    cpu.pc += 2;
                }
                0xE => { // 8XYE Stores the most significant bit of VX in VF and then shifts VX to the left by 1.[b]
                    let most_sing_bit = (0x80 & cpu.registers.v[x]) >> 7;
                    cpu.registers.v[0xF as usize] = most_sing_bit as u8;
                    cpu.registers.v[x] = cpu.registers.v[x] << 1;
                    cpu.pc += 2;

                }
                _ => {panic!("opcode not recognized {}", opcode);}
            }

        }
        0x9 => { // 9XY0 Skips the next instruction if VX does not equal VY. (Usually the next instruction is a jump to skip a code block)
            let x  = ((0x0F00 & opcode) >> 8) as usize;
            let y =  ((0x00F0 & opcode) >> 4) as usize;
            if cpu.registers.v[x] != cpu.registers.v[y]{
                cpu.pc += 4;
            }else{
                cpu.pc += 2;
            }

        }
        0xA => { // ANNN Sets I to the address NNN.
            let nnn = 0x0FFF & opcode;
            cpu.registers.i = nnn;
            cpu.pc += 2;
        }
        0xB => { // BNNN Jumps to the address NNN plus V0.
            let nnn = 0x0FFF & opcode;
            cpu.pc = nnn + cpu.registers.v[0] as u16;
            assert!(cpu.pc != 0, "opcode : {}" , opcode);
        }
        0xC => { //CXNN Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN.
            let x  = ((0x0F00 & opcode) >> 8) as usize;
            let nn = (0x00FF & opcode) as u8;
            let random : u8 = rand::thread_rng().gen_range(0..255);
            cpu.registers.v[x] = random & nn;
            cpu.pc += 2;

        }
        0xD => { //  DXYN
            // 	Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N+1 pixels.
            // Each row of 8 pixels is read as bit-coded starting from memory location I; I value does not change after the execution of this instruction.
            // As described above, VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn, and to 0 if that does not happen
            let x  = ((0x0F00 & opcode) >> 8) as usize;
            let y =  ((0x00F0 & opcode) >> 4) as usize;
            let n = 0x000F & opcode;
            let vx = cpu.registers.v[x];
            let vy = cpu.registers.v[y];
            let mut flag = true;
            for j in 0..n{
                let line_from_memory = cpu.memory.bytes[cpu.registers.i as usize + j as usize];
                for i in 0..8{
                    let x = (vx as u16 + i) as usize; //% display.len();
                    let y = (vy as u16 + j) as usize; //% display[x].len();


                    let bit = (line_from_memory & (0x80 >> i)) >> (7 - i);
                    let diapay_value = cpu.display.v_memory[(x + (y * 64)) % 2048];
                    if bit == 1{
                        if diapay_value == 1{
                            flag = false;
                        }
                        cpu.display.v_memory[(x + (y * 64)) % 2048] ^= 1;
                    }

                    // if bit == 1 {
                    //     display[x][y] = !display[x][y];
                    //     if !display[x][y] { // was set to false
                    //         flag = false;
                    //     }
                    // }
                    //println!("{}" , bit);
                }
                //println!("{:b}" , line_from_memory);
            }
            if flag{
                cpu.registers.v[0xF as usize] = 0;
            }else {
                cpu.registers.v[0xF as usize] = 1;
            }
            cpu.pc += 2;
        }
        0xE => {
            let last_8_bits = opcode & 0x00FF;
            let x = (opcode & 0x0F00) >> 8;
            match last_8_bits{
                0x9E => { // EX9E Skips the next instruction if the key stored in VX is pressed. (Usually the next instruction is a jump to skip a code block)
                    if keyboard_input[cpu.registers.v[x as usize] as usize]{
                        cpu.pc += 4;
                    }else{
                        cpu.pc += 2;
                    }
                }
                0xA1 => { //  EXA1 Skips the next instruction if the key stored in VX is not pressed. (Usually the next instruction is a jump to skip a code block)
                    if !keyboard_input[cpu.registers.v[x as usize] as usize]{
                        cpu.pc += 4;
                    }else{
                        cpu.pc += 2;
                    }
                }
                _ => {panic!("opcode not recognized: {:x}" , opcode)}
            }
        }
        0xF => {
            let x = (opcode & 0x0F00) >> 8;
            let last_8_bits = opcode & 0x00FF;
            match last_8_bits{
                0x07 => { // FX07 Sets VX to the value of the delay timer.
                    cpu.registers.v[x as usize] = cpu.timers.delay_timer;
                    cpu.pc += 2;
                }
                0x0A => { // FX0A A key press is awaited, and then stored in VX. (Blocking Operation. All instruction halted until next key event)
                    let mut flag = true;
                    for i in 0..keyboard_input.len(){
                        if keyboard_input[i]{
                            cpu.registers.v[x as usize] = i as u8;
                            flag = false;
                        }
                    }
                    if flag{ // there is no key press we need  to redo this opcode until there is one
                        //cpu.pc -= 1;
                        //panic!("oh no")
                    }else{
                        cpu.pc += 2;
                    }
                }
                0x15 => { // FX15	Sets the delay timer to VX.
                    cpu.timers.delay_timer = cpu.registers.v[x as usize];
                    cpu.pc += 2;
                }
                0x18 => { // FX18 Sets the sound timer to VX.
                    cpu.timers.sound_timer = cpu.registers.v[x as usize];
                    cpu.pc += 2;
                }
                0x1E => { // FX1E Adds VX to I. VF is not affected
                    cpu.registers.i += cpu.registers.v[x as usize] as u16;
                    cpu.pc += 2;
                }
                0x29 => { // FX29 Sets I to the location of the sprite for the character in VX. Characters 0-F (in hexadecimal) are represented by a 4x5 font.
                    cpu.registers.i = cpu.registers.v[x as usize] as u16 * 5;
                    cpu.pc += 2;
                }
                0x33 => { // FX33
                    //Stores the binary-coded decimal representation of VX,
                    // with the most significant of three digits at the address in I,
                    // the middle digit at I plus 1, and the least significant digit at I plus 2.
                    // (In other words, take the decimal representation of VX, place the hundreds digit in memory at location in I,
                    // the tens digit at location I+1, and the ones digit at location I+2.)

                    cpu.memory.bytes[cpu.registers.i as usize] = cpu.registers.v[x as usize] / 100;
                    cpu.memory.bytes[cpu.registers.i as usize + 1] = (cpu.registers.v[x as usize] / 10 ) % 10;
                    cpu.memory.bytes[cpu.registers.i as usize + 2] = (cpu.registers.v[x as usize] % 100)  % 10;
                    cpu.pc += 2;

                }
                0x55 => { // FX33 Stores V0 to VX (including VX) in memory starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified
                    for i in 0..(x+1){
                        cpu.memory.bytes[(cpu.registers.i  + i) as usize] = cpu.registers.v[i as usize];
                    }
                    cpu.pc += 2;
                }
                0x65 => { // FX65 Fills V0 to VX (including VX) with values from memory starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified
                    for i in 0..(x+1){
                        cpu.registers.v[i as usize] = cpu.memory.bytes[(cpu.registers.i + i) as usize];
                    }
                    cpu.pc += 2;
                }
                _ => {panic!("opcode not recognized: {:x}, {}" , opcode , cpu.pc)}
            }

        }

        _ => {panic!("opcode not recognized {:x}", opcode);}
    }
}

fn clear_display(cpu : &mut CPU){
    for i in 0..cpu.display.v_memory.len(){
        cpu.display.v_memory[i] = 0;
    }
}


fn draw_the_screen(engine :&mut ConsoleEngine, cpu : &mut CPU){
    for i in 0..cpu.display.v_memory.len(){
        let y = i / 64; // diplay width is 64 the length of each simulated part of the matrix
        let x = i % 64; //
        if cpu.display.v_memory[i] == 1{
            set_pixel( engine, x as i32, y as i32);
        }

    }
}


fn update_inputs(keyborad_inputs : &mut [bool ; 16] , engine: &ConsoleEngine){

    keyborad_inputs[0]  = true;//engine.is_key_pressed(KeyCode::Char('0'));

    keyborad_inputs[1]  = true;//engine.is_key_held(KeyCode::Char('1'));
    keyborad_inputs[2]  = true;//engine.is_key_held(KeyCode::Char('2'));
    keyborad_inputs[3]  = true;//engine.is_key_held(KeyCode::Char('3'));
    keyborad_inputs[4]  = true;//engine.is_key_held(KeyCode::Char('4'));
    keyborad_inputs[5]  = true;//engine.is_key_held(KeyCode::Char('5'));
    keyborad_inputs[6]  = true;//engine.is_key_held(KeyCode::Char('6'));
    keyborad_inputs[7]  = true;//engine.is_key_held(KeyCode::Char('7'));
    keyborad_inputs[8]  = true;//engine.is_key_held(KeyCode::Char('8'));
    keyborad_inputs[9]  = true;//engine.is_key_held(KeyCode::Char('9'));
    keyborad_inputs[10] = true;//engine.is_key_held(KeyCode::Char('z'));
    keyborad_inputs[11] = true;//engine.is_key_held(KeyCode::Char('x'));
    keyborad_inputs[12] = true;//engine.is_key_held(KeyCode::Char('c'));
    keyborad_inputs[13] = true;//engine.is_key_held(KeyCode::Char('v'));
    keyborad_inputs[14] = true;//engine.is_key_held(KeyCode::Char('b'));
    keyborad_inputs[15] = engine.is_key_held(KeyCode::Char('n'));
}