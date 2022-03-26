mod external {
  #[link(name = "CoreAudio", kind = "framework")]
  extern "C" {
    pub fn AudioGetCurrentHostTime() -> u64;
    pub fn AudioConvertNanosToHostTime(inNanos: u64) -> u64;
    pub fn AudioConvertHostTimeToNanos(inHostTime: u64) -> u64;
  }
}

pub fn coremidi_timestamp_to_nanos(timestamp: u64) -> u64 {
  unsafe { external::AudioConvertHostTimeToNanos(timestamp) }
}
