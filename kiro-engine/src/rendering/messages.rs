use crate::rendering::renderer_plan::RenderPlan;

// #[derive(Debug, Clone)]
pub enum Message {
  MoveRenderPlan(Box<RenderPlan>),
}
