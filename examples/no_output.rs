#[argio::argio]
fn main(n: usize) {
    let ans = (0..n).map(|i| i * 2).collect::<Vec<_>>();
    for (i, x) in ans.into_iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", x);
    }
    println!();
}
