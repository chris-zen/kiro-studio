use core_foundation::runloop::CFRunLoop;
use kiro_midi::{self as midi, drivers::DriverSpec, Filter, InputConfig, SourceMatch};

fn main() {
  let mut driver = midi::drivers::create("test").unwrap();

  let input_config1 = InputConfig::new("all-devices").with_all_sources(Filter::default());

  let input_config2 = InputConfig::new("novation-devices").with_source(
    SourceMatch::regex("Novation.*").unwrap(),
    Filter::default().with_channels(1, &[1, 2]),
  );

  driver
    .create_input(input_config1, |event| println!("all      >> {:?}", event))
    .unwrap();

  driver
    .create_input(input_config2, |event| println!("novation >> {:?}", event))
    .unwrap();

  print_endpoints(&driver);

  std::thread::spawn(move || loop {
    loop {
      let mut input_line = String::new();
      std::io::stdin()
        .read_line(&mut input_line)
        .expect("Failed to read line");

      print_endpoints(&driver);
    }
  });

  println!("=== Press Ctrl-C to stop ===");
  println!("=== Press Enter to list endpoints ===");

  // This is required in MacOS to be able to handle notifications whenever devices are plugged/unplugged
  CFRunLoop::run_current();
}

fn print_endpoints(driver: &midi::drivers::Driver) {
  println!("===================================================================================");
  println!("Sources:");
  for mut source in driver.sources() {
    let input_names = (!source.connected_inputs.is_empty())
      .then(|| {
        source.connected_inputs.sort();
        format!(" ({})", source.connected_inputs.join(", "))
      })
      .unwrap_or_default();
    println!("  [{:08x}] {} {}", source.id, source.name, input_names);
  }
  println!("Destinations:");
  for destination in driver.destinations() {
    println!("  [{:08x}] {}", destination.id, destination.name);
  }
  println!("===================================================================================");
}
