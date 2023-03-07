struct Mov {
    src: Reg,
    dst: Reg,
}

impl Mov {
    fn asm(&self) -> String {
        format!("mov {}, {}",
                self.dst.asm().to_lowercase(), self.src.asm().to_lowercase())
    }
}

#[allow(unused)]
struct Reg {
    region: Region,
    name: &'static str,
    mnemonic: &'static str,
}

impl Reg {
    fn new(mnemonic: &'static str, name: &'static str, region: Region) -> Self {
        Self { mnemonic, name, region }
    }

    fn asm(&self) -> &str {
        self.mnemonic
    }
}


// this also works for the R/M field, if MOD = 0b11
// (register to register copy)
fn parse_reg_field(reg: u8, w: bool) -> Reg {
    use Region::*;
    match (reg, w) {
        (0b000, false) => Reg::new("AL", "A", Low),
        (0b000, true) => Reg::new("AX", "A", Xtended),

        (0b001, false) => Reg::new("CL", "C", Low),
        (0b001, true) => Reg::new("CX", "C", Xtended),

        (0b010, false) => Reg::new("DL", "D", Low),
        (0b010, true) => Reg::new("DX", "D", Xtended),

        (0b011, false) => Reg::new("BL", "B", Low),
        (0b011, true) => Reg::new("BX", "B", Xtended),

        (0b100, false) => Reg::new("AH", "A", High),
        (0b100, true) => Reg::new("SP", "SP", Xtended),

        (0b101, false) => Reg::new("CH", "C", High),
        (0b101, true) => Reg::new("BP", "BP", Xtended),

        (0b110, false) => Reg::new("DH", "D", High),
        (0b110, true) => Reg::new("SI", "SI", Xtended),

        (0b111, false) => Reg::new("BH", "B", High),
        (0b111, true) => Reg::new("DI", "DI", Xtended),

        _ => panic!("unexpected reg pattern"),
    }
}

fn parse_mov_100_010_xx(bs: [u8; 2]) -> Mov {
    // bit 0    bit 1
    // 100010DW MOD|REG|R/M
    //           2   3   3

    // is_wide
    let w = bs[0] & 0b_0000_0001 != 0;
    // can ignore this for now, we're only handling reg-reg
    let _mod_bits = (bs[1] & 0b_1100_0000) >> 6;
    let reg_bits = (bs[1] & 0b_0011_1000) >> 3;
    let r_m_bits = bs[1] & 0b_0000_0111;

    let d_bit = bs[0] & 0b00000010 != 0;
    let reg_register = parse_reg_field(reg_bits, w);
    let r_m_register = parse_reg_field(r_m_bits, w);

    if d_bit {
        Mov { dst: reg_register, src: r_m_register }
    } else {
        Mov { src: reg_register, dst: r_m_register }
    }
}


#[allow(unused)]
enum Region {
    // 16 bits
    Xtended,
    // 8 bits
    Low,
    High,
}

fn main() {
    let mut args = std::env::args();
    args.next().unwrap();
    let filename = args.next().unwrap_or_else(
        || panic!("Must supply filename"));
    let bytes = std::fs::read(filename).unwrap();

    println!("bits 16");
    let mut i = 0;
    while i < bytes.len() {
        let first_byte = bytes[i].to_le();
        if first_byte & 0b11111000 == 0b10001000 {
            let asm = parse_mov_100_010_xx([bytes[i], bytes[i+1]]).asm();
            println!("{}", asm);
            i += 2;
        } else {
            panic!("0{:b}", bytes[i]);
        }
    }
}
