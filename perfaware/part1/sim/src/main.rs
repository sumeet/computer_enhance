#![feature(variant_count)]

use std::io::Write;

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
    IP,
}

impl Reg {
    const fn num() -> usize {
        std::mem::variant_count::<Self>()
    }
}

#[derive(Clone, Copy)]
#[repr(u8)]
enum Flag {
    Parity = 0,
    Zero,
    Carry,
    Sign,
}

impl Flag {
    const fn num() -> usize {
        std::mem::variant_count::<Self>()
    }

    fn format(&self) -> char {
        match self {
            Flag::Parity => 'P',
            Flag::Zero => 'Z',
            Flag::Carry => 'C',
            Flag::Sign => 'S',
        }
    }
}

struct CPU {
    // not implementing segmented memory, otherwise we'd have more than 64k
    memory: [u8; u16::MAX as usize],
    // indexed by `Reg as usize`
    registers: [u16; Reg::num()],
    flags: [bool; Flag::num()],
}

fn check_parity(n: u16) -> bool {
    let lsb = n & 0xff;
    lsb.count_ones() % 2 == 0
}

fn check_sign(n: u16) -> bool {
    (n as i16) < 0
}

impl CPU {
    fn new() -> Self {
        Self {
            memory: [0; u16::MAX as usize],
            registers: [0; Reg::num()],
            flags: [false; Flag::num()],
        }
    }

    fn ip(&self) -> u16 {
        self.get_src(Loc::Reg(RegIndex::IP))
    }

    fn set_ip(&mut self, ip: u16) {
        self.set_dest(Loc::Reg(RegIndex::IP), ip);
    }

    // TODO: this would also manage internally the IP register, right now it's being done by the caller
    // also returns the jump offset
    fn exec(&mut self, inst: Instruction) -> i8 {
        match inst {
            Instruction::Mov(mov) => {
                let src = self.get_src(mov.src);
                self.set_dest(mov.dst, src);
            }
            Instruction::Jump(jump) => {
                let should_jump = match jump.typ {
                    JumpType::Jnz => !self.get_flag(Flag::Zero),
                    _ => todo!("other jumps not implemented"),
                };
                return if should_jump { jump.offset } else { 0 };
            }
            Instruction::Add(add) => {
                let src = self.get_src(add.src);
                let dst = self.get_src(add.dst);
                let (sum, is_overflow) = src.overflowing_add(dst);
                self.set_dest(add.dst, sum);
                self.set_flag(Flag::Parity, check_parity(sum));
                self.set_flag(Flag::Carry, is_overflow);
                self.set_flag(Flag::Zero, sum == 0);
                self.set_flag(Flag::Sign, check_sign(sum));
            }
            Instruction::Sub(sub) => {
                let src = self.get_src(sub.src);
                let (diff, is_overflow) = self.get_src(sub.dst).overflowing_sub(src);
                self.set_dest(sub.dst, diff);
                self.set_flag(Flag::Zero, diff == 0);
                self.set_flag(Flag::Parity, check_parity(diff));
                self.set_flag(Flag::Carry, is_overflow);
                self.set_flag(Flag::Sign, check_sign(diff));
            }
            Instruction::Cmp(cmp) => {
                // TODO: share code with sub, it's exactly the same except not storing the result
                let src = self.get_src(cmp.src);
                let dst = self.get_src(cmp.dst);
                let (diff, is_overflow) = src.overflowing_sub(dst);
                self.set_flag(Flag::Zero, diff == 0);
                self.set_flag(Flag::Parity, check_parity(diff));
                self.set_flag(Flag::Carry, is_overflow);
                self.set_flag(Flag::Sign, check_sign(diff));
            }
        }
        0
    }

    fn get_flag(&self, flag: Flag) -> bool {
        self.flags[flag as usize]
    }

    fn set_flag(&mut self, flag: Flag, val: bool) {
        self.flags[flag as usize] = val;
    }

