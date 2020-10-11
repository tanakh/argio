#[argio::argio]
fn main(n: usize, x: [i64; n]) -> i64 {
    x.into_iter().sum()
}
