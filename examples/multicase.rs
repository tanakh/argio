use std::fmt::{self, Display};

struct Wrap<T>(T);

impl<T: Display> Display for Wrap<Vec<T>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (ix, r) in self.0.iter().enumerate() {
            if ix > 0 {
                write!(f, " ")?;
            }
            r.fmt(f)?;
        }
        Ok(())
    }
}

#[argio::argio(multicase, output = Wrap)]
fn main(n: usize) -> Vec<usize> {
    (0..n).map(|i| i * 2).collect()
}
