use crate::world::World;
use crate::{Component, Entity, ID};

use std::cmp::min;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::cell::{Ref, RefMut};

use paste::paste;

pub trait Query<'a> {
    type Out;
    type OutMut;

    fn query_shallow(w: &'a World) -> Self::Out;
    fn query_shallow_mut(w: &'a mut World) -> Self::OutMut;

    fn query_deep(w: &'a World) -> Self::Out;
    fn query_deep_mut(w: &'a mut World) -> Self::OutMut;
}

macro_rules! count {
    ($one:ident) => { 1 };
    ($first:ident, $($rest:ident),*) => {
        1 + count!($($rest),*)
    }
}

/// Given a reference to a world and a type, evaluates to a reference to the pool of that type or else returns an empty
/// view
macro_rules! pool_or_return {
    ($world:expr, $view_type:ident, $is_deep:expr, $pool_type:ident) => {
        match $world.pool::<$pool_type>() {
            Some(pool) => pool,
            None => {
                return $view_type {
                    w: $world,
                    entities_ids: Vec::new(),

                    is_deep: $is_deep,
                    deep_state: None,

                    _marker: PhantomData,
                }
            }
        }
    };
}

macro_rules! prepare_query {
    ($world:expr, $view_type:ident, $is_deep:expr, $( $pool_type:ident ),*) => {
        $(
            let paste!{[< pool_ $pool_type:snake >]} = pool_or_return! {$world, $view_type, $is_deep, $pool_type};
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

            is_deep: $is_deep,
            deep_state: None,

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

macro_rules! next_shallow {
    ($self:expr, $get_one_x:tt, $( $pool_type:ident ),* ) => {
        loop {
            let entity_id: ID = *$self.entities_ids.first()?;
            $self.entities_ids.remove(0);

            $(
                let paste!{[< pool_ $pool_type:snake >]} = $self.w.pool::<$pool_type>().unwrap();
            )*

            $(
                let paste!{[< $pool_type:lower >]} = if let Some(c) = paste!{[< pool_ $pool_type:snake >]}.$get_one_x(entity_id) {
                    c
                } else {
                    continue;
                };
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

macro_rules! next_deep {
    ($self:expr, $get_nth_x:tt, $( $pool_type:ident ),* ) => {
        {
            $(
                let paste!{[< pool_ $pool_type:snake >]} = $self.w.pool::<$pool_type>().unwrap();
            )*

            if $self.deep_state.is_none() {
                loop {
                    let entity_id: ID = *$self.entities_ids.first()?;
                    $self.entities_ids.remove(0);

                    $self.deep_state = Some((entity_id, [(0, 0); count!($($pool_type),*)]));
                    let mut deep_state = $self.deep_state.as_mut().unwrap();

                    let mut i = 0;

                    $(
                        deep_state.1[i] = (0, paste!{[< pool_ $pool_type:snake >]}.count(entity_id));
                        i += 1;
                    )*
                }
            }

            let mut deep_state = $self.deep_state.as_mut().unwrap();
            let entity_id = deep_state.0;

            let mut i = 0;
            $(
                let paste!{[< $pool_type:lower >]} = paste!{[< pool_ $pool_type:snake >]}.$get_nth_x(entity_id, deep_state.1[i].0).unwrap();
                i += 1;
            )*

            for i in (0..count!($($pool_type),*)).rev() {
                let (at, top) = &mut deep_state.1[i];
                if *at + 1 < *top {
                    *at += 1;
                    break;
                } else {
                    if i == 0 {
                        $self.deep_state = None;
                        // Rust is not smart enough to know that this is the last iteration, give him a little help
                        break;
                    } else {
                        *at = 0;
                    }
                }
            }

            let entity = $self.w.entity_from_id(entity_id).unwrap();

            Some((entity, (
                $(
                    paste!{[< $pool_type:lower >]},
                )*
            )))
        }
    }
}

macro_rules! query_facilities {
    ( $view_type:ident, $view_mut_type:ident, $( $t:ident ),* ) => {
         pub struct $view_type<'a, $( $t: Component, )* > {
            w: &'a World,
            entities_ids: Vec<ID>,

            is_deep: bool,
            deep_state: Option<(ID, [(usize, usize); count!( $( $t ),* )])>,

            _marker: PhantomData<( $( &'a $t, )* )>,
        }

        pub struct $view_mut_type<'a, $( $t: Component, )* > {
            w: &'a World,
            entities_ids: Vec<ID>,

            is_deep: bool,
            deep_state: Option<(ID, [(usize, usize); count!( $( $t ),* )])>,

            _marker: PhantomData<( $( &'a mut $t, )* )>,
        }

        impl<'a, $( $t: Component, )* > Query<'a> for ( $( $t, )* ) {
            type Out = $view_type<'a, $( $t, )* >;
            type OutMut = $view_mut_type<'a, $( $t, )* >;

            fn query_shallow(w: &'a World) -> Self::Out {
                prepare_query!{w, $view_type, false, $( $t ),* }
            }

            fn query_shallow_mut(w: &'a mut World) -> Self::OutMut {
                let w = &*w;
                prepare_query!{w, $view_mut_type, false, $( $t ),* }
            }

            fn query_deep(w: &'a World) -> Self::Out {
                prepare_query!{w, $view_type, true, $( $t ),* }
            }

            fn query_deep_mut(w: &'a mut World) -> Self::OutMut {
                let w = &*w;
                prepare_query!{w, $view_mut_type, true, $( $t ),* }
            }
        }

        impl<'a, $( $t: Component, )* > Iterator for $view_type<'a, $( $t, )* > {
            type Item = (
                Entity,
                ( $( impl Deref<Target=$t>, )* ),
            );

            fn next(&mut self) -> Option<Self::Item> {
                if self.is_deep {
                    next_deep!(self, get_nth, $( $t ),* )
                } else {
                    next_shallow!(self, get_one, $( $t ),* )
                }
            }
        }

        impl<'a, $( $t: Component, )* > Iterator for $view_mut_type<'a, $( $t, )* > {
            type Item = (
                Entity,
                ( $( impl DerefMut<Target=$t>, )* ),
            );

            fn next(&mut self) -> Option<Self::Item> {
                if self.is_deep {
                    next_deep!(self, get_nth_mut, $( $t ),* )
                } else {
                    next_shallow!(self, get_one_mut, $( $t ),* )
                }
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
    fn query_shallow() {
        let mut w = World::default();

        let e1 = w.create();
        w.attach::<i32>(e1, 10);
        w.attach::<i32>(e1, 99);
        w.attach::<&'static str>(e1, "A");
        w.detach::<i32>(e1);

        let e2 = w.create();
        w.attach::<i32>(e1, 20);
        w.attach::<i32>(e1, 99);
        w.attach::<&'static str>(e1, "B");

        let e3 = w.create();
        w.attach::<i32>(e3, 30);
        w.attach::<i32>(e3, 99);

        let want = vec![
            (e1, (99, "A")),
            (e2, (20, "B")),
        ];

        for (i, (e, (n, s))) in <(i32, &'static str)>::query_shallow(&w).enumerate() {
            let (want_e, (want_n, want_s)) = want[i];
            assert_eq!(e, want_e);
            assert_eq!(*n, want_n);
            assert_eq!(*s, want_s);
        }
    }

    #[test]
    fn query_shallow_mut() {
        let mut w = World::default();

        let e1 = w.create();
        w.attach::<i32>(e1, 10);
        w.attach::<i32>(e1, 99);
        w.attach::<String>(e1, String::from("A"));

        let e2 = w.create();
        w.attach::<i32>(e1, 20);
        w.attach::<i32>(e1, 99);
        w.attach::<String>(e1, String::from("B"));

        let e3 = w.create();
        w.attach::<i32>(e3, 30);
        w.attach::<i32>(e3, 99);

        for(e, (mut n, mut s)) in <(i32, String)>::query_shallow_mut(&mut w) {
            *n += 1;
            *s = s.to_ascii_lowercase();
        }

        let want = vec![
            (e1, (11, String::from("a"))),
            (e2, (21, String::from("b"))),
        ];

        for (i, (e, (n, s))) in <(i32, String)>::query_shallow(&w).enumerate() {
            let (want_e, (want_n, want_s)) = want[i].clone();
            assert_eq!(e, want_e);
            assert_eq!(*n, want_n);
            assert_eq!(s.clone(), want_s);
        }
    }

    #[test]
    fn query_deep() {
        let mut w = World::default();

        let e1 = w.create();
        w.attach::<i32>(e1, 1);
        w.attach::<i32>(e1, 10);
        w.attach::<&'static str>(e1, "A");

        let e2 = w.create();
        w.attach::<i32>(e1, 2);
        w.attach::<i32>(e1, 20);
        w.attach::<&'static str>(e1, "B");
        w.attach::<&'static str>(e1, "b");

        let want = vec![
            (e1, (1, "A")),
            (e1, (10, "A")),
            (e2, (2, "B")),
            (e2, (2, "b")),
            (e2, (20, "B")),
            (e2, (20, "b")),
        ];

        for (i, (e, (n, s))) in <(i32, &'static str)>::query_deep(&w).enumerate() {
            let (want_e, (want_n, want_s)) = want[i];
            assert_eq!(e, want_e);
            assert_eq!(*n, want_n);
            assert_eq!(*s, want_s);
        }
    }

    #[test]
    fn query_deep_mut() {
        let mut w = World::default();

        let e1 = w.create();
        w.attach::<i32>(e1, 1);
        w.attach::<i32>(e1, 10);
        w.attach::<&'static str>(e1, "A");

        let e2 = w.create();
        w.attach::<i32>(e1, 2);
        w.attach::<i32>(e1, 20);
        w.attach::<&'static str>(e1, "B");
        w.attach::<&'static str>(e1, "b");

        {
            let mut view = <(i32, &'static str)>::query_deep_mut(&mut w);
            let (_, (mut n, _)) = view.nth(2).unwrap();
            *n = 99;
        }

        let want = vec![
            (e1, (1, "A")),
            (e1, (10, "A")),
            (e2, (99, "B")),
            (e2, (99, "b")),
            (e2, (20, "B")),
            (e2, (20, "b")),
        ];

        for (i, (e, (n, s))) in <(i32, &'static str)>::query_deep(&w).enumerate() {
            let (want_e, (want_n, want_s)) = want[i];
            assert_eq!(e, want_e);
            assert_eq!(*n, want_n);
            assert_eq!(*s, want_s);
        }
    }
}
