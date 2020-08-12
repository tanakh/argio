[![Workflow Status](https://github.com/tanakh/argio/workflows/rust/badge.svg)](https://github.com/tanakh/argio/actions?query=workflow%3A%22rust%22)

# argio

A macro to convert function input and output to stdio

This macro changes the arguments and return value of a function to take them from standard input and output.

```rust
#[argio]
fn main(n: i32) -> i32 {
    n * 2
}
```

Instead of taking an integer as an argument, this function reads an integer from the standard input and outputs the result to the standard output.

Because this macro uses [proconio](https://crates.io/crates/proconio) as a backend for input, you can put the same arguments as those that can be passed to the `input!` macro of `proconio` in the function (even if they are not the correct syntax for Rust).

```rust
#[argio]
fn main(n: usize, x: [i64; n]) -> i64 {
    x.into_iter().sum()
}
```

This function takes such an input

```
N
x_1 x_2 ... x_N
```

from the standard input and outputs the sum to the standard output.

You can change the macro for the input by setting the `input` parameter. A macro takes the arguments of the function as they are.

```rust

macro_rules! my_input {
    ...
}

#[argio(input = my_input)]
fn main(n: usize, x: [i64; n]) -> i64 {
    x.into_iter().sum()
}
```

Because the `Display` trait is used to display the return value, functions such as `Vec` which does not implement the `Display` trait cannot be compiled as it is.

You can customize the behavior of the output by using a wrapper struct that implements the `Display` trait.

```rust
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

#[argio]
fn main(n: usize) -> Wrap<Vec<usize>> {
    Wrap((0..n).map(|i| i * 2).collect())
}
```

```
$ echo 10 | cargo run
0 2 4 6 8 10 12 14 16 18
```

Of course, you can also output manually. If the return value of the function is `()`, it does not output anything to the standard output, so you can output it manually and return `()`.

```rust
#[argio]
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
```

You can also specify a wrapper for the output from a macro parameter. This has the advantage of removing information about the wrapper from the code, allowing you to move the output customization to the template part of the code.

```rust
#[argio(output = Wrap)]
fn main(n: usize) -> Vec<usize> {
    (0..n).map(|i| i * 2).collect()
}
```

If `multicase` is specified as an attribute, it can be used to automatically execute multiple inputs for multiple cases that start with the number of cases.

The value of the attribute `multicase` is a string to be displayed at the top of each case. The variable `i` contains the case number of 0 origin, so you can customize the display by using it.

```rust
#[argio(multicase = "Case #{i+1}: ", output = Wrap)]
fn main(n: usize) -> Vec<usize> {
    (0..n).map(|i| i * 2).collect()
}
```

```
$ echo "3 2 3 5" | cargo run
Case #1: 0 2
Case #2: 0 2 4
Case #3: 0 2 4 6 8
```
