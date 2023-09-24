use proc_macro::TokenStream as TokenStream1;
use proc_macro_error::proc_macro_error;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn};

#[proc_macro_error]
#[proc_macro_attribute]
pub fn component(_: TokenStream1, input: TokenStream1) -> TokenStream1 {
  let input = match syn::parse::<ItemFn>(input) {
    Ok(i) => i,
    Err(e) => return TokenStream1::from(e.to_compile_error()),
  };
  let attrs = &input.attrs;
  let vis = &input.vis;
  let sig = &input.sig;
  let block = &input.block;
  let generics = &sig.generics.params;
  let name = &sig.ident;
  let inner_name = format_ident!("_{}", name);
  let inputs = &sig.inputs;
  let input_pats = sig
    .inputs
    .iter()
    .filter_map(|x| match x {
      FnArg::Receiver(_) => None,
      FnArg::Typed(x) => Some(&x.pat),
    })
    .collect::<Vec<_>>();
  quote!(
    impl Circuit {
      #(#attrs)*
      #vis #sig {
        self.component((#(#input_pats,)*), |circuit, (#(#input_pats,)*)| {
          circuit.#inner_name(#(#input_pats),*);
        });
      }
      fn #inner_name<#generics>(#inputs) #block
    }
  )
  .into()
}
