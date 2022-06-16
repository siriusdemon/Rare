pub const MASK_INTERRUPT_BIT: u64 = 1 << 63;

#[derive(Debug, Copy, Clone)]
pub enum RvInterrupt {
    UserSoftwareInterrupt,
    SupervisorSoftwareInterrupt,
    MachineSoftwareInterrupt,
    UserTimerInterrupt,
    SupervisorTimerInterrupt,
    MachineTimerInterrupt,
    UserExternalInterrupt,
    SupervisorExternalInterrupt,
    MachineExternalInterrupt,
}



impl RvInterrupt {
    pub fn code(self) -> u64 {
        use RvInterrupt::*;
        match self {
            UserSoftwareInterrupt => 0 | MASK_INTERRUPT_BIT,
            SupervisorSoftwareInterrupt => 1 | MASK_INTERRUPT_BIT,
            MachineSoftwareInterrupt => 3 | MASK_INTERRUPT_BIT,
            UserTimerInterrupt => 4 | MASK_INTERRUPT_BIT,
            SupervisorTimerInterrupt => 5 | MASK_INTERRUPT_BIT,
            MachineTimerInterrupt => 7 | MASK_INTERRUPT_BIT,
            UserExternalInterrupt => 8 | MASK_INTERRUPT_BIT,
            SupervisorExternalInterrupt => 9 | MASK_INTERRUPT_BIT,
            MachineExternalInterrupt => 11 | MASK_INTERRUPT_BIT,
        }
    }
}
