pub trait Storable: Sized {
    fn name() -> String;
    fn id(&self) -> String;
    fn key(&self) -> String;
}