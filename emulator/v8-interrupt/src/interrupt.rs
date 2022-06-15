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
    fn code(self) -> u64 {
        use RvInterrupt::*;
        match self {
            UserSoftwareInterrupt => 0,
            SupervisorSoftwareInterrupt => 1,
            MachineSoftwareInterrupt => 3,
            UserTimerInterrupt => 4,
            SupervisorTimerInterrupt => 5,
            MachineTimerInterrupt => 7,
            UserExternalInterrupt => 8,
            SupervisorExternalInterrupt => 9,
            MachineExternalInterrupt => 11,
        }
    }
}
