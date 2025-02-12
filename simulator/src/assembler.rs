
pub mod assembler {
    use nom::{
        IResult,
        bytes::complete::tag,
        branch::alt,
        sequence::{preceded, terminated},
        combinator::{recognize, value, map_res, map},
        character::complete::{one_of, char ,digit1},
        multi::{many0, many1, separated_list1},
    };

    #[derive(Clone, Copy, Debug)]
    enum InstrType {
        ALU       = 0b000,
        Memory    = 0b001,
        Control   = 0b010,
        Interrupt = 0b011,
    }

    #[derive(Clone, Copy, Debug)]
    enum AddrMode {
        Reg = 0b000,
        Imm = 0b001,
    }


    fn parse_alu(input: &str) -> IResult<&str, (InstrType, u32)> {
        alt((
            value((InstrType::ALU, 0b0000), tag("MOV")),
            value((InstrType::ALU, 0b0001), tag("ADD")),
            value((InstrType::ALU, 0b0010), tag("SUB")),
            value((InstrType::ALU, 0b0011), tag("IMUL")),
            value((InstrType::ALU, 0b0100), tag("IDIV")),
            value((InstrType::ALU, 0b0101), tag("AND")),
            value((InstrType::ALU, 0b0110), tag("OR")),
            value((InstrType::ALU, 0b0111), tag("XOR")),
            value((InstrType::ALU, 0b1000), tag("CMP")),
            value((InstrType::ALU, 0b1001), tag("MOD")),
            value((InstrType::ALU, 0b1010), tag("NOT")),
            value((InstrType::ALU, 0b1011), tag("LSL")),
            value((InstrType::ALU, 0b1100), tag("LSR")),
        ))(input)
    }

    fn parse_memory(input: &str) -> IResult<&str, (InstrType, u32)> {
        alt((
            value((InstrType::Memory, 0b0000), tag("LDR")),
            value((InstrType::Memory, 0b0001), tag("STR")),
        ))(input)
    }

    fn parse_control(input: &str) -> IResult<&str, (InstrType, u32)> {
        alt((
            value((InstrType::Control, 0b0000), tag("BEQ")),
            value((InstrType::Control, 0b0001), tag("BLT")),
            value((InstrType::Control, 0b0010), tag("BGT")),
            value((InstrType::Control, 0b0011), tag("BNE")),
            value((InstrType::Control, 0b0100), tag("B")),
            value((InstrType::Control, 0b0101), tag("CALL")),
            value((InstrType::Control, 0b0110), tag("RET")),
            value((InstrType::Control, 0b0111), tag("BGE")),
            value((InstrType::Control, 0b1000), tag("BLE")),
        ))(input)
    }

    fn parse_interrupt(input: &str) -> IResult<&str, (InstrType, u32)> {
        alt((
            value((InstrType::Interrupt, 0b0000), tag("NOP")),
            value((InstrType::Interrupt, 0b0001), tag("HLT")),
        ))(input)
    }

    fn parse_regs(input: &str) -> IResult<&str, (AddrMode, u32)> {
        alt((
            value((AddrMode::Reg, 0b00000), tag("R0")),
            value((AddrMode::Reg, 0b00001), tag("R1")),
            value((AddrMode::Reg, 0b00010), tag("R2")),
            value((AddrMode::Reg, 0b00011), tag("R3")),
            value((AddrMode::Reg, 0b00100), tag("R4")),
            value((AddrMode::Reg, 0b00101), tag("R5")),
            value((AddrMode::Reg, 0b00110), tag("R6")),
            value((AddrMode::Reg, 0b00111), tag("R7")),
            value((AddrMode::Reg, 0b01000), tag("R8")),
            value((AddrMode::Reg, 0b01001), tag("R9")),
            value((AddrMode::Reg, 0b01010), tag("R10")),
            value((AddrMode::Reg, 0b01011), tag("R11")),
            value((AddrMode::Reg, 0b01100), tag("SP")),
            value((AddrMode::Reg, 0b01101), tag("BF")),
            value((AddrMode::Reg, 0b01111), tag("LR")),
            value((AddrMode::Reg, 0b10000), tag("PC")),
        ))(input)
    }

    fn parse_hex(input: &str) -> IResult<&str, u32> {
        map_res(
        preceded(
            alt((tag("0x"), tag("0X"))),
            recognize(
                many1(
                    terminated(one_of("0123456789abcdefABCDEF"), many0(char('_')))
                )
            )
        ),
        |out: &str| u32::from_str_radix(&str::replace(&out, "_", ""), 16)
        )(input)
    }

    fn parse_nums(input: &str) -> IResult<&str, (AddrMode, u32)> {
        map(
            alt((parse_hex, map_res(digit1, str::parse::<u32>))),
            |out| (AddrMode::Imm, out)
        )(input)
    }

    fn parse_comma_sep(input: &str) -> IResult<&str, Vec<(AddrMode, u32)>> {
        separated_list1(tag(","), alt((parse_regs, parse_nums)))(input)
    }

    pub fn parse_line(input: &str) -> u32 {
        let (input, (instr_type, opcode)) = alt((parse_alu, parse_memory, parse_control, parse_interrupt))(input).unwrap();
        let input = input.chars().filter(|c| !c.is_whitespace()).collect::<String>();
        
        let (_, regs) = parse_comma_sep(&input).unwrap();
        
        let addr_type = regs.iter().map(|(mode, _)| *mode as u32).fold(0, |acc, x| acc << 1 | x);

        let output = (instr_type as u32) << 29 | opcode << 25 | addr_type << 22;
        match addr_type {
            0b000 => output | regs.iter().map(|(_, val)| val).fold(0, |acc, x| acc << 4 | x) << 16,
            _ => output | regs.iter().map(|(_, val)| val).fold(0, |acc, x| acc << 16 | x),
        }
    }


}