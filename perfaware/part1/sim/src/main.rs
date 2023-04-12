enum Instruction {
    Mov(Mov),
    Jump(Jump),
    Add(Add),
    Sub(Sub),
    Cmp(Cmp),
}

impl Instruction {
    fn asm(&self) -> String {
        match self {
            Self::Mov(m) => m.asm(),
            Self::Jump(j) => j.asm(),
            Self::Add(a) => a.asm(),
            Self::Sub(s) => s.asm(),
            Self::Cmp(c) => c.asm(),
        }
    }
}

#[derive(Clone, Copy)]
#[repr(u8)]
enum Reg {
    A = 0,
    B,
    C,
    D,
    DI,
    SI,
    SP,
    BP,
}

struct CPU {
    registers: [u32; 8], // indexed by `Reg as usize`
}

impl CPU {
    fn new() -> Self {
        Self { registers: [0; 8] }
    }

    fn exec(&mut self, inst: Instruction) {
        match inst {
            Instruction::Mov(mov) => {
                let src = self.get_src(mov.src);
                self.set_dest(mov.dst, src);
            }
            Instruction::Jump(jump) => todo!(),
            Instruction::Add(add) => todo!(),
            Instruction::Sub(sub) => todo!(),
            Instruction::Cmp(cmp) => todo!(),
        }
    }

    fn get_src(&self, loc: Loc) -> u32 {
        match loc {
            Loc::Imm8(n) => n as _,
            Loc::Imm16(n) => n as _,
            Loc::Reg(_) => todo!(),
            Loc::EAC(_) => todo!(),
        }
    }

    fn set_dest(&mut self, loc: Loc, val: u32) {
        match loc {
            Loc::Reg(reg) => {
                todo!()
            },
            Loc::EAC(_) => todo!(),
            Loc::Imm8(_) | Loc::Imm16(_) => unreachable!(),
        }
    }
}

struct Jump {
    typ: JumpType,
    offset: i8,
}

impl Jump {
    fn asm(&self) -> String {
        let mnemonic = match self.typ {
            JumpType::Jnz => "jnz",
            JumpType::Je => "je",
            JumpType::Jl => "jl",
            JumpType::Jle => "jle",
            JumpType::Jb => "jb",
            JumpType::Jbe => "jbe",
            JumpType::Jp => "jp",
            JumpType::Jo => "jo",
            JumpType::Js => "js",
            JumpType::Jnl => "jnl",
            JumpType::Jg => "jg",
            JumpType::Jnb => "jnb",
            JumpType::Ja => "ja",
            JumpType::Jnp => "jnp",
            JumpType::Jno => "jno",
            JumpType::Jns => "jns",
            JumpType::Loop => "loop",
            JumpType::Loopz => "loopz",
            JumpType::Loopnz => "loopnz",
            JumpType::Jcxz => "jcxz",
        };
        // nasm is weird, and takes the offset for BEFORE the instruction
        // instead of after, so we have to mix in the instruction size
        let nasm_offset = Self::instruction_size() as i8 + self.offset;
        if nasm_offset >= 0 {
            format!("{mnemonic} $+{nasm_offset}")
        } else {
            format!("{mnemonic} ${nasm_offset}")
        }
    }

    // for now, they're all 2, see page 168 in the intel 8086 manual
    const fn instruction_size() -> usize {
        2
    }
}

#[repr(u8)]
#[derive(Copy, Clone)]
enum JumpType {
    Jnz = 0b_0111_0101, // also stands for Jne
    Je = 0b_0111_0100,
    Jl = 0b_0111_1100,
    Jle = 0b_0111_1110,
    Jb = 0b_0111_0010,
    Jbe = 0b_0111_0110,
    Jp = 0b_0111_1010,
    Jo = 0b_0111_0000,
    Js = 0b_0111_1000,
    Jnl = 0b_0111_1101,
    Jg = 0b_0111_1111,
    Jnb = 0b_0111_0011,
    Ja = 0b_0111_0111,
    Jnp = 0b_0111_1011,
    Jno = 0b_0111_0001,
    Jns = 0b_0111_1001,
    Loop = 0b_1110_0010,
    Loopz = 0b_1110_0001,
    Loopnz = 0b_1110_0000,
    Jcxz = 0b_1110_0011,
}

