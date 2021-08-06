use crate::world::World;
use crate::{Component, Entity, ID};

use std::cmp::min;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use paste::paste;

pub trait Query<'a> {
    type Out;
    type OutMut;

    fn query(w: &'a World) -> Self::Out;
    fn query_mut(w: &'a mut World) -> Self::OutMut;
}

/// Given a reference to a world and a type, evaluates to a reference to the pool of that type or else returns an empty
/// view
macro_rules! pool_or_return {
    ($world:expr, $view_type:ident, $pool_type:ident) => {
        match $world.pool::<$pool_type>() {
            Some(pool) => pool,
            None => {
                return $view_type {
                    w: $world,
                    entities_ids: Vec::new(),
                    _marker: PhantomData,
                }
            }
        }
    };
}

macro_rules! prepare_query {
    ($world:expr, $view_type:ident, $( $pool_type:ident ),*) => {
        $(
            let paste!{[< pool_ $pool_type:snake >]} = pool_or_return! {$world, $view_type, $pool_type};
        )*

        let mut min_len = usize::MAX;

        $(
            min_len = min(min_len, paste!{[< pool_ $pool_type:snake >]}.entities_ids().len());
        )*

        let entities_ids = 'entities_ids: {
            $(
                if paste!{[< pool_ $pool_type:snake >]}.entities_ids().len() == min_len {
                    break 'entities_ids paste!{[< pool_ $pool_type:snake >]}.entities_ids();
                }
            )*
            unreachable!();
        };

        $view_type {
            w: $world,
            entities_ids,
            _marker: PhantomData,
        }
    }
}

macro_rules! component_or_continue {
    ($c:expr) => {
        if let Some(c) = $c {
            c
        } else {
            continue;
        }
    };
}

macro_rules! next {
    ($self:expr, $get:tt, $( $pool_type:ident ),* ) => {
        loop {
            let entity_id: ID = *$self.entities_ids.first()?;
            $self.entities_ids.remove(0);

            $(
                let paste!{[< pool_ $pool_type:snake >]} = $self.w.pool::<$pool_type>().unwrap();
            )*

            $(
                let paste!{[< $pool_type:lower >]} = component_or_continue!(paste!{[< pool_ $pool_type:snake >]}.$get(entity_id));
            )*

            let entity = $self.w.entity_from_id(entity_id).unwrap();

            break Some((entity, (
                $(
                    paste!{[< $pool_type:lower >]},
                )*
            )));
        }
    }
}

macro_rules! query_facilities {
    ( $view_type:ident, $view_mut_type:ident, $( $t:ident ),* ) => {
         pub struct $view_type<'a, $( $t: Component, )* > {
            w: &'a World,
            entities_ids: Vec<ID>,
            _marker: PhantomData<( $( &'a $t, )* )>,
        }

        pub struct $view_mut_type<'a, $( $t: Component, )* > {
            w: &'a World,
            entities_ids: Vec<ID>,
            _marker: PhantomData<( $( &'a mut $t, )* )>,
        }

        impl<'a, $( $t: Component, )* > Query<'a> for ( $( $t, )* ) {
            type Out = $view_type<'a, $( $t, )* >;
            type OutMut = $view_mut_type<'a, $( $t, )* >;

            fn query(w: &'a World) -> Self::Out {
                prepare_query!{w, $view_type, $( $t ),* }
            }

             fn query_mut(w: &'a mut World) -> Self::OutMut {
                let w = &*w;
                prepare_query!{w, $view_mut_type, $( $t ),* }
            }
        }

        impl<'a, $( $t: Component, )* > Iterator for $view_type<'a, $( $t, )* > {
            type Item = (
                Entity,
                ( $( impl Deref<Target=$t>, )* ),
            );

            fn next(&mut self) -> Option<Self::Item> {
                next!(self, get_one, $( $t ),* )
            }
        }

        impl<'a, $( $t: Component, )* > Iterator for $view_mut_type<'a, $( $t, )* > {
            type Item = (
                Entity,
                ( $( impl DerefMut<Target=$t>, )* ),
            );

            fn next(&mut self) -> Option<Self::Item> {
                next!(self, get_one_mut, $( $t ),* )
            }
        }
    }
}

query_facilities! { View1, View1Mut, A }
query_facilities! { View2, View2Mut, A, B }
query_facilities! { View3, View3Mut, A, B, C }
query_facilities! { View4, View4Mut, A, B, C, D }
query_facilities! { View5, View5Mut, A, B, C, D, E }
query_facilities! { View6, View6Mut, A, B, C, D, E, F }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn query() {
        let mut w = World::default();

        let e1 = w.create();
        w.attach::<i32>(e1, 10);
        w.attach::<&'static str>(e1, "A");
        w.attach::<char>(e1, 'a');

        let e2 = w.create();
        w.attach::<&'static str>(e2, "B");
        w.attach::<char>(e2, 'b');

        let e3 = w.create();
        w.attach::<i32>(e3, 30);
        w.attach::<&'static str>(e3, "C");

        let vec = <(i32,)>::query_mut(&mut w).collect::<Vec<_>>();
        for (_, (mut n,)) in vec {
            *n += 1;
        }

        let vec = <(i32, &'static str)>::query(&w).collect::<Vec<_>>();

        assert_eq!(
            vec.iter()
                .map(|(entity, (n, s))| (*entity, (n.deref(), s.deref())))
                .collect::<Vec<_>>(),
            vec![(e1, (&11, &"A")), (e3, (&31, &"C")),]
        );

        let mut iters = 0;

        for (entity, (n, s, c)) in <(i32, &'static str, char)>::query(&w) {
            iters += 1;

            assert_eq!(entity, e1);
            assert_eq!(*n, 11);
            assert_eq!(*s, "A");
            assert_eq!(*c, 'a');
        }

        assert_eq!(iters, 1);
    }
}
