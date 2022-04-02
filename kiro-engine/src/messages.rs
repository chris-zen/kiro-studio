use crate::callback::plan::RenderPlan;

// #[derive(Debug, Clone)]
pub enum Message {
  MoveRenderPlan(Box<RenderPlan>),
}
