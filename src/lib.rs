//! A macro to convert function input and output to stdio
//!
//! This macro changes the arguments and return value of a function to take them from standard input and output.
//!
//! ```should_panic
//! # use argio::argio;
//! #[argio]
//! fn main(n: i32) -> i32 {
//!     n * 2
//! }
//! ```
//!
//! Instead of taking an integer as an argument, this function reads an integer from the standard input and outputs the result to the standard output.
//!
//! Because this macro uses [proconio](https://crates.io/crates/proconio) as a backend for input, you can put the same arguments as those that can be passed to the `input!` macro of `proconio` in the function (even if they are not the correct syntax for Rust).
//!
//! ```should_panic
//! # use argio::argio;
//! #[argio]
//! fn main(n: usize, x: [i64; n]) -> i64 {
//!     x.into_iter().sum()
//! }
//! ```
//!
//! This function takes such an input
//!
//! ```text
//! N
//! x_1 x_2 ... x_N
//! ```
//!
//! from the standard input and outputs the sum to the standard output.
//!
//! You can change the macro for the input by setting the `input` parameter. A macro takes the arguments of the function as they are.
//!
//! ```compile_fail
//! # use argio::argio;
//! macro_rules! my_input {
//!     ...
//! }
//!
//! #[argio(input = my_input)]
//! fn main(n: usize, x: [i64; n]) -> i64 {
//!     x.into_iter().sum()
//! }
//! ```
//!
//! Because the `Display` trait is used to display the return value, functions such as `Vec` which does not implement the `Display` trait cannot be compiled as it is.
//!
//! You can customize the behavior of the output by using a wrapper struct that implements the `Display` trait.
//!
//! ```should_panic
//! # use argio::argio;
//! # use std::{fmt, fmt::Display};
//! struct Wrap<T>(T);
//!
//! impl<T: Display> Display for Wrap<Vec<T>> {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         for (ix, r) in self.0.iter().enumerate() {
//!             if ix > 0 {
//!                 write!(f, " ")?;
//!             }
//!             r.fmt(f)?;
//!         }
//!         Ok(())
//!     }
//! }
//!
//! #[argio]
//! fn main(n: usize) -> Wrap<Vec<usize>> {
//!     Wrap((0..n).map(|i| i * 2).collect())
//! }
//! ```
//!
//! ```text
//! $ echo 10 | cargo run
//! 0 2 4 6 8 10 12 14 16 18
//! ```
//!
//! Of course, you can also output manually. If the return value of the function is `()`, it does not output anything to the standard output, so you can output it manually and return `()`.
//!
//! ```should_panic
//! # use argio::argio;
//! #[argio]
//! fn main(n: usize) {
//!     let ans = (0..n).map(|i| i * 2).collect::<Vec<_>>();
//!     for (i, x) in ans.into_iter().enumerate() {
//!         if i > 0 {
//!             print!(" ");
//!         }
//!         print!("{}", x);
//!     }
//!     println!();
//! }
//! ```
//!
//! You can also specify a wrapper for the output from a macro parameter. This has the advantage of removing information about the wrapper from the code, allowing you to move the output customization to the template part of the code.
//!
//! ```should_panic
//! # use argio::argio;
//! # use std::fmt::{self, Display};
//! # struct Wrap<T>(T);
//! # impl<T: Display> Display for Wrap<Vec<T>> {
//! #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//! #         for (ix, r) in self.0.iter().enumerate() {
//! #             if ix > 0 {
//! #                 write!(f, " ")?;
//! #             }
//! #             r.fmt(f)?;
//! #         }
//! #         Ok(())
//! #     }
//! # }
//! #[argio(output = Wrap)]
//! fn main(n: usize) -> Vec<usize> {
//!     (0..n).map(|i| i * 2).collect()
//! }
//! ```
//!
//! If `multicase` is specified as an attribute, it can be used to automatically execute multiple inputs for multiple cases that start with the number of cases.
//!
//! The value of the attribute `multicase` is a string to be displayed at the top of each case. The variable `i` contains the case number of 0 origin, so you can customize the display by using it.
//!
//! ```should_panic
//! # use argio::argio;
//! # use std::fmt::{self, Display};
//! # struct Wrap<T>(T);
//! # impl<T: Display> Display for Wrap<Vec<T>> {
//! #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//! #         for (ix, r) in self.0.iter().enumerate() {
//! #             if ix > 0 {
//! #                 write!(f, " ")?;
//! #             }
//! #             r.fmt(f)?;
//! #         }
//! #         Ok(())
//! #     }
//! # }
//! #[argio(multicase = "Case #{i+1}: ", output = Wrap)]
//! fn main(n: usize) -> Vec<usize> {
//!     (0..n).map(|i| i * 2).collect()
//! }
//! ```
//!
//! ```text
//! $ echo "3 2 3 5" | cargo run
//! Case #1: 0 2
//! Case #2: 0 2 4
//! Case #3: 0 2 4 6 8
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, visit_mut::VisitMut, Token};

