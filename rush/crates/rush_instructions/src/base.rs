use rush_lib::inst::{
    CompileSignature, InstMetadata, InstSignature, PseudoExpand, PseudoSignature, RuntimeMetadata,
    RuntimeSignature,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct InstructionYaml {
    pub name: String,
    pub desc_short: Option<String>,
    pub desc_long: Option<String>,
    pub compile: CompileYaml,
    pub runtime: RuntimeYaml,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CompileYaml {
    pub format: Vec<ArgumentType>,
    #[serde(default)]
    pub relative_label: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeYaml {
    #[serde(rename = "type")]
    pub inst_type: InstructionType,
    pub opcode: Option<u8>,
    pub funct: Option<u8>,
    pub shamt: Option<u8>,
    pub rs: Option<u8>,
    pub rt: Option<u8>,
    pub rd: Option<u8>,
    pub reads: Vec<ReadsRegisterType>,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum InstructionType {
    R,
    I,
    J,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PseudoInstructionYaml {
    pub name: String,
    pub desc_short: Option<String>,
    pub desc_long: Option<String>,
    pub compile: CompileYaml,
    pub expand: Vec<InstructionExpansionYaml>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct InstructionExpansionYaml {
    pub inst: String,
    pub data: Vec<String>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArgumentType {
    Rd,
    Rs,
    Rt,
    Shamt,
    I16,
    U16,
    J,
    OffRs,
    OffRt,
    F32,
    F64,

    // pseudo
    Rx,
    I32,
    U32,
    Off32Rs,
    Off32Rt,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadsRegisterType {
    Rs,
    Rt,
    OffRs,
    OffRt,
}

impl From<InstructionYaml> for InstSignature {
    fn from(x: InstructionYaml) -> InstSignature {
        InstSignature::new(
            x.name.to_ascii_lowercase(),
            x.compile.into(),
            x.runtime.clone().into(),
            x.runtime.into(),
            InstMetadata::new(x.desc_short, x.desc_long),
        )
    }
}

impl From<CompileYaml> for CompileSignature {
    fn from(x: CompileYaml) -> CompileSignature {
        CompileSignature::new(
            x.format.into_iter().map(Into::into).collect(),
            x.relative_label,
        )
    }
}

impl From<RuntimeYaml> for RuntimeSignature {
    fn from(x: RuntimeYaml) -> RuntimeSignature {
        match x.inst_type {
            InstructionType::R => RuntimeSignature::R {
                opcode: x.opcode.unwrap_or(0),
                funct: x.funct.unwrap_or(0),
                shamt: x.shamt,
                rs: x.rs,
                rt: x.rt,
                rd: x.rd,
            },
            InstructionType::I => RuntimeSignature::I {
                opcode: x.opcode.expect("I-type requires opcode"),
                rt: x.rt,
            },
            InstructionType::J => RuntimeSignature::J {
                opcode: x.opcode.expect("J-type requires opcode"),
            },
        }
    }
}

impl From<RuntimeYaml> for RuntimeMetadata {
    fn from(x: RuntimeYaml) -> RuntimeMetadata {
        RuntimeMetadata::new(x.reads.into_iter().map(Into::into).collect())
    }
}

impl From<ArgumentType> for rush_lib::ArgumentType {
    fn from(x: ArgumentType) -> rush_lib::ArgumentType {
        match x {
            ArgumentType::Rd => rush_lib::ArgumentType::Rd,
            ArgumentType::Rs => rush_lib::ArgumentType::Rs,
            ArgumentType::Rt => rush_lib::ArgumentType::Rt,
            ArgumentType::Shamt => rush_lib::ArgumentType::Shamt,
            ArgumentType::I16 => rush_lib::ArgumentType::I16,
            ArgumentType::U16 => rush_lib::ArgumentType::U16,
            ArgumentType::J => rush_lib::ArgumentType::J,
            ArgumentType::OffRs => rush_lib::ArgumentType::OffRs,
            ArgumentType::OffRt => rush_lib::ArgumentType::OffRt,
            ArgumentType::F32 => rush_lib::ArgumentType::F32,
            ArgumentType::F64 => rush_lib::ArgumentType::F64,
            ArgumentType::Rx => panic!("Rx is not a real register -- it must be macroed away"),
            ArgumentType::I32 => rush_lib::ArgumentType::I32,
            ArgumentType::U32 => rush_lib::ArgumentType::U32,
            ArgumentType::Off32Rs => rush_lib::ArgumentType::Off32Rs,
            ArgumentType::Off32Rt => rush_lib::ArgumentType::Off32Rt,
        }
    }
}

impl From<ReadsRegisterType> for rush_lib::inst::ReadsRegisterType {
    fn from(x: ReadsRegisterType) -> rush_lib::inst::ReadsRegisterType {
        match x {
            ReadsRegisterType::Rs => rush_lib::inst::ReadsRegisterType::Rs,
            ReadsRegisterType::Rt => rush_lib::inst::ReadsRegisterType::Rt,
            ReadsRegisterType::OffRs => rush_lib::inst::ReadsRegisterType::OffRs,
            ReadsRegisterType::OffRt => rush_lib::inst::ReadsRegisterType::OffRt,
        }
    }
}

impl From<PseudoInstructionYaml> for PseudoSignature {
    fn from(x: PseudoInstructionYaml) -> PseudoSignature {
        PseudoSignature::new(
            x.name.to_ascii_lowercase(),
            x.compile.into(),
            x.expand.into_iter().map(Into::into).collect(),
        )
    }
}

impl From<InstructionExpansionYaml> for PseudoExpand {
    fn from(x: InstructionExpansionYaml) -> PseudoExpand {
        PseudoExpand::new(x.inst, x.data)
    }
}
