struct Mov {
    src: Loc,
    dst: Loc,
}

enum Loc {
    Reg(Reg),
    EAC(EAC),
    Imm(u16), // this is only applicable when Loc is a src
}

// Effective Address Calculation
struct EAC {
    base: EABase,
    displacement: Option<u16>, // can be either 0, 8, or 16 bits
}

impl EAC {
    fn new(base: EABase, displacement: Option<u16>) -> Self {
        Self { base, displacement }
    }

    fn asm(&self) -> String {
        if let Some(0) | None = self.displacement {
            format!("[{}]", self.base.asm())
        } else {
            format!("[{} + {}]", self.base.asm(), self.displacement.unwrap())
        }
    }
}

enum EABase {
    BxSi,
    BxDi,
    BpSi,
    BpDi,
    Si,
    Di,
    // TODO: direct address
    Bx,
    Bp,
}

impl EABase {
    fn asm(&self) -> &'static str {
        match self {
            Self::BxSi => "bx + si",
            Self::BxDi => "bx + di",
            Self::BpSi => "bp + si",
            Self::BpDi => "bp + di",
            Self::Si => "si",
            Self::Di => "di",
            Self::Bx => "bx",
            Self::Bp => "bp",
        }
    }
}

impl Mov {
    fn asm(&self) -> String {
        format!("mov {}, {}",
                self.dst.asm().to_lowercase(),
                self.src.asm().to_lowercase())
    }
}

impl Loc {
    fn asm(&self) -> String {
        match self {
            Self::Reg(reg) => reg.asm().to_string(),
            Self::Imm(n) => n.to_string(),
            Self::EAC(eac) => eac.asm(),
        }
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

fn parse_r_m_field(r_m_bits: u8, displacement: Option<u16>) -> EAC {
    use EABase::*;
    match r_m_bits {
        0b000 => EAC::new(BxSi, displacement),
        0b001 => EAC::new(BxDi, displacement),
        0b010 => EAC::new(BpSi, displacement),
        0b011 => EAC::new(BpDi, displacement),
        0b100 => EAC::new(Si, displacement),
        0b101 => EAC::new(Di, displacement),
        0b110 if displacement.is_none() => panic!("not handling Direct Address yet"),
        0b110 if displacement.is_some() => EAC::new(Bp, displacement),
        0b111 => EAC::new(Bx, displacement),
        _ => panic!("unexpected bit pattern: 0b_{:b}", r_m_bits),
    }
}

fn parse_mov_100_010_xx(bs: &mut impl Iterator<Item = u8>) -> Mov {
    let b0 = bs.next().unwrap();
    let b1 = bs.next().unwrap();
    
    // bit 0    bit 1
    // 100010DW MOD|REG|R/M
    //           2   3   3
    let w = b0 & 0b_0000_0001 != 0; // is_wide
    let mod_bits = (b1 & 0b_1100_0000) >> 6;
    let reg_bits = (b1 & 0b_0011_1000) >> 3;
    let r_m_bits = b1 & 0b_0000_0111;

    let d_bit = b0 & 0b00000010 != 0;
    let reg_register = parse_reg_field(reg_bits, w);
    let r_m_loc = match mod_bits {
        0b11 => Loc::Reg(parse_reg_field(r_m_bits, w)),
        0b00 => Loc::EAC(parse_r_m_field(r_m_bits, None)),
        0b01 => {
            let displacement = bs.next().unwrap() as u16;
            Loc::EAC(parse_r_m_field(r_m_bits, Some(displacement)))
        },
        0b10 => {
            let displacement = consume_u16(bs);
            Loc::EAC(parse_r_m_field(r_m_bits, Some(displacement)))
        },
        _ => panic!("unexpected MOD field: 0b_{:b}", mod_bits),
    };

    if d_bit {
        Mov { dst: Loc::Reg(reg_register), src: r_m_loc }
    } else {
        Mov { src: Loc::Reg(reg_register), dst: r_m_loc }
    }
}

fn consume_u16(bs: &mut impl Iterator<Item = u8>) -> u16 {
    u16::from_le_bytes([bs.next().unwrap(), bs.next().unwrap()])
}


fn parse_mov_1011_xxxx(bs: &mut impl Iterator<Item = u8>) -> Mov {
    let b0 = bs.next().unwrap();
    // bit 0    
    // 1011|W|REG
    //      1  3    
    let w = (b0 & 0b_0000_1000) != 0;
    let reg = b0 & 0b_0000_0111;
    let dst = parse_reg_field(reg, w);
    let src = Loc::Imm(if w {
        // if w bit is set, then read 16 bit imm value from next 2 bytes
        consume_u16(bs)
    } else {
        // otherwise read 8 bit imm value from the next byte
        bs.next().unwrap() as u16
    });
    Mov { src, dst: Loc::Reg(dst) }
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
    let mut bytes = std::fs::read(filename).unwrap().into_iter().peekable();

    println!("bits 16");
    while let Some(byte) = bytes.peek() {
        if byte >> 2 == 0b_10_0010 {
            let asm = parse_mov_100_010_xx(&mut bytes).asm();
            println!("{}", asm);
        } else if byte >> 4 == 0b_1011  {
            let asm = parse_mov_1011_xxxx(&mut bytes).asm();
            println!("{}", asm);
        } else {
            panic!("0b{:b}", byte);
        }
    }
}
