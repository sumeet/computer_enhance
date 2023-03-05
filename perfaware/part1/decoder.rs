struct Mov {
    src: Reg,
    dst: Reg,
}

impl Mov {
    fn asm(&self) -> String {
        "".to_string()
    }
}

fn parse_mov_100_010_xx(bs: [u8; 2]) -> Mov {
    // bit 0    bit 1
    // 100010DW MOD|REG|R/M
    //           2   3   3
    let is_wide = bs[0] & 0b00000001 != 0;
    let mod_bits = bs[0];
    let reg_bits = bs[1];
    let rm_bits = bs[2];

    // if the d_bit is 1 then reg register is destination
    // if the d_bit is 0 then reg register is source
    let d_bit = bs[0] & 0b00000010 != 0;
    let destination = if d_bit {  } else { };
    Mov {
        src: todo!(),
        dst: todo!(),
    }
}


struct Reg {
    region: Region,
    name: char,
}

enum Region {
    Extended,
    Low,
    High,
}

fn main() {
    let mut args = std::env::args();
    args.next().unwrap();
    let filename = args.next().unwrap_or_else(
        || panic!("Must supply filename"));
    let bytes = std::fs::read(filename).unwrap();

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
    println!("bits 16");
}
