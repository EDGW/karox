.altmacro

/// Save GPR xn at sp+8n
.macro SAVE_GPR i
    sd x\i, 0 + \i*8(sp)
.endm

/// Load GPR xn from sp+8n
.macro LOAD_GPR i
    ld x\i, 0 + \i*8(sp)
.endm

/// Trap from kernel handler
/// 1. Save the task context to the kernel stack
/// 2. Call the handler
/// 3. Restore
/// A pointer to [TaskContext] is promised to be saved in 'sscratch' register when executing a task
.globl __trap_from_kernel_handler
.align 4
__trap_from_kernel_handler:
    addi sp, sp, -0x140 // make space for saving context

    // save register 1, 3-31 (without sp)
    SAVE_GPR 1
    .set n, 3
    .rept 29
        SAVE_GPR %n
        .set n, n+1
    .endr

    // save sstatus and sepc
    csrr t0, sstatus
    csrr t1, sepc
    sd  t0, 0x100(sp)
    sd  t1, 0x108(sp)

    // call
    mv  a0, sp // param1: trap context
    csrr a1, scause // param2: scause
    csrr a2, stval  // param3: stval
    call __handler
    
    // return
    // resume sstatus and sepc
    ld t0, 0x100(sp)
    ld t1, 0x108(sp)
    csrw sstatus, t0
    csrw sepc, t1

    // resume x1
    LOAD_GPR 1

    // return x3-x31
    .set n, 3
    .rept 29
        LOAD_GPR %n
        .set n, n+1
    .endr

    // resume sp
    addi sp, sp, 0x140 // restore space
    sret

/// Trap from user handler
/// 1. Save the task context to the task context area, usually in TCB, referenced by sscratch register
/// 2. Switch to kernel stack
/// 3. Make tp a pointer to the hart info struct of the current hart, which is needed before entering the kernel mode
/// 4. Call the handler
/// 5. Call the returner
/// A pointer to [TaskContext] is promised to be saved in 'sscratch' register when executing a task
.globl __trap_from_user_handler
.align 4
__trap_from_user_handler:
    csrrw sp, sscratch, sp // exchange 'sp' and 'sscratch' value.
                           // 'sscratch' is the pointer to the task context, 
                           // and now 'sscratch' stores the original 'sp' value

    // save register 1, 3-31 (without sp)
    SAVE_GPR 1
    .set n, 3
    .rept 29
        SAVE_GPR %n
        .set n, n+1
    .endr

    // save sstatus and sepc
    csrr t0, sstatus
    csrr t1, sepc
    sd  t0, 0x100(sp)
    sd  t1, 0x108(sp)

    // now we have all the registers except sp avaiable

    // save sp
    csrr t0, sscratch // read sp
    sd t0, 0 + 2*8(sp) // save sp

    mv s1, sp // trap context pointer. save it s1 because it is callee saved.

    // load tp and switch to kernel stack
    ld tp, 0x110(sp) // load tp
    ld sp, 0x118(sp) // set stack

    // call
    mv  a0, s1 // param1: trap context
    csrr a1, scause // param2: scause
    csrr a2, stval  // param3: stval
    call __handler

    // return
    // mv  s1, s1 param1: trap context 
    j __return_to_task

/// __return_to_user(context(s1))
/// 1. Resume the task context
/// 2. Set sscratch to task context pointer.
/// 3. Switch to user stack
/// 4. Return
.globl __return_to_task
__return_to_task:
    mv sp, s1
    // resume sstatus and sepc
    ld t0, 0x100(sp)
    ld t1, 0x108(sp)
    csrw sstatus, t0
    csrw sepc, t1

    // set task context pointer
    sd tp, 0x110(sp) // save tp
    csrw sscratch, tp

    // resume x1
    LOAD_GPR 1

    // return x3-x31
    .set n, 3
    .rept 29
        LOAD_GPR %n
        .set n, n+1
    .endr

    // resume sp
    ld    sp, 0 + 2*8(sp)
    sret