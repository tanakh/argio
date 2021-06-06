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
        parse_quote! { argio::proconio::input }
    };

    let ret = if let Some((fmt_str, fmt_span)) = &attr.multicase {
        let (case_id, print_header) = if !fmt_str.contains('{') {
            (
                parse_quote! { case_id },
                quote! {
                    print!(#fmt_str);
                },
            )
        } else {
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

            (
                case_id,
                quote! {
                    print!(#fmt_str, #fmt_arg);
                },
            )
        };

        quote! {
            #vis fn #name() {
                #input_macro ! {
                    cases: usize,
                }

                for #case_id in 0..cases {
                    #print_header

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
