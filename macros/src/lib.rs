use proc_macro::TokenStream as TokenStream1;
use proc_macro_error::proc_macro_error;
use quote::{format_ident, quote};
use syn::{ConstParam, Field, FnArg, GenericParam, ItemFn, ItemStruct, LifetimeParam, TypeParam};

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

#[proc_macro_error]
#[proc_macro_attribute]
pub fn wiring(_: TokenStream1, input: TokenStream1) -> TokenStream1 {
  let input = match syn::parse::<ItemStruct>(input) {
    Ok(i) => i,
    Err(e) => return TokenStream1::from(e.to_compile_error()),
  };
  let attrs = &input.attrs;
  let vis = &input.vis;
  let generics = input.generics.params.iter().collect::<Vec<_>>();
  let generics_pass = input
    .generics
    .params
    .iter()
    .map(|x| match x {
      GenericParam::Lifetime(LifetimeParam { lifetime, .. }) => quote!(#lifetime),
      GenericParam::Type(TypeParam { ident, .. }) => quote!(#ident),
      GenericParam::Const(ConstParam { ident, .. }) => quote!(#ident),
    })
    .collect::<Vec<_>>();
  let where_clause = &input.generics.where_clause;
  let name = &input.ident;
  let (fields, fields_macro) = match &input.fields {
    syn::Fields::Named(fields) => {
      let fields_macro = fields
        .named
        .iter()
        .enumerate()
        .map(|(i, Field { ident, ty, .. })| {
          let i = syn::Index::from(i);
          quote!(#i #ident: #ty)
        });
      let fields = fields
        .named
        .iter()
        .map(|Field { vis, ident, ty, .. }| quote!(#vis #ident: <#ty as Bundle>::Of<X>));
      (quote!({ #(#fields),* }), quote!({ #(#fields_macro),* }))
    }
    syn::Fields::Unnamed(fields) => {
      let fields_macro = fields
        .unnamed
        .iter()
        .enumerate()
        .map(|(i, Field { ty, .. })| {
          let i = syn::Index::from(i);
          quote!(#i #ty)
        });
      let fields = fields
        .unnamed
        .iter()
        .map(|Field { vis, ty, .. }| quote!(#vis <#ty as Bundle>::Of<X>));
      (quote!((#(#fields),*)), quote!((#(#fields_macro),*)))
    }
    syn::Fields::Unit => (quote!(), quote!()),
  };
  let semi = &input.semi_token;
  quote!(
    #(#attrs)*
    #vis struct #name<#(#generics,)* X = Wire> #fields #semi

    impl<#(#generics),*> Clone for #name<#(#generics_pass),*> {
      fn clone(&self) -> Self {
        *self
      }
    }
    impl<#(#generics),*> Copy for #name<#(#generics_pass),*> {}

    impl_bundle!(
      (#(#generics),*),
      #name<#(#generics_pass),*>,
      (#where_clause),
      #name<#(#generics_pass,)* X>,
      #name #fields_macro
    );
  )
  .into()
}
