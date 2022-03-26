use core_foundation::runloop::CFRunLoop;

pub fn main_loop() {
  CFRunLoop::run_current()
}