    fn get_src(&self, loc: Loc) -> u16 {
        match loc {
            Loc::Imm8(n) => n as _,
            Loc::Imm16(n) => n as _,
            Loc::Reg(reg) => self.registers[reg.register as usize],
            Loc::EAC(eac) => {
                let offset = (self.get_offset(eac.base) as i32
                    + eac.displacement.unwrap_or(0) as i32) as usize;
                u16::from_le_bytes(self.memory[offset..offset + 2].try_into().unwrap())
            }
        }
    }

    fn set_dest(&mut self, loc: Loc, val: u16) {
        match loc {
            Loc::Reg(reg) => {
                self.registers[reg.register as usize] = val;
            }
            Loc::EAC(eac) => {
                let offset = (self.get_offset(eac.base) as i32
                    + eac.displacement.unwrap_or(0) as i32) as usize;
                let bytes = val.to_le_bytes();
                self.memory[offset..offset + bytes.len()].copy_from_slice(&bytes);
            }
            Loc::Imm8(_) | Loc::Imm16(_) => unreachable!(),
        }
    }

    fn get_offset(&self, base: EABase) -> u16 {
        match base {
            EABase::DirectAddr(n) => n,
            EABase::Bx => self.get_src(Loc::Reg(RegIndex::BX)),
            EABase::BpSi => {
                let bp = self.get_src(Loc::Reg(RegIndex::BP));
                let si = self.get_src(Loc::Reg(RegIndex::SI));
                bp.wrapping_add(si)
            }
            EABase::Bp => self.get_src(Loc::Reg(RegIndex::BP)),
            otherwise => panic!("TODO: get_offset for {:?}", otherwise),
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
    const ALL: [Self; 20] = [
        Self::Jnz,
        Self::Je,
        Self::Jl,
        Self::Jle,
        Self::Jb,
        Self::Jbe,
        Self::Jp,
        Self::Jo,
        Self::Js,
        Self::Jnl,
        Self::Jg,
        Self::Jnb,
        Self::Ja,
        Self::Jnp,
        Self::Jno,
        Self::Jns,
        Self::Loop,
        Self::Loopz,
        Self::Loopnz,
        Self::Jcxz,
    ];

    fn find(inst: u8) -> Option<Self> {
        Self::ALL.iter().find(|b| **b as u8 == inst).copied()
    }
}

fn try_parse_jump(b: u8, bs: &mut impl Iterator<Item = u8>) -> Option<Jump> {
    let typ = JumpType::find(b)?;
    bs.next().unwrap(); // advance the iterator forward 1 to consume the
                        // first byte
    Some(Jump {
        typ,
        offset: consume_i8(bs),
    })
}

struct Mov {
    src: Loc,
    dst: Loc,
}

impl Mov {
    fn asm(&self) -> String {
        format!(
            "mov {}, {}",
            self.dst.asm().to_lowercase(),
            self.src.asm().to_lowercase()
        )
    }
}

struct Add {
    src: Loc,
    dst: Loc,
}

impl Add {
    fn asm(&self) -> String {
        format!(
            "add {}, {}",
            self.dst.asm().to_lowercase(),
            self.src.asm().to_lowercase()
        )
    }
}

struct Sub {
    src: Loc,
    dst: Loc,
}

impl Sub {
    fn asm(&self) -> String {
        format!(
            "sub {}, {}",
            self.dst.asm().to_lowercase(),
            self.src.asm().to_lowercase()
        )
    }
}

struct Cmp {
    src: Loc,
    dst: Loc,
}

impl Cmp {
    fn asm(&self) -> String {
        format!(
            "cmp {}, {}",
            self.dst.asm().to_lowercase(),
            self.src.asm().to_lowercase()
        )
    }
}

#[derive(Clone, Copy)]
enum Loc {
    Reg(RegIndex),
    EAC(EAC),
    Imm8(u8),   // this is only applicable when Loc is a src
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
#[derive(Copy, Clone)]
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
            Some(d @ 0..) => format!("[{} + {}]", self.base.asm(), d),
            Some(d) => format!("[{} - {}]", self.base.asm(), -d),
        }
    }
}

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone)]
struct RegIndex {
    #[allow(unused)]
    region: Region,
    register: Reg,
    mnemonic: &'static str, // only used for printing assembly
}

