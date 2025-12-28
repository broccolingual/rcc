pub(super) const ARG_REGS: [Reg; 6] = [Reg::Rdi, Reg::Rsi, Reg::Rdx, Reg::Rcx, Reg::R8, Reg::R9];

#[allow(dead_code)]
#[derive(Hash, Eq, PartialEq, Clone)]
pub(super) enum Reg {
    Rax,
    Rcx,
    Rdx,
    Rdi,
    Rsi,
    R8,
    R9,
    R10,
    R11,
}

impl Reg {
    pub(super) fn by_size(&self, size: usize) -> &'static str {
        match size {
            1 => self.byte(),
            2 => self.word(),
            4 => self.dword(),
            8 => self.qword(),
            _ => panic!("Unsupported register size: {}", size),
        }
    }

    pub(super) fn qword(&self) -> &'static str {
        match self {
            Reg::Rax => "rax",
            Reg::Rcx => "rcx",
            Reg::Rdx => "rdx",
            Reg::Rdi => "rdi",
            Reg::Rsi => "rsi",
            Reg::R8 => "r8",
            Reg::R9 => "r9",
            Reg::R10 => "r10",
            Reg::R11 => "r11",
        }
    }

    pub(super) fn dword(&self) -> &'static str {
        match self {
            Reg::Rax => "eax",
            Reg::Rcx => "ecx",
            Reg::Rdx => "edx",
            Reg::Rdi => "edi",
            Reg::Rsi => "esi",
            Reg::R8 => "r8d",
            Reg::R9 => "r9d",
            Reg::R10 => "r10d",
            Reg::R11 => "r11d",
        }
    }

    pub(super) fn word(&self) -> &'static str {
        match self {
            Reg::Rax => "ax",
            Reg::Rcx => "cx",
            Reg::Rdx => "dx",
            Reg::Rdi => "di",
            Reg::Rsi => "si",
            Reg::R8 => "r8w",
            Reg::R9 => "r9w",
            Reg::R10 => "r10w",
            Reg::R11 => "r11w",
        }
    }

    pub(super) fn byte(&self) -> &'static str {
        match self {
            Reg::Rax => "al",
            Reg::Rcx => "cl",
            Reg::Rdx => "dl",
            Reg::Rdi => "dil",
            Reg::Rsi => "sil",
            Reg::R8 => "r8b",
            Reg::R9 => "r9b",
            Reg::R10 => "r10b",
            Reg::R11 => "r11b",
        }
    }
}
