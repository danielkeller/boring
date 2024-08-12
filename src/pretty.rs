use std::fmt::Display;

pub struct Commas<T>(pub T);

impl<T, I> Display for Commas<T>
where
    T: IntoIterator<Item = I>,
    T: Clone,
    I: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.0.clone().into_iter();
        if let Some(next) = iter.next() {
            next.fmt(f)?;
            for rest in iter {
                write!(f, ", {rest}")?;
            }
        }
        Ok(())
    }
}
