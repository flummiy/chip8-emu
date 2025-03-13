use chip8_emu::Chip8;

fn main() {
    let mut emu = Chip8::new();

    emu.run("roms/Pong.ch8", 10);
}
