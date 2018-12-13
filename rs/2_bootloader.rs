#![no_std]
#![no_main]
#![feature(asm)]

mod cop0 {
    pub fn prid() -> u32 {
        let prid;
        unsafe { asm!("mfc0 $0, $$15" : "=r" (prid)) };
        prid
    }
}

fn ee_setup() -> ! {
    unimplemented!("EE code goes here");
}

fn iop_setup() -> ! {
    unimplemented!("IOP code goes here");
}

fn _start() -> ! {
    let prid = cop0::prid();
    let model = prid & 0xFF00;

    match model {
        0x2E00 => ee_setup(),
        0x0000 => iop_setup(),
        _ => unimplemented!("Couldn't detect host processor"),
    }
}
