use kiro_synth::config::Config;
use kiro_synth::engine::SynthEngine;
use kiro_synth::graph::SynthGraph;

fn main() -> anyhow::Result<()> {
  let mut synth_engine = SynthEngine::new(Config::default())?;
  let sample_rate = synth_engine.sample_rate();

  let _synth_graph = SynthGraph::try_new(synth_engine.engine_mut(), sample_rate, 1)?;

  synth_engine.engine_mut().update_render_plan()?;
  synth_engine.start()?;

  loop {
    std::thread::sleep(std::time::Duration::from_secs(1));
  }

  Ok(())
}
