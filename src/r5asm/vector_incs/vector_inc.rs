use rust_macro::*;
use rust_macro_internal::*;

#[csv2enum_variants("src/r5asm/vector_incs/value_from.csv", type_name)]
pub enum ValueForm { }

csv2lookup!("src/r5asm/vector_incs/value_from.csv", type_name, type_size, ValueForm);

#[csv2enum_variants("src/r5asm/vector_incs/value_op.csv", type_name)]
pub enum ValueOp { }

csv2lookup!("src/r5asm/vector_incs/value_op.csv", type_name, type_size, ValueOp);

#[csv2enum_variants("src/r5asm/vector_incs/vred_op.csv", name)]
pub enum VRedOp { }

csv2lookup!("src/r5asm/vector_incs/vred_op.csv", name, value, VRedOp);

#[csv2enum_variants("src/r5asm/vector_incs/vmask_op.csv", name)]
pub enum VMaskOp { }

csv2lookup!("src/r5asm/vector_incs/vmask_op.csv", name, value, VMaskOp);

#[csv2enum_variants("src/r5asm/vector_incs/vwidth.csv", name)]
pub enum VWidth { }

csv2lookup!("src/r5asm/vector_incs/vwidth.csv", name, size, VWidth);

#[csv2enum_variants("src/r5asm/vector_incs/vstore_load.csv", name)]
pub enum VStoreKind { }

csv2lookup!("src/r5asm/vector_incs/vstore_load.csv", name, size, VStoreKind);

#[csv2enum_variants("src/r5asm/vector_incs/vstore_load.csv", name)]
pub enum VLoadKind { }

csv2lookup!("src/r5asm/vector_incs/vstore_load.csv", name, size, VLoadKind);

pub enum ValueSrc {
    Vs1(u8),   // for VV
    Rs1(u8),   // for VX
    Imm(i8),   // for VI
}

#[packet_bit_vec("src/r5asm/vector_incs/vector_inc.mmd")]
#[repr(C)]
#[derive(Clone, PartialEq, Eq)]
#[derive(Accessors)]
pub struct VectorInc {

}

impl VectorInc {
    fn encode_rvv_base(
        opcode: u8,
        funct3: u8,
        funct6: u8,
        vd: u8,
        vs2: u8,
        rs1: u8,
        vm: bool,
    ) -> VectorInc {
        let mut inc = VectorInc::new();
        inc.set_opcode_bits(opcode.into());
        inc.set_rd_bits(vd.into());
        inc.set_funct3_bits(funct3.into());
        inc.set_rs1_bits(rs1.into());
        inc.set_vs2_bits(vs2.into());
        inc.set_vm_bits((vm as u8).into());
        inc.set_funct6_bits(funct6.into());
        inc
    }

    pub fn encode_value(
        op: ValueOp,
        form: ValueForm,
        vd: u8,
        vs2: u8,
        src: ValueSrc,
        vm: bool,
    ) -> VectorInc {
        let opcode = 0b1010111;

        let funct3 = form.lookup_size_valueform().unwrap() as u8;

        let funct6 = op.lookup_size_valueop().unwrap() as u8;

        let (rs1, vs2_final) = match (form, src) {
            (ValueForm::VV, ValueSrc::Vs1(vs1)) => (vs1, vs2),
            (ValueForm::VX, ValueSrc::Rs1(rs1)) => (rs1, vs2),
            (ValueForm::VI, ValueSrc::Imm(imm)) => ((imm as u8) & 0b11111, vs2),
            _ => panic!("Invalid VALU combination"),
        };

        Self::encode_rvv_base(opcode, funct3, funct6, vd, vs2_final, rs1, vm)
    }

    pub fn encode_vload(
        kind: VLoadKind,
        width: VWidth,
        vd: u8,
        base: u8,
        index_or_stride: Option<u8>,
    ) -> VectorInc {
        let opcode = 0b0000111;

        let funct3 = width.lookup_size_vwidth().unwrap() as u8;

        let funct6 = match kind {
            VLoadKind::UnitStride => 0b000000,
            VLoadKind::Strided    => 0b010000,
            VLoadKind::Indexed    => 0b100000,
        };

        let rs2 = index_or_stride.unwrap_or(0);

        Self::encode_rvv_base(opcode, funct3, funct6, vd, rs2, base, true)
    }

    pub fn encode_vstore(
        kind: VStoreKind,
        width: VWidth,
        vs3: u8,
        base: u8,
        index_or_stride: Option<u8>,
    ) -> VectorInc {
        let opcode = 0b0100111;

        let funct3 = width.lookup_size_vwidth().unwrap() as u8;

        let funct6 = kind.lookup_size_vstorekind().unwrap() as u8;

        let rs2 = index_or_stride.unwrap_or(0);

        Self::encode_rvv_base(opcode, funct3, funct6, vs3, rs2, base, true)
    }

    pub fn encode_vreduction(
        op: VRedOp,
        vd: u8,
        vs2: u8,
        vs1: u8,
        vm: bool,
    ) -> VectorInc {
        let opcode = 0b1010111;
        let funct3 = 0b010;

        let funct6 = op.lookup_size_vredop().unwrap() as u8;

        Self::encode_rvv_base(opcode, funct3, funct6, vd, vs2, vs1, vm)
    }

    pub fn encode_vmask(
        op: VMaskOp,
        vd: u8,
        vs1: u8,
        vs2: Option<u8>,
    ) -> VectorInc {
        let opcode = 0b1010111;
        let funct3 = 0b010;

        let funct6 = op.lookup_size_vmaskop().unwrap() as u8;

        let vs2_val = vs2.unwrap_or(0);

        Self::encode_rvv_base(opcode, funct3, funct6, vd, vs2_val, vs1, true)
    }
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