initSidebarItems({"fn":[["deregister_interrupt","Returns an interrupt to the system by setting the handler to the default function.  The application provides the current interrupt handler as a safety check.  The function fails if the current handler and 'func' do not match"],["eoi","Send an end of interrupt signal, which works for all types of interrupt chips (APIC, x2apic, PIC) irq arg is only used for PIC"],["init","initializes the interrupt subsystem and properly sets up safer early exception handlers, but no other IRQ handlers. # Arguments:  * `double_fault_stack_top_unusable`: the address of the top of a newly allocated stack, to be used as the double fault exception handler stack. * `privilege_stack_top_unusable`: the address of the top of a newly allocated stack, to be used as the privilege stack (Ring 3 -> Ring 0 stack)."],["init_ap","Similar to `init()`, but for APs to call after the BSP has already invoked `init()`."],["init_handlers_apic",""],["init_handlers_pic",""],["is_exception_handler_with_error_code","Returns `true` if the given address is the exception handler in the current `IDT` for any exception in which the CPU pushes an error code onto the stack."],["register_interrupt","Registers an interrupt handler.  The function fails if the interrupt number is already in use. "],["register_msi_interrupt","Returns an interrupt number assigned by the OS and sets its handler function.  The function fails if there is no unused interrupt number."]],"static":[["APIC_TIMER_TICKS",""],["IDT","The single system-wide IDT Note: this could be per-core instead of system-wide, if needed."]]});