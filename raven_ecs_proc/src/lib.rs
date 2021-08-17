use proc_macro::TokenStream;
use quote::quote;
use syn;
use syn::DeriveInput;

pub use typetag;

#[proc_macro_derive(Component)]
pub fn component(input: TokenStream) -> TokenStream {
    let name = syn::parse::<DeriveInput>(input).unwrap().ident;

    quote!(
        #[::raven_ecs_proc::typetag::serde]
        impl ::raven_ecs::Component for #name {
            fn inject(self: ::std::box::Box<Self>, w: &mut ::raven_ecs::world::World, e: ::raven_ecs::Entity) {
                w.attach::<Self>(e, *self);
            }
        }
    ).into()
}