impl JumpType {
    const ALL : [Self; 20] = [
        Self::Jnz, Self::Je, Self::Jl, Self::Jle, Self::Jb,
        Self::Jbe, Self::Jp, Self::Jo, Self::Js, Self::Jnl,
        Self::Jg, Self::Jnb, Self::Ja, Self::Jnp, Self::Jno,
        Self::Jns, Self::Loop, Self::Loopz, Self::Loopnz,
        Self::Jcxz];

    fn find(inst: u8) -> Option<Self> {
        Self::ALL.iter().find(|b| **b as u8 == inst).copied()
    }
}

fn try_parse_jump(b: u8, bs: &mut impl Iterator<Item = u8>) -> Option<Jump> {
    let typ = JumpType::find(b)?;
    bs.next().unwrap(); // advance the iterator forward 1 to consume the
                        // first byte
    Some(Jump { typ, offset: consume_i8(bs) })
}

struct Mov {
    src: Loc,
    dst: Loc,
}

impl Mov {
    fn asm(&self) -> String {
        format!("mov {}, {}",
                self.dst.asm().to_lowercase(),
                self.src.asm().to_lowercase())
    }
}

struct Add {
    src: Loc,
    dst: Loc,
}

impl Add {
    fn asm(&self) -> String {
        format!("add {}, {}",
                self.dst.asm().to_lowercase(),
                self.src.asm().to_lowercase())
    }
}

struct Sub {
    src: Loc,
    dst: Loc,
}

impl Sub {
    fn asm(&self) -> String {
        format!("sub {}, {}",
                self.dst.asm().to_lowercase(),
                self.src.asm().to_lowercase())
    }
}

struct Cmp {
    src: Loc,
    dst: Loc,
}

impl Cmp {
    fn asm(&self) -> String {
        format!("cmp {}, {}",
                self.dst.asm().to_lowercase(),
                self.src.asm().to_lowercase())
    }
}

enum Loc {
    Reg(RegIndex),
    EAC(EAC),
    Imm8(u8), // this is only applicable when Loc is a src
    Imm16(u16), // this is only applicable when Loc is a src
}

impl Loc {
    fn asm(&self) -> String {
        match self {
            Self::Reg(reg) => reg.asm().to_string(),
            Self::Imm8(n) => format!("byte {}", n),
            Self::Imm16(n) => format!("word {}", n),
            Self::EAC(eac) => eac.asm(),
        }
    }
}


// Effective Address Calculation
struct EAC {
    base: EABase,
    displacement: Option<i16>, // can be either 0, 8, or 16 bits
}

impl EAC {
    fn new(base: EABase, displacement: Option<i16>) -> Self {
        Self { base, displacement }
    }

