pub trait Observed
{
    fn stale(&self) -> bool;
    fn refresh(&mut self);
}