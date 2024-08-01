use core::ptr;
use libc::{shm_unlink, shmat, shmget, IPC_CREAT, IPC_EXCL, MAP_SHARED, PROT_READ, PROT_WRITE};
use num_traits::ToPrimitive;
use std::{
    env,
    os::fd::RawFd,
    println,
    string::ToString,
    sync::{LazyLock, Mutex},
    vec,
    vec::Vec,
};
const MAP_SIZE: usize = 65536;

pub static CURR_POS: LazyLock<Mutex<i32>> = LazyLock::new(|| Mutex::new(0));

pub struct SharedMem {}

impl SharedMem {
    pub fn save(by: isize) -> () {
        let key_str = env::var("__AFL_SHM_ID");
        match key_str {
            Ok(key) => {
                let keyyyy = key
                    .parse::<i32>()
                    .expect("Failed to parse __AFL_SHM_ID as i32");

                println!("KEY = {}", keyyyy);
                let size: usize = 65536;

                // Try to create a new shared memory segment
                let mut shmid = unsafe { shmget(keyyyy, size, IPC_CREAT | IPC_EXCL | 0o600) };

                if shmid == -1 {
                    // If creation fails, try to get the existing segment
                    shmid = unsafe { shmget(keyyyy, size, 0o600) };

                    if shmid == -1 {
                        let error_code = std::io::Error::last_os_error();
                        panic!(
                            "shmget failed with error code {}: {}",
                            error_code.raw_os_error().unwrap(),
                            error_code
                        );
                    }
                }

                // Attach to the shared memory segment
                let shmptr = unsafe { shmat(shmid, ptr::null(), 0) };
                if shmptr == ptr::null_mut() {
                    let error_code = std::io::Error::last_os_error();
                    panic!(
                        "shmat failed with error code {}: {}",
                        error_code.raw_os_error().unwrap(),
                        error_code
                    );
                }

                let mut curr_pos = CURR_POS.lock().unwrap();

                *curr_pos += by as i32;
                let pos = *curr_pos as usize;

                // Update the value at the specified offset
                unsafe {
                    let value_ptr = (shmptr as *mut u8).add(pos);
                    let current_value = *value_ptr;
                    *value_ptr = current_value.wrapping_add(1);
                }

                println!("offset & curr_pos = {:?} & {:?}", by, pos);
            }
            Err(_) => {
                // println!("Not in afl, tchao");
            }
        }
    }
}