impl RegIndex {
    const AL: RegIndex = RegIndex::new("AL", Reg::A, Region::Low);
    const AX: RegIndex = RegIndex::new("AX", Reg::A, Region::Xtended);
    const BX: RegIndex = RegIndex::new("BX", Reg::B, Region::Xtended);
    const CX: RegIndex = RegIndex::new("CX", Reg::C, Region::Xtended);
    const DX: RegIndex = RegIndex::new("DX", Reg::D, Region::Xtended);
    const SP: RegIndex = RegIndex::new("SP", Reg::SP, Region::Xtended);
    const BP: RegIndex = RegIndex::new("BP", Reg::BP, Region::Xtended);
    const SI: RegIndex = RegIndex::new("SI", Reg::SI, Region::Xtended);
    const DI: RegIndex = RegIndex::new("DI", Reg::DI, Region::Xtended);
    const IP: RegIndex = RegIndex::new("IP", Reg::IP, Region::Xtended);

    const fn new(mnemonic: &'static str, register: Reg, region: Region) -> Self {
        Self {
            mnemonic,
            register,
            region,
        }
    }

    fn asm(&self) -> &str {
        self.mnemonic
    }

    fn acc(w: bool) -> Self {
        if w {
            Self::AX
        } else {
            Self::AL
        }
    }

    fn is_acc(&self) -> bool {
        matches!(self.register, Reg::A)
    }
}

