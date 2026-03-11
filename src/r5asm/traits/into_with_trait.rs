pub trait IntoWith<Rhs, Output> {
    fn into_with(self, rhs: Rhs) -> Output;
}