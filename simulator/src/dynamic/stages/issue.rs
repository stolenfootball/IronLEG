use crate::memory::{Memory, MemoryValue, StageType};
use super::super::registers::{self, Registers, Register};
use super::super::reservation::ReservationStations;
use super::super::instruction::{self, Operation, AddrMode};

fn fetch(mem: Box<&mut dyn Memory>, regs: &mut Registers) {
    if let Register::Value(next_instr_addr) = regs.registers[registers::PC] {
        if let Some(MemoryValue::Value(value)) =
            mem.read(next_instr_addr, StageType::Fetch, false)
        {
            regs.instr_queue.push_back(value);
            regs.registers[registers::PC] = Register::Value(value + 4);
        }
    }
}

fn decode(
    regs: &mut Registers,
    stations: &mut ReservationStations,
    instr: usize,
) {

    if let Some(station) = stations.get_free_station() {

        let instr_type = instr >> 29;
        let opcode = (instr >> 25) & 0xF;
        let addr_mode = (instr >> 22) & 0x7;

        station.operation = Some(Operation::decode(instr_type, opcode));

        let reg_1 = (instr >> 18) & 0xF;
        station.argument_1 = Some(regs.registers[reg_1]);
        station.destination = Some(reg_1);

        match AddrMode::from_usize(addr_mode) {
            AddrMode::RegReg => station.argument_2 = Some(regs.registers[(instr >> 14) & 0xF]),
            AddrMode::RegRegOff => {
                station.argument_2 = Some(regs.registers[(instr >> 14) & 0xF]);
                station.immediate = Some(instr & 0xFFFF);
            },
            AddrMode::RegImm => station.immediate = Some(instr & 0xFFF),
            AddrMode::Imm => {
                station.argument_1 = None;
                station.immediate = Some(instr & 0x3FFFFF);
                station.destination = None;
            },
            AddrMode::Reg => (),
        };

        if let Some(Operation::ALU(value)) = station.operation {
            if value == instruction::ALUType::CMP {
                station.destination = Some(registers::BF);
            }
        }

        if let Some(Operation::Control(_)) = station.operation {
            station.destination = Some(registers::BF);
        }
    }
    
}

pub fn issue(
    regs: &mut Registers,
    stations: &mut ReservationStations,
    mem: Box<&mut dyn Memory>,
) {
    fetch(mem, regs);
    if let Some(instr) = regs.instr_queue.pop_back() {
        decode(regs, stations, instr);
    } 
}