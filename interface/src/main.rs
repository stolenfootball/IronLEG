use simulator::assembler::assembler;

fn main() {
    let asm = "HLT R1, R2";
    let instr = assembler::parse_line(asm);
    println!("{:032b}", instr);
}