use core_foundation::runloop::CFRunLoop;
use kiro_midi::{self as midi, drivers::DriverSpec};

fn main() {
  let mut driver = midi::drivers::create("test").unwrap();

  let input_config1 = midi::InputConfig::new("novation").with_source(
    midi::SourceMatch::regex("Novation SL MkIII.*").unwrap(),
    midi::Filter::default(),
  );

  let input_config2 = midi::InputConfig::new("arturia").with_source(
    midi::SourceMatch::regex("Midi.*").unwrap(),
    midi::Filter::default().with_channels(1, &[1, 2]),
  );

  driver
    .create_input(input_config1, |event| println!(">> {:?}", event))
    .unwrap();

  driver
    .create_input(input_config2, |event| println!(">> {:?}", event))
    .unwrap();

  print_endpoints(&driver);

  let _join = std::thread::spawn(move || loop {
    loop {
      let mut input_line = String::new();
      std::io::stdin()
        .read_line(&mut input_line)
        .expect("Failed to read line");

      print_endpoints(&driver);

      if let Some(mut arturia_config) = driver.get_input_config("arturia") {
        arturia_config.sources.add_source(
          midi::SourceMatch::regex("Midi.*").unwrap(),
          midi::Filter::default().with_channels(1, &[1]),
        );

        driver
          .set_input_sources(
            "arturia",
            midi::SourceMatches::default().with_source(
              midi::SourceMatch::regex("IAC.*").unwrap(),
              midi::Filter::default().with_channels(1, &[1]),
            ),
          )
          .ok();
      }
    }
  });

  println!("=== Press Ctrl-C to stop ===");
  println!("=== Press Enter to list endpoints ===");
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
