#![no_std]
#![no_main]

use core::panic::PanicInfo;

use bootloader_api::config::Mapping;
use bootloader_api::{entry_point, BootInfo};
use device::SerialPort;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static HELLO: &[u8] = b"Hello World!";

extern "C" {
    fn __kernel_start();
    fn __kernel_end();

}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let cmos = unsafe { SerialPort::new(0x3F8) };
    cmos.init();
    cmos.data.write(b'h');

    let mut size = unsafe { __kernel_end as usize - __kernel_start as usize };

    while size > 0 {
        cmos.data.write((size % 10) as u8);
        size /= 10;
    }

    // let vga_buffer = 0xb8000 as *mut u8;
    //
    // for (i, &byte) in HELLO.iter().enumerate() {
    //     unsafe {
    //         *vga_buffer.offset(i as isize * 2) = byte;
    //         *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
    //     }
    // }

    loop {}
}
const CONFIG: bootloader_api::BootloaderConfig = {
    let mut config = bootloader_api::BootloaderConfig::new_default();
    config.kernel_stack_size = 100 * 1024; // 100 KiB
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};
entry_point!(kernel_main, config = &CONFIG);

mod device {
    use core::marker::PhantomData;

    pub use x86_64::{
        instructions::port::{
            PortReadAccess as IoPortReadAccess, PortWriteAccess as IoPortWriteAccess,
            ReadOnlyAccess, ReadWriteAccess, WriteOnlyAccess,
        },
        structures::port::{PortRead, PortWrite},
    };
    pub struct IoPort<T, A> {
        port: u16,
        value_marker: PhantomData<T>,
        access_marker: PhantomData<A>,
    }

    impl<T, A> IoPort<T, A> {
        /// Create an I/O port.
        ///
        /// # Safety
        ///
        /// This function is marked unsafe as creating an I/O port is considered
        /// a privileged operation.
        pub const unsafe fn new(port: u16) -> Self {
            Self {
                port,
                value_marker: PhantomData,
                access_marker: PhantomData,
            }
        }
    }

    impl<T: PortRead, A: IoPortReadAccess> IoPort<T, A> {
        #[inline]
        pub fn read(&self) -> T {
            unsafe { PortRead::read_from_port(self.port) }
        }
    }

    impl<T: PortWrite, A: IoPortWriteAccess> IoPort<T, A> {
        #[inline]
        pub fn write(&self, value: T) {
            unsafe { PortWrite::write_to_port(self.port, value) }
        }
    }
    pub struct SerialPort {
        pub data: IoPort<u8, ReadWriteAccess>,
        pub int_en: IoPort<u8, WriteOnlyAccess>,
        pub fifo_ctrl: IoPort<u8, WriteOnlyAccess>,
        pub line_ctrl: IoPort<u8, WriteOnlyAccess>,
        pub modem_ctrl: IoPort<u8, WriteOnlyAccess>,
        pub line_status: IoPort<u8, ReadWriteAccess>,
        pub modem_status: IoPort<u8, ReadWriteAccess>,
    }

    impl SerialPort {
        /// Create a serial port.
        ///
        /// # Safety
        ///
        /// User must ensure the `port` is valid serial port.
        pub const unsafe fn new(port: u16) -> Self {
            let data = IoPort::new(port);
            let int_en = IoPort::new(port + 1);
            let fifo_ctrl = IoPort::new(port + 2);
            let line_ctrl = IoPort::new(port + 3);
            let modem_ctrl = IoPort::new(port + 4);
            let line_status = IoPort::new(port + 5);
            let modem_status = IoPort::new(port + 6);
            Self {
                data,
                int_en,
                fifo_ctrl,
                line_ctrl,
                modem_ctrl,
                line_status,
                modem_status,
            }
        }

        pub fn init(&self) {
            // Disable interrupts
            self.int_en.write(0x00);
            // Enable DLAB
            self.line_ctrl.write(0x80);
            // Set maximum speed to 38400 bps by configuring DLL and DLM
            self.data.write(0x03);
            self.int_en.write(0x00);
            // Disable DLAB and set data word length to 8 bits
            self.line_ctrl.write(0x03);
            // Enable FIFO, clear TX/RX queues and
            // set interrupt watermark at 14 bytes
            self.fifo_ctrl.write(0xC7);
            // Mark data terminal ready, signal request to send
            // and enable auxilliary output #2 (used as interrupt line for CPU)
            self.modem_ctrl.write(0x0B);
            // Enable interrupts
            self.int_en.write(0x01);
        }
    }
}
