use crate::arch::trap::context::TrapContext;

pub const EXCEPTION_DESC: [&'static str; 16] = {
    let mut res = ["Reserved or Designated for Custom Use"; 16];
    res[0] = "Instruction Address Misaligned";
    res[1] = "Instruction Access Fault";
    res[2] = "Illegal Instruction";
    res[3] = "Breakpoint";
    res[4] = "Load Address Misaligned";
    res[5] = "Load Access Fault";
    res[6] = "Store/AMO Address Misaligned";
    res[7] = "Store/AMO Access Fault";
    res[8] = "Environment Call from U-mode";
    res[9] = "Environment Call from S-mode";
    res[12] = "Instruction Page Fault";
    res[13] = "Load Page Fault";
    res[15] = "Store/AMO Page Fault";
    res
};

pub fn get_exception_desc(code: usize) -> &'static str {
    if code >= EXCEPTION_DESC.len() {
        "Reserved or Designated for Custom Use"
    } else {
        EXCEPTION_DESC[code]
    }
}
pub fn exception_handler(exception_code: usize, context: &mut TrapContext, _stval: usize) {
    panic!(
        "Unexcepted Exception {:#x}({:}) Occurred in kernel at {:#x}",
        exception_code,
        get_exception_desc(exception_code),
        context.sepc
    );
}
