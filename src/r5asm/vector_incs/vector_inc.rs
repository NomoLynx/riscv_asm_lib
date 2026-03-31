use rust_macro::*;
use rust_macro_internal::*;

#[packet_bit_vec("src/r5asm/vector_incs/vector_inc.mmd")]
#[repr(C)]
#[derive(Clone, PartialEq, Eq)]
#[derive(Accessors)]
pub struct VectorInc {

}

/// test to generate a vector inc instruction and print its machine code
#[test]
fn test_vector_inc() {
    let opcode = 0b1010111u8;
    let rd = 0b00001u8;
    let funct3 = 0b000u8;
    let rs1 = 0b00010u8;
    let vs2 = 0b00011u8;
    let vm = 0b1u8;
    let funct6 = 0b000000u8;

    let mut inc = VectorInc::new();
    inc.set_opcode_bits(opcode.into());
    inc.set_rd_bits(rd.into());
    inc.set_funct3_bits(funct3.into());
    inc.set_rs1_bits(rs1.into());
    inc.set_vs2_bits(vs2.into());
    inc.set_vm_bits(vm.into());
    inc.set_funct6_bits(funct6.into());

    let machine_code = inc.to_le_bytes();
    
    assert_eq!(machine_code[0], 0x02);
    assert_eq!(machine_code[1], 0x31);
    assert_eq!(machine_code[2], 0x00);
    assert_eq!(machine_code[3], 0xD7);

}