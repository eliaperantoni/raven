use crate::world::World;
use crate::{Component, Entity, ID};

use std::cmp::min;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use paste::paste;

pub trait Query<'a> {
    type Out;

    fn query(w: &'a World) -> Self::Out;
}

pub trait QueryMut<'a> {
    type Out;

    fn query_mut(w: &'a mut World) -> Self::Out;
}

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
    }
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
    ( $n:tt, $( $t:ident ),* ) => {paste!{
         pub struct [< View $n >]<'a, $( $t: Component, )* > {
            w: &'a World,
            entities_ids: Vec<ID>,
            _marker: PhantomData<( $( &'a $t, )* )>,
        }

        pub struct [< View $n Mut >]<'a, $( $t: Component, )* > {
            w: &'a World,
            entities_ids: Vec<ID>,
            _marker: PhantomData<( $( &'a mut $t, )* )>,
        }

        impl<'a, $( $t: Component, )* > Query<'a> for ( $( $t, )* ) {
            type Out = [< View $n >]<'a, $( $t, )* >;

            fn query(w: &'a World) -> Self::Out {
                prepare_query!{w, [< View $n >], $( $t ),* }
            }
        }

        impl<'a, $( $t: Component, )* > QueryMut<'a> for ( $( $t, )* ) {
            type Out = [< View $n Mut >]<'a, $( $t, )* >;

            fn query_mut(w: &'a mut World) -> Self::Out {
                let w = &*w;
                prepare_query!{w, [< View $n Mut >], $( $t ),* }
            }
        }

        impl<'a, $( $t: Component, )* > Iterator for [< View $n >]<'a, $( $t, )* > {
            type Item = (
                Entity,
                ( $( impl Deref<Target=$t>, )* ),
            );

            fn next(&mut self) -> Option<Self::Item> {
                next!(self, get, $( $t ),* )
            }
        }

        impl<'a, $( $t: Component, )* > Iterator for [< View $n Mut >]<'a, $( $t, )* > {
            type Item = (
                Entity,
                ( $( impl DerefMut<Target=$t>, )* ),
            );

            fn next(&mut self) -> Option<Self::Item> {
                next!(self, get_mut, $( $t ),* )
            }
        }
    }}
}

query_facilities!{ 1, A }
query_facilities!{ 2, A, B }
query_facilities!{ 3, A, B, C }
query_facilities!{ 4, A, B, C, D }
query_facilities!{ 5, A, B, C, D, E }
query_facilities!{ 6, A, B, C, D, E, F }

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
            vec![(e1, (&11, &"A")), (e3, (&31, &"C")), ]
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