// this also works for the R/M field, if MOD = 0b11
// (register to register copy)
fn parse_reg_field(reg: u8, w: bool) -> RegIndex {
    use Region::*;
    match (reg, w) {
        (0b000, _) => RegIndex::acc(w),

        (0b001, false) => RegIndex::new("CL", Reg::C, Low),
        (0b001, true) => RegIndex::CX,

        (0b010, false) => RegIndex::new("DL", Reg::D, Low),
        (0b010, true) => RegIndex::DX,

        (0b011, false) => RegIndex::new("BL", Reg::B, Low),
        (0b011, true) => RegIndex::BX,

        (0b100, false) => RegIndex::new("AH", Reg::A, High),
        (0b100, true) => RegIndex::SP,

        (0b101, false) => RegIndex::new("CH", Reg::C, High),
        (0b101, true) => RegIndex::BP,

        (0b110, false) => RegIndex::new("DH", Reg::D, High),
        (0b110, true) => RegIndex::SI,

        (0b111, false) => RegIndex::new("BH", Reg::B, High),
        (0b111, true) => RegIndex::DI,

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
        .then(|| BinOpCode::find((opcode >> 1) & 0b111))
        .flatten();
    if !is_mov && binop.is_none() {
        return None;
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
    Mov {
        src,
        dst: Loc::Reg(dst),
    }
}

fn parse_imm_to_acc(b: u8, bs: &mut impl Iterator<Item = u8>) -> Option<Instruction> {
    // byte 0
    // 00BIN10W
    if b & 0b11_000_110 != 0b00_000_100 {
        // 00_xxx_10x
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
    const ALL: [Self; 3] = [Self::Add, Self::Sub, Self::Cmp];

    fn find(binop: u8) -> Option<Self> {
        Self::ALL.iter().find(|b| **b as u8 == binop).copied()
    }
}

const MOV_OPCODE: u8 = 0b_110_0011;
const MOV_OPCODE_LEN: u8 = 7;

const IMM_TO_R_M_OPCODE: u8 = 0b_10_0000;
const IMM_TO_R_M_OPCODE_LEN: u8 = 6;

fn parse_r_m_loc(bs: &mut impl Iterator<Item = u8>, mod_bits: u8, r_m_bits: u8, w: bool) -> Loc {
    match mod_bits {
        0b11 => Loc::Reg(parse_reg_field(r_m_bits, w)),
        0b00 if r_m_bits == 0b110 => Loc::EAC(parse_r_m_direct_addr(consume_u16(bs))),
        0b00 => Loc::EAC(parse_r_m_field(r_m_bits, None)),
        0b01 => {
            let displacement = (bs.next().unwrap() as i8) as i16;
            Loc::EAC(parse_r_m_field(r_m_bits, Some(displacement)))
        }
        0b10 => {
            let displacement = consume_i16(bs);
            Loc::EAC(parse_r_m_field(r_m_bits, Some(displacement)))
        }
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
        let imm16: u16 = unsafe { std::mem::transmute(imm16) };
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

#[derive(Copy, Clone)]
enum Region {
    Xtended, // 16 bits
    Low,     // 8 bits
    High,    // 8 bits
}

fn decode_mov(byte: u8, bytes: &mut impl Iterator<Item = u8>) -> Option<Mov> {
    if byte >> 4 == 0b_1011 {
        Some(parse_imm_to_reg_mov(bytes))
    } else if byte >> 1 == 0b_101_0000 {
        Some(parse_mem_to_acc_mov(bytes))
    } else if byte >> 1 == 0b_101_0001 {
        Some(parse_acc_to_mem_mov(bytes))
    } else {
        None
    }
}

// returns an instruction, and number of bytes in that instruction
fn decode_first_at(bytes: &[u8], ip: usize) -> (Instruction, usize) {
    let bytes = bytes[ip..].iter().copied();
    let mut bytes = CountingIterator::new(bytes);
    let next = decode_stream(&mut bytes).next().unwrap();
    (next, bytes.num_consumed)
}

fn decode_stream(bytes: &mut impl Iterator<Item = u8>) -> impl Iterator<Item = Instruction> + '_ {
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
    let filename = args
        .next()
        .unwrap_or_else(|| panic!("Must supply filename"));

    let flags = args.collect::<Vec<_>>();

    // third argument provided means we're running in sim mode
    let is_sim = flags.iter().find(|&f| f == "-exec").is_some();
    let is_image = flags.iter().find(|&f| f == "-image").is_some();
    let is_cycle_estimate = flags.iter().find(|&f| f == "-cycle-estimate").is_some();

    let bytes = std::fs::read(filename)
        .unwrap()
        .into_iter()
        .collect::<Vec<_>>();
    // only decode the instructions
    if !is_sim {
        println!("bits 16");

        let mut total = 0;

        for inst in decode_stream(&mut bytes.into_iter()) {
            print!("{}", inst.asm());

            if is_cycle_estimate {
                let est = estimate_8086(&inst);
                total += est;
                println!(" ; +{} = {}", est, total);
            } else {
                println!();
            }
        }

        if is_cycle_estimate {
            println!();
            println!("Total cycles: {}", total);
        }

        return;
    }

    let mut cpu = CPU::new();
    while (cpu.ip() as usize) < bytes.len() {
        let (inst, num_bytes) = decode_first_at(&bytes, cpu.ip() as usize);
        println!("{}", inst.asm());
        let jump_offset = cpu.exec(inst);
        let next_ip = (cpu.ip() as i32) + jump_offset as i32 + num_bytes as i32;
        cpu.set_ip(next_ip as u16);
    }

    println!("Final registers:");
    for reg in [
        RegIndex::AX,
        RegIndex::BX,
        RegIndex::CX,
        RegIndex::DX,
        RegIndex::SP,
        RegIndex::BP,
        RegIndex::SI,
        RegIndex::DI,
        RegIndex::IP,
    ] {
        let val = cpu.get_src(Loc::Reg(reg));
        println!(
            "      {}: {:#06x} ({})",
            reg.mnemonic.to_lowercase(),
            val,
            val
        );
    }

    print!("   flags: ");
    for flag in [Flag::Parity, Flag::Zero, Flag::Sign, Flag::Carry] {
        if cpu.get_flag(flag) {
            print!("{}", flag.format());
        }
    }
    print!("\n");

    if is_image {
        let mut f = std::fs::File::create("image.bin").unwrap();
        f.write_all(&cpu.memory).unwrap();
    }
}

struct CountingIterator<I: Iterator> {
    iter: I,
    num_consumed: usize,
}

impl<I: Iterator> CountingIterator<I> {
    pub fn new(iter: I) -> Self {
        CountingIterator {
            iter,
            num_consumed: 0,
        }
    }
}

impl<I: Iterator> Iterator for CountingIterator<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.iter.next();
        if item.is_some() {
            self.num_consumed += 1;
        }
        item
    }
}

// from table 2-21, on page 2-61 in the 8086 manual
fn estimate_8086(inst: &Instruction) -> usize {
    match inst {
        Instruction::Mov(mov) => match (mov.dst, mov.src) {
            // memory, accumulator
            (Loc::EAC(_), Loc::Reg(reg)) if reg.is_acc() => 10,
            // accumulator, memory
            (Loc::Reg(reg), Loc::EAC(_)) if reg.is_acc() => 10,
            // register, register
            (Loc::Reg(_), Loc::Reg(_)) => 2,
            // register, memory
            (Loc::Reg(_), Loc::EAC(eac)) => 8 + estimate_8086_eac(eac),
            // memory, register
            (Loc::EAC(eac), Loc::Reg(_)) => 9 + estimate_8086_eac(eac),
            // register, immediate
            (Loc::Reg(_), Loc::Imm8(_) | Loc::Imm16(_)) => 4,
            // memory, immediate
            (Loc::EAC(eac), Loc::Imm8(_) | Loc::Imm16(_)) => 10 + estimate_8086_eac(eac),
            _ => panic!("counting cycles for {} is not implemented yet", inst.asm()),
        },
        Instruction::Add(add) => match (add.dst, add.src) {
            // register, register
            (Loc::Reg(_), Loc::Reg(_)) => 3,
            // register, memory
            (Loc::Reg(_), Loc::EAC(eac)) => 9 + estimate_8086_eac(eac),
            // memory, register
            (Loc::EAC(eac), Loc::Reg(_)) => 16 + estimate_8086_eac(eac),
            // register (or accumulator), immediate
            (Loc::Reg(_), Loc::Imm8(_) | Loc::Imm16(_)) => 4,
            // memory, immediate
            (Loc::EAC(eac), Loc::Imm8(_) | Loc::Imm16(_)) => 17 + estimate_8086_eac(eac),
            _ => panic!("counting cycles for {} is not implemented yet", inst.asm()),
        },
        _ => panic!("counting cycles for {} is not implemented yet", inst.asm()),
    }
}

// from table 2-20, on page 2-51 in the 8086 manual
fn estimate_8086_eac(eac: EAC) -> usize {
    use EABase::*;
    match (eac.base, eac.displacement) {
        // displacement only
        (DirectAddr(_), None) => 6,
        // base or index only
        (Bx | Bp | Si | Di, None | Some(0)) => 5,
        // displacement + base or index
        (Bx | Bp | Si | Di, Some(_)) => 9,
        // base + index
        (BpDi | BxSi, None) => 7,
        (BpSi | BxDi, None) => 8,
        // displacement + base + index
        (BpDi, Some(_)) => 11,
        (BxSi, Some(_)) => 11,
        (BpSi, Some(_)) => 12,
        (BxDi, Some(_)) => 12,
        (DirectAddr(_), Some(_)) => panic!("direct addr + displacement is impossible"),
    }
}
