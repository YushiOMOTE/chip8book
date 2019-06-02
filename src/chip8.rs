use log::*;
use rand::{rngs::JitterRng, Rng};
use uefi::{
    prelude::*,
    proto::console::{
        gop::{BltOp, BltPixel, GraphicsOutput},
        text::{Key, ScanCode},
    },
};

struct KeyInfo {
    key: char,
    time: u64,
}

struct Hardware<'a, 'b, 'c> {
    st: &'a SystemTable<Boot>,
    gop: &'b mut GraphicsOutput<'c>,
    vramsz: (usize, usize),
    vram: [bool; 64 * 32],
    pressed: Option<KeyInfo>,
    rng: JitterRng,
}

fn tsc() -> u64 {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::_rdtsc;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::_rdtsc;

    unsafe { _rdtsc() as u64 }
}

impl<'a, 'b, 'c> Hardware<'a, 'b, 'c> {
    fn new(st: &'a SystemTable<Boot>, gop: &'b mut GraphicsOutput<'c>) -> Self {
        Self {
            st,
            gop,
            vramsz: (0, 0),
            vram: [false; 64 * 32],
            pressed: None,
            rng: JitterRng::new_with_timer(tsc),
        }
    }

    fn set_pixel(&mut self, x: usize, y: usize, d: bool) {
        let stride = self.gop.current_mode_info().stride();
        let pixel_index = (y * stride) + x;
        let pixel_base = 4 * pixel_index;

        if d {
            unsafe {
                self.gop
                    .frame_buffer()
                    .write_value(pixel_base, BltPixel::new(255, 255, 255));
            }
        } else {
            unsafe {
                self.gop
                    .frame_buffer()
                    .write_value(pixel_base, BltPixel::new(0, 0, 0));
            }
        }
    }

    fn set_pixel8(&mut self, x: usize, y: usize, d: bool) {
        let (cw, ch) = (64, 32);
        let (w, h) = self.gop.current_mode_info().resolution();
        let rx = (w / cw) / 2;
        let ry = (h / ch) / 2;

        let xs = (w - cw * rx) / 2;
        let ys = (h - ch * ry) / 2;

        let xb = x * rx;
        let yb = y * ry;
        for yo in 0..ry {
            for xo in 0..rx {
                self.set_pixel(xb + xo + xs, yb + yo + ys, d);
            }
        }
    }

    fn get_key(&mut self) -> Option<Key> {
        let comp = self.st.stdin().read_key().expect("Couldn't poll key input");
        comp.expect("Couldn't extract key result")
    }
}

impl<'a, 'b, 'c> libchip8::Hardware for Hardware<'a, 'b, 'c> {
    fn rand(&mut self) -> u8 {
        self.rng.gen()
    }

    fn key(&mut self, key: u8) -> bool {
        match self.pressed.as_ref() {
            Some(p) => {
                debug!("{} down detected", p.key);

                p.key
                    == match key {
                        0 => 'x',
                        1 => '1',
                        2 => '2',
                        3 => '3',
                        4 => 'q',
                        5 => 'w',
                        6 => 'e',
                        7 => 'a',
                        8 => 's',
                        9 => 'd',
                        0xa => 'z',
                        0xb => 'c',
                        0xc => '4',
                        0xd => 'e',
                        0xe => 'd',
                        0xf => 'c',
                        _ => return false,
                    }
            }
            None => return false,
        }
    }

    fn vram_set(&mut self, x: usize, y: usize, d: bool) {
        self.vram[y * 64 + x] = d;
        self.set_pixel8(x, y, d);
    }

    fn vram_get(&mut self, x: usize, y: usize) -> bool {
        self.vram[y * 64 + x]
    }

    fn vram_setsize(&mut self, size: (usize, usize)) {
        self.vramsz = size;
    }

    fn vram_size(&mut self) -> (usize, usize) {
        self.vramsz
    }

    fn clock(&mut self) -> u64 {
        if cfg!(features = "uefi_time_source") {
            let rt = self.st.runtime_services();
            let t = rt
                .get_time()
                .expect("Couldn't get time")
                .expect("Couln't extract time");

            let days = days_from_civil(t.year() as i64, t.month() as i64, t.day() as i64);

            (days as u64) * 24 * 3600_000_000_000
                + (t.hour() as u64) * 3600_000_000_000
                + (t.minute() as u64) * 60_000_000_000
                + (t.second() as u64) * 1000_000_000
                + t.nanosecond() as u64
        } else {
            tsc() / 2
        }
    }

    fn beep(&mut self) {}

    fn sched(&mut self) -> bool {
        match self.get_key() {
            Some(Key::Special(ScanCode::ESCAPE)) => return true,
            Some(Key::Printable(code)) => {
                self.pressed = Some(KeyInfo {
                    key: code.into(),
                    time: self.clock(),
                });
                debug!("pressed {}", self.pressed.as_ref().unwrap().key);
            }
            _ => {
                let clk = self.clock();

                if let Some(k) = self.pressed.as_ref() {
                    if clk.wrapping_sub(k.time) > 200_000_000 {
                        self.pressed = None;
                        debug!("released");
                    }
                }
            }
        }

        self.st.boot_services().stall(1000_000 / 600);

        false
    }
}

#[allow(unused)]
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400);
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

pub fn run(st: &SystemTable<Boot>) {
    info!("Running chip8");

    let gop = st
        .boot_services()
        .locate_protocol::<GraphicsOutput>()
        .expect("No graphics output protocol available");
    let gop = gop.expect("Error on opening graphics output protocol");
    let gop = unsafe { &mut *gop.get() };

    setup(gop);
    run_chip8(st, gop);
}

fn setup(gop: &mut GraphicsOutput) {
    let mode = gop
        .modes()
        .map(|mode| mode.expect("Couldn't get graphics mode"))
        .nth(0)
        .expect("No graphics mode");
    gop.set_mode(&mode)
        .expect_success("Couldn't set graphics mode");
}

fn clear(gop: &mut GraphicsOutput) {
    let op = BltOp::VideoFill {
        color: BltPixel::new(0, 0, 0),
        dest: (0, 0),
        dims: gop.current_mode_info().resolution(),
    };
    gop.blt(op)
        .expect_success("Failed to fill screen with color");
}

fn run_chip8(st: &SystemTable<Boot>, gop: &mut GraphicsOutput) {
    let hw = Hardware::new(st, gop);
    let chip8 = libchip8::Chip8::new(hw);
    chip8.run(include_bytes!("roms/invaders.ch8"));
    clear(gop);
}