    fn asm(&self) -> String {
        match self.displacement {
            None => format!("[{}]", self.base.asm()),
            Some(d@0..) => format!("[{} + {}]", self.base.asm(), d),
            Some(d) => format!("[{} - {}]", self.base.asm(), -d),
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
    DirectAddr(u16),
    Bx,
    Bp,
}

impl EABase {
    fn asm(&self) -> String {
        match self {
            Self::BxSi => "bx + si".into(),
            Self::BxDi => "bx + di".into(),
            Self::BpSi => "bp + si".into(),
            Self::BpDi => "bp + di".into(),
            Self::Si => "si".into(),
            Self::Di => "di".into(),
            Self::Bx => "bx".into(),
            Self::Bp => "bp".into(),
            Self::DirectAddr(n) => n.to_string(),
        }
    }
}

struct RegIndex {
    #[allow(unused)]
    region: Region,
    register: Reg,
    mnemonic: &'static str, // only used for printing assembly
}

impl RegIndex {
    const fn new(mnemonic: &'static str, register: Reg, region: Region) -> Self {
        Self { mnemonic, register, region }
    }

    fn asm(&self) -> &str {
        self.mnemonic
    }

    fn acc(w: bool) -> Self {
        if w { Self::AX } else { Self::AL }
    }

    const AL : RegIndex = RegIndex::new("AL", Reg::A, Region::Low);
    const AX : RegIndex = RegIndex::new("AX", Reg::A, Region::Xtended);
}

// this also works for the R/M field, if MOD = 0b11
// (register to register copy)
fn parse_reg_field(reg: u8, w: bool) -> RegIndex {
    use Region::*;
    match (reg, w) {
        (0b000, _) => RegIndex::acc(w),

        (0b001, false) => RegIndex::new("CL", Reg::C, Low),
        (0b001, true) => RegIndex::new("CX", Reg::C, Xtended),

        (0b010, false) => RegIndex::new("DL", Reg::D, Low),
        (0b010, true) => RegIndex::new("DX", Reg::D, Xtended),

        (0b011, false) => RegIndex::new("BL", Reg::B, Low),
        (0b011, true) => RegIndex::new("BX", Reg::B, Xtended),

        (0b100, false) => RegIndex::new("AH", Reg::A, High),
        (0b100, true) => RegIndex::new("SP", Reg::SP, Xtended),

        (0b101, false) => RegIndex::new("CH", Reg::C, High),
        (0b101, true) => RegIndex::new("BP", Reg::BP, Xtended),

        (0b110, false) => RegIndex::new("DH", Reg::D, High),
        (0b110, true) => RegIndex::new("SI", Reg::SI, Xtended),

        (0b111, false) => RegIndex::new("BH", Reg::B, High),
        (0b111, true) => RegIndex::new("DI", Reg::DI, Xtended),

        _ => panic!("unexpected reg pattern"),
    }
}

fn parse_r_m_field(r_m_bits: u8, displacement: Option<i16>) -> EAC {
    use EABase::*;
    match r_m_bits {
        0b000 => EAC::new(BxSi, displacement),
        0b001 => EAC::new(BxDi, displacement),
        0b010 => EAC::new(BpSi, displacement),
        0b011 => EAC::new(BpDi, displacement),
        0b100 => EAC::new(Si, displacement),
        0b101 => EAC::new(Di, displacement),
        0b110 if displacement.is_none() => unreachable!("not handling Direct Address from this function, should have used parse_r_m_direct_addr"),
        0b110 if displacement.is_some() => EAC::new(Bp, displacement),
        0b111 => EAC::new(Bx, displacement),
        _ => panic!("unexpected bit pattern: 0b_{:b}", r_m_bits),
    }
}

fn parse_r_m_direct_addr(direct_addr: u16) -> EAC {
    use EABase::*;
    EAC::new(DirectAddr(direct_addr), None)
}

fn parse_mem_to_acc_mov(bs: &mut impl Iterator<Item = u8>) -> Mov {
    let b0 = bs.next().unwrap();
    let addr = consume_u16(bs);
    // byte 0  
    // 1010000W
    let w = b0 & 0b_0000_0001 != 0; // is_wide
    let dst = Loc::Reg(RegIndex::acc(w));
    let src = Loc::EAC(EAC::new(EABase::DirectAddr(addr), None));
    Mov { src, dst }
}

fn parse_acc_to_mem_mov(bs: &mut impl Iterator<Item = u8>) -> Mov {
    let b0 = bs.next().unwrap();
    let addr = consume_u16(bs);
    // byte 0  
    // 1010001W
    let w = b0 & 0b_0000_0001 != 0; // is_wide
    let src = Loc::Reg(RegIndex::acc(w));
    let dst = Loc::EAC(EAC::new(EABase::DirectAddr(addr), None));
    Mov { src, dst }
}

fn parse_r_m_to_r_m(b: u8, bs: &mut impl Iterator<Item = u8>) -> Option<Instruction> {
    // byte 0   byte 1
    // OPCODE|DW MOD|REG|R/M
    //   6       2   3   3
    let opcode = b >> 2;
    let is_mov = opcode == 0b_1000_10;

    // inside the 6 bits of OPCODE, if not a mov
    // 00|BINOP|0
    //      3
    let binop = (0b_11_000_1 & opcode == 0)
        .then(|| BinOpCode::find((opcode >> 1) & 0b111)).flatten();
    if !is_mov && binop.is_none() {
        return None
    }

    let b0 = bs.next().unwrap();
    let b1 = bs.next().unwrap();
    let w = b0 & 0b_0000_0001 != 0; // is_wide
    let mod_bits = (b1 & 0b_1100_0000) >> 6;
    let reg_bits = (b1 & 0b_0011_1000) >> 3;
    let r_m_bits = b1 & 0b_0000_0111;

    let d_bit = b0 & 0b00000010 != 0;
    let reg_register = parse_reg_field(reg_bits, w);
    let r_m_loc = parse_r_m_loc(bs, mod_bits, r_m_bits, w);
    let (src, dst) = if d_bit {
        (r_m_loc, Loc::Reg(reg_register))
    } else {
        (Loc::Reg(reg_register), r_m_loc)
    };
    let params = BinopParams::from(is_mov, binop);
    Some(binop_to_instruction(params, src, dst))
}

fn parse_imm_to_reg_mov(bs: &mut impl Iterator<Item = u8>) -> Mov {
    let b0 = bs.next().unwrap();
    // byte 0    
    // 1011|W|REG
    //      1  3    
    let w = (b0 & 0b_0000_1000) != 0;
    let reg = b0 & 0b_0000_0111;
    let dst = parse_reg_field(reg, w);
    let src = if w {
        Loc::Imm16(consume_u16(bs))
    } else {
        Loc::Imm8(bs.next().unwrap())
    };
    Mov { src, dst: Loc::Reg(dst) }
}

fn parse_imm_to_acc(b: u8, bs: &mut impl Iterator<Item = u8>) -> Option<Instruction> {
    // byte 0
    // 00BIN10W
    if b & 0b11_000_110 != 0b00_000_100 { // 00_xxx_10x
        return None;
    }

    let binop = BinOpCode::find((b >> 3) & 0b111);
    if binop.is_none() {
        return None;
    } 
    let binop = binop.unwrap();

    let b0 = bs.next().unwrap();
    let w = b0 & 0b_0000_0001 != 0; // is_wide
    let (src, dst) = if w {
        (Loc::Imm16(consume_u16(bs)), Loc::Reg(RegIndex::acc(w)))
    } else {
        (Loc::Imm8(bs.next().unwrap()), Loc::Reg(RegIndex::acc(w)))
    };
    Some(binop_to_instruction(BinopParams::Op(binop), src, dst))
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
enum BinOpCode {
    Add = 0b000,
    Sub = 0b101,
    Cmp = 0b111,
}

impl BinOpCode {
    const ALL : [Self; 3] = [Self::Add, Self::Sub, Self::Cmp];

    fn find(binop: u8) -> Option<Self> {
        Self::ALL.iter().find(|b| **b as u8 == binop).copied()
    }
}

const MOV_OPCODE: u8 = 0b_110_0011;
const MOV_OPCODE_LEN : u8 = 7;

const IMM_TO_R_M_OPCODE: u8 = 0b_10_0000;
const IMM_TO_R_M_OPCODE_LEN : u8 = 6;

fn parse_r_m_loc(bs: &mut impl Iterator<Item = u8>, mod_bits: u8, r_m_bits: u8, w: bool) -> Loc {
    match mod_bits {
            0b11 => Loc::Reg(parse_reg_field(r_m_bits, w)),
            0b00 if r_m_bits == 0b110 => {
                Loc::EAC(parse_r_m_direct_addr(consume_u16(bs)))
            },
            0b00 => Loc::EAC(parse_r_m_field(r_m_bits, None)),
            0b01 => {
                let displacement = (bs.next().unwrap() as i8) as i16;
                Loc::EAC(parse_r_m_field(r_m_bits, Some(displacement)))
            },
            0b10 => {
                let displacement = consume_i16(bs);
                Loc::EAC(parse_r_m_field(r_m_bits, Some(displacement)))
            },
            _ => panic!("unexpected MOD field: 0b_{:b}", mod_bits),
        }
}

fn parse_imm_to_r_m(b: u8, bs: &mut impl Iterator<Item = u8>) -> Option<Instruction> {
    let is_mov = b >> (8 - MOV_OPCODE_LEN) == MOV_OPCODE;
    let is_other_imm_to_r_m = b >> (8 - IMM_TO_R_M_OPCODE_LEN) == IMM_TO_R_M_OPCODE;
    if !is_mov && !is_other_imm_to_r_m {
        return None;
    }

    let b0 = bs.next().unwrap();
    let b1 = bs.next().unwrap();
    // XXXXXX: opcode
    // byte 0   byte 1
    // XXXXXXSW MOD|BINOP|R/M
    //           2    3    3
    let w = b0 & 0b_0000_0001 != 0; // is_wide
    // SPECIAL CASE:
    // for the MOV instruction, `s` can be considered as 
    // always 0
    let s = !is_mov && (b0 & 0b_0000_0010 != 0); // is_sign_extended
    let binop = BinOpCode::find((b1 >> 3) & 0b111);
    let mod_bits = (b1 & 0b_1100_0000) >> 6;
    let r_m_bits = b1 & 0b_0000_0111;

    let r_m_loc = parse_r_m_loc(bs, mod_bits, r_m_bits, w);
    let src = if w && !s {
        Loc::Imm16(consume_u16(bs))
    } else if w && s {
        // sign extending, not sure if i'm doing it right
        // TODO: make sure we have a test for the sign extension
        let imm16 = (bs.next().unwrap() as i8) as i16;
        let imm16 : u16 = unsafe { std::mem::transmute(imm16) };
        Loc::Imm16(imm16)
    } else {
        Loc::Imm8(bs.next().unwrap())
    };

    let params = BinopParams::from(is_mov, binop);
    Some(binop_to_instruction(params, src, r_m_loc))
}

#[derive(Clone, Copy)]
enum BinopParams {
    Mov,
    Op(BinOpCode),
}

impl BinopParams {
    fn from(is_mov: bool, code: Option<BinOpCode>) -> Self {
        if is_mov {
            Self::Mov
        } else {
            Self::Op(code.unwrap())
        }
    }
}

fn binop_to_instruction(params: BinopParams, src: Loc, dst: Loc) -> Instruction {
    match params {
        BinopParams::Mov => Instruction::Mov(Mov { src, dst }),
        BinopParams::Op(BinOpCode::Add) => Instruction::Add(Add { src, dst }),
        BinopParams::Op(BinOpCode::Sub) => Instruction::Sub(Sub { src, dst }),
        BinopParams::Op(BinOpCode::Cmp) => Instruction::Cmp(Cmp { src, dst }),
    }
}

fn consume_u16(bs: &mut impl Iterator<Item = u8>) -> u16 {
    u16::from_le_bytes([bs.next().unwrap(), bs.next().unwrap()])
}

fn consume_i16(bs: &mut impl Iterator<Item = u8>) -> i16 {
    i16::from_le_bytes([bs.next().unwrap(), bs.next().unwrap()])
}

fn consume_i8(bs: &mut impl Iterator<Item = u8>) -> i8 {
    i8::from_le_bytes([bs.next().unwrap()])
}

enum Region {
    Xtended, // 16 bits
    Low, // 8 bits
    High, // 8 bits
}

fn decode_mov(byte: u8, bytes: &mut impl Iterator<Item = u8>) -> Option<Mov> {
     if byte >> 4 == 0b_1011  {
         Some(parse_imm_to_reg_mov(bytes))
     } else if byte >> 1 == 0b_101_0000  {
         Some(parse_mem_to_acc_mov(bytes))
     } else if byte >> 1 == 0b_101_0001  {
         Some(parse_acc_to_mem_mov(bytes))
     } else {
         None
     }
}

fn decode(bytes: impl Iterator<Item = u8>) -> impl Iterator<Item = Instruction> {
    let mut bytes = bytes.peekable();
    std::iter::from_fn(move || {
        let byte = *bytes.peek()?;
        // catch alls
        if let Some(inst) = parse_imm_to_r_m(byte, &mut bytes) {
            Some(inst)
        } else if let Some(inst) = parse_r_m_to_r_m(byte, &mut bytes) {
            Some(inst)
        } else if let Some(inst) = parse_imm_to_acc(byte, &mut bytes) {
            Some(inst)
        } else if let Some(jump) = try_parse_jump(byte, &mut bytes) {
            Some(Instruction::Jump(jump))
        } else if let Some(mov) = decode_mov(byte, &mut bytes) {
            Some(Instruction::Mov(mov))
        } else {
            panic!("0b{:b}", byte);
        }
    })
}

// using https://edge.edx.org/c4x/BITSPilani/EEE231/asset/8086_family_Users_Manual_1_.pdf
// as reference for how to decode the instructions
fn main() {
    let mut args = std::env::args();
    args.next().unwrap();
    let filename = args.next().unwrap_or_else(
        || panic!("Must supply filename"));
    println!("bits 16");
    let bytes = std::fs::read(filename).unwrap().into_iter();
    for decoded in decode(bytes) {
        println!("{}", decoded.asm());
    }
}