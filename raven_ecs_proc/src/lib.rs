use proc_macro::TokenStream;

use quote::{format_ident, quote};
use syn::{self, DeriveInput};

#[proc_macro_derive(Component)]
pub fn component(input: TokenStream) -> TokenStream {
    let comp_name = syn::parse::<DeriveInput>(input).unwrap().ident;
    let mod_name = format_ident!("impl_{}", comp_name);

    quote!(
        #[allow(non_snake_case)]
        mod #mod_name {
            use super::#comp_name;
            // Module `typetag` needs to be brought into scope because the `typetag::serde` macro requires it to be in
            // scope
            use ::raven_ecs::typetag;

            #[typetag::serde]
            impl ::raven_ecs::Component for #comp_name {
                fn inject(self: ::std::boxed::Box<Self>, w: &mut ::raven_ecs::World, e: ::raven_ecs::Entity) {
                    w.attach::<Self>(e, *self);
                }
            }
        }
    ).into()
}