/// A macro to convert function input and output to stdio
#[proc_macro_attribute]
pub fn argio(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as ArgioAttr);
    let item = parse_macro_input!(item as syn::ItemFn);

    let vis = item.vis;
    let name = &item.sig.ident;
    let ret_type = item.sig.output;
    let args = &item.sig.inputs;
    let body = item.block.as_ref();

    let ret_var: syn::Ident = parse_quote! { ret };
    let wrapped: syn::Expr = if let Some(wrapper) = &attr.output {
        parse_quote! { #wrapper(#ret_var) }
    } else {
        parse_quote! { #ret_var }
    };

    let unit_type: syn::Type = parse_quote! {()};

    let ret_type: syn::Type = match ret_type {
        syn::ReturnType::Default => unit_type.clone(),
        syn::ReturnType::Type(_, ty) => parse_quote! { #ty },
    };

    let print_code = if ret_type == unit_type {
        quote! {}
    } else {
        quote! {
            println!("{}", #wrapped);
        }
    };

    let input_macro: syn::Path = if let Some(path) = &attr.input {
        path.clone()
    } else {
        parse_quote! { proconio::input }
    };

    let ret = if let Some((fmt_str, fmt_span)) = &attr.multicase {
        let re = regex::Regex::new(r"^([^{]*)\{([^:}]+)(:[^}]+)?\}(.*)$").unwrap();
        let caps = if let Some(caps) = re.captures(&fmt_str) {
            caps
        } else {
            return syn::Error::new(*fmt_span, "Invalid multicase format")
                .to_compile_error()
                .into();
        };

        let fmt_str = format!(
            "{}{{{}}}{}",
            &caps[1],
            caps.get(3).map(|r| r.as_str()).unwrap_or(""),
            &caps[4]
        );

        let mut fmt_arg: syn::Expr = match syn::parse_str(&caps[2]) {
            Ok(fmt_arg) => fmt_arg,
            Err(err) => {
                return syn::Error::new(*fmt_span, format!("{}: `{}`", err, &caps[2]))
                    .to_compile_error()
                    .into();
            }
        };

        let case_id: syn::Ident = parse_quote! { case_id };

        VarRewriter {
            case_id: case_id.clone(),
        }
        .visit_expr_mut(&mut fmt_arg);

        quote! {
            #vis fn #name() {
                #input_macro ! {
                    cases: usize,
                }

                for #case_id in 0..cases {
                    print!(#fmt_str, #fmt_arg);

                    let #ret_var = (|| -> #ret_type {
                        #input_macro ! {
                            #args
                        }
                        #body
                    })();

                    #print_code
                }
            }
        }
    } else {
        quote! {
            #vis fn #name() {
                let #ret_var = (|| -> #ret_type {
                    #input_macro ! {
                        #args
                    }
                    #body
                })();

                #print_code
            }
        }
    };
    ret.into()
}

struct VarRewriter {
    case_id: syn::Ident,
}

impl syn::visit_mut::VisitMut for VarRewriter {
    fn visit_ident_mut(&mut self, i: &mut syn::Ident) {
        if i == "i" {
            *i = self.case_id.clone();
        }
    }
}

struct ArgioAttr {
    multicase: Option<(String, proc_macro2::Span)>,
    input: Option<syn::Path>,
    output: Option<syn::Path>,
}

impl syn::parse::Parse for ArgioAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut ret = ArgioAttr {
            multicase: None,
            input: None,
            output: None,
        };

        let mut first = true;

        loop {
            if first {
                first = false;
            } else {
                if !input.peek(Token![,]) {
                    break;
                }
                input.parse::<Token![,]>()?;
            };

            if !input.peek(syn::Ident) {
                break;
            }
            let var = input.parse::<syn::Ident>()?;

            if var == "multicase" {
                if input.peek(Token![=]) {
                    input.parse::<Token![=]>()?;
                    let s = input.parse::<syn::LitStr>()?;
                    ret.multicase = Some((s.value(), s.span()));
                } else {
                    ret.multicase = Some(("Case #{i+1}: ".to_string(), input.span()));
                }
            } else if var == "output" {
                input.parse::<Token![=]>()?;
                let path = input.parse::<syn::Path>()?;
                ret.output = Some(path);
            } else if var == "input" {
                input.parse::<Token![=]>()?;
                let path = input.parse::<syn::Path>()?;
                ret.input = Some(path);
            } else {
                return Err(syn::Error::new(
                    var.span(),
                    format!("argio: invalid attr: {}", var),
                ));
            }
        }

        Ok(ret)
    }
}
