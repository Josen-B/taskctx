use core::arch::naked_asm;
use memory_addr::VirtAddr;

/// Saved hardware states of a task.
///
/// The context usually includes:
///
/// - Callee-saved registers
/// - Stack pointer register
/// - Thread pointer register (for thread-local storage, currently unsupported)
/// - FP/SIMD registers
///
/// On context switch, current task saves its context from CPU to memory,
/// and the next task restores its context from memory to CPU.
#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Default)]
pub struct TaskContext {
    pub ra: usize, // return address (x1)
    pub sp: usize, // stack pointer (x2)

    pub s0: usize, // x8-x9
    pub s1: usize,

    pub s2: usize, // x18-x27
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,

    pub tp: usize,
    // TODO: FP states
}

impl TaskContext {
    /// Creates a new default context for a new task.
    pub const fn new() -> Self {
        unsafe { core::mem::MaybeUninit::zeroed().assume_init() }
    }

    /// Initializes the context for a new task, with the given entry point and
    /// kernel stack.
    pub fn init(&mut self, entry: usize, kstack_top: VirtAddr, tls_area: VirtAddr) {
        self.sp = kstack_top.as_usize();
        self.ra = entry;
        self.tp = tls_area.as_usize();
    }

    pub fn thread_saved_fp(&self) -> usize {
        self.s0
    }

    pub fn thread_saved_pc(&self) -> usize {
        self.ra
    }
}

#[cfg(target_arch = "riscv32")]
macro_rules! STR_LDR {
    () => {
        r"
.ifndef XLENB
.equ XLENB, 4

.macro STR rs2, rs1, off
    sw \rs2, \off*XLENB(\rs1)
.endm
.macro LDR rd, rs, off
    lw \rd, \off*XLENB(\rs)
.endm

.endif"
    };
}

#[cfg(target_arch = "riscv64")]
macro_rules! STR_LDR {
    () => {
        r"
.ifndef XLENB
.equ XLENB, 8

.macro STR rs2, rs1, off
    sd \rs2, \off*XLENB(\rs1)
.endm
.macro LDR rd, rs, off
    ld \rd, \off*XLENB(\rs)
.endm

.endif"
    };
}

#[naked]
/// Switches the context from the current task to the next task.
///
/// # Safety
///
/// This function is unsafe because it directly manipulates the CPU registers.
pub unsafe extern "C" fn context_switch(_current_task: &mut TaskContext, _next_task: &TaskContext) {
    unsafe {
        naked_asm!(
            STR_LDR!(),
            "
            // save old context (callee-saved registers)
            STR     ra, a0, 0
            STR     sp, a0, 1
            STR     s0, a0, 2
            STR     s1, a0, 3
            STR     s2, a0, 4
            STR     s3, a0, 5
            STR     s4, a0, 6
            STR     s5, a0, 7
            STR     s6, a0, 8
            STR     s7, a0, 9
            STR     s8, a0, 10
            STR     s9, a0, 11
            STR     s10, a0, 12
            STR     s11, a0, 13

            // restore new context
            LDR     s11, a1, 13
            LDR     s10, a1, 12
            LDR     s9, a1, 11
            LDR     s8, a1, 10
            LDR     s7, a1, 9
            LDR     s6, a1, 8
            LDR     s5, a1, 7
            LDR     s4, a1, 6
            LDR     s3, a1, 5
            LDR     s2, a1, 4
            LDR     s1, a1, 3
            LDR     s0, a1, 2
            LDR     sp, a1, 1
            LDR     ra, a1, 0

            ret",
        )
    }
}
