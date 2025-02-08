// SPDX-License-Identifier: MPL-2.0

//! Platform-specific code for the RISC-V platform.

pub mod boot;
pub(crate) mod cpu;
pub mod device;
pub mod iommu;
pub(crate) mod irq;
pub mod mm;
pub(crate) mod pci;
pub mod qemu;
pub mod serial;
pub mod task;
pub mod timer;
pub mod trap;

use core::{fmt::{self, Arguments, Write}, sync::atomic::Ordering};

use crate::task::{Task, KERNEL_STACK_SIZE};

#[cfg(feature = "cvm_guest")]
pub(crate) fn init_cvm_guest() {
    // Unimplemented, no-op
}

pub(crate) fn init_on_bsp() {
    // SAFETY: this function is only called once on BSP.
    unsafe {
        trap::init(true);
    }
    irq::init();

    // SAFETY: they are only called once on BSP and ACPI has been initialized.
    unsafe {
        crate::cpu::init_num_cpus();
        crate::cpu::set_this_cpu_id(0);
    }

    // SAFETY: no CPU local objects have been accessed by this far. And
    // we are on the BSP.
    unsafe { crate::cpu::local::init_on_bsp() };

    crate::boot::smp::boot_all_aps();
}

pub(crate) unsafe fn init_on_ap() {
    unimplemented!()
}

pub(crate) fn interrupts_ack(irq_number: usize) {
    unimplemented!()
}

/// Return the frequency of TSC. The unit is Hz.
pub fn tsc_freq() -> u64 {
    timer::TIMEBASE_FREQ.load(Ordering::Relaxed)
}

/// Reads the current value of the processor’s time-stamp counter (TSC).
pub fn read_tsc() -> u64 {
    unimplemented!()
}

pub(crate) fn enable_cpu_features() {
    // Unimplemented, no-op
}

extern "Rust" {
    /// Informs KernMiri to alloc `count` pages at `paddr`.
    ///
    /// Kernel should not allocate a page at the same address twice. 
    /// If an address that has already been allocated is allocated again 
    /// before being deallocated, KernMiri will treat it as UB.
    pub fn kern_miri_alloc_pages(paddr: usize, count: usize);
    
    /// Informs KernMiri to dealloc `count` pages at `paddr`.
    /// 
    /// The kernel should only deallocate an address that 
    /// has already been allocated by `kern_miri_alloc_pages`;
    /// otherwise, such behavior will be considered UB.
    pub fn kern_miri_dealloc_pages(paddr: usize, count: usize);
    
    /// Informs KernMiri to retype `count` pages at `paddr` to `page_type`.
    ///
    /// After retyping, the page will become a typed page, 
    /// and the memory on the page will be converted into 
    /// a contiguous series of slots of the same size. 
    /// The kernel can only retype a page that has been allocated, 
    /// and the same page cannot be retyped twice, otherwise, 
    /// it will be considered UB.
    pub fn kern_miri_retype_pages(paddr: usize, count: usize, page_type: PageType, type_size: usize);
    
    pub fn kern_miri_zero(paddr: usize, count: usize);

    // u8, u16, u32, u64 untyped read/write operation and untyped copy operation. If the operated `ptr` points to a unused or typed memory, this operation will be treated as UB.
    
    pub fn kern_miri_read_u8_untyped(ptr: *const u8) -> u8;

    pub fn kern_miri_write_u8_untyped(ptr: *mut u8, value: u8);
    
    pub fn kern_miri_copy_untyped(dst: *const u8, src: *const u8, len: usize);

    pub fn kern_miri_log(info: usize);

    pub fn kern_miri_copy(dst: usize, src: usize, len: usize);

    pub fn kern_miri_get_cpu_local_va(cpu_local_va: usize) -> usize;

    pub fn kern_miri_init_ap(cpu_id: usize, func: fn(usize), arg: usize, task: &Task, stack_end: usize, stack_size: usize) -> usize;

    fn miri_write_to_stdout(bytes: &[u8]);

    pub fn kern_miri_record_time(index: usize);

    pub fn kern_miri_get_cpu_local_base() -> usize;

    pub fn kern_miri_set_cpu_local_base(base_vaddr: usize);
}

/// The type of the typed page, used to inform miri which 
/// type of typed page the current page should be retyped into.
#[repr(usize)]
#[derive(Clone, Copy)]
pub enum PageType {
    Slab = 1,
    PageTable = 2,
    Stack = 3,
    Interpreter = 4,
}

pub struct MiriStdout;

impl Write for MiriStdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe {
            miri_write_to_stdout(s.as_bytes());
        }
        Ok(())
    }
}

/// Prints the formatted arguments to the standard output.
pub fn _miri_print(args: Arguments) {
    MiriStdout.write_fmt(args).unwrap();
}

/// Copied from Rust std: <https://github.com/rust-lang/rust/blob/master/library/std/src/macros.rs>
#[macro_export]
macro_rules! miri_print {
    ($($arg:tt)*) => {{
        $crate::arch::miri::_miri_print(format_args!($($arg)*));
    }};
}

/// Copied from Rust std: <https://github.com/rust-lang/rust/blob/master/library/std/src/macros.rs>
#[macro_export]
macro_rules! miri_println {
    () => {
        $crate::miri_print!("\n")
    };
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::arch::miri::_miri_print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    };
}

#[macro_export]
macro_rules! miri_record {
    ($index:expr) => {
        $crate::arch::miri::kern_miri_record_time($index);
    };
    ($index1:expr, $index2:expr) => {
        $crate::arch::miri::kern_miri_record_time($index1);
        $crate::arch::miri::kern_miri_record_time($index2);
    };
}