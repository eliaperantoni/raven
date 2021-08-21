use std::cmp::min;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use paste::paste;

use crate::{Component, Entity, ID};
use crate::world::World;

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

/// I needed a way to create an n-tuple of usize given n different idents. But you must use the argument $t in some way
/// so this is the best solution I came out with
macro_rules! repeat {
    ($t:ident, $tok:tt) => { $tok };
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
            ), (
                // Because this query is shallow, the components returned are always the 0-th
                $(
                    repeat!($pool_type, 0),
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
                        deep_state.1[i] = (0, {
                            let len = paste!{[< pool_ $pool_type:snake >]}.count(entity_id);
                            if len == 0 {
                                continue;
                            }
                            len
                        });
                        #[allow(unused)]
                        i += 1;
                    )*

                    break;
                }
            }

            let deep_state = $self.deep_state.as_mut().unwrap();
            let entity_id = deep_state.0;

            let mut i = 0;
            $(
                #[allow(non_snake_case)]
                let paste!{[< $pool_type:lower __n >]} = deep_state.1[i].0;
                let paste!{[< $pool_type:lower >]} = paste!{[< pool_ $pool_type:snake >]}.$get_nth_x(entity_id, paste!{[< $pool_type:lower __n >]}).unwrap();
                #[allow(unused)]
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
            ), (
                $(
                    paste!{[< $pool_type:lower __n >]},
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
                ( $( repeat!($t, usize), )* ),
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
                ( $( repeat!($t, usize), )* ),
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
    use crate::test::*;

    use super::*;

    #[test]
    fn query_shallow() {
        let mut w = World::default();

        let e1 = w.create();
        w.attach::<CompX>(e1, CompX::new("A"));
        w.attach::<CompX>(e1, CompX::new("B"));
        w.attach::<CompY>(e1, CompY::new("C"));
        w.detach_one::<CompX>(e1);

        let e2 = w.create();
        w.attach::<CompX>(e2, CompX::new("D"));
        w.attach::<CompX>(e2, CompX::new("E"));
        w.attach::<CompY>(e2, CompY::new("F"));

        let e3 = w.create();
        w.attach::<CompX>(e3, CompX::new("G"));
        w.attach::<CompX>(e3, CompX::new("H"));

        let want = vec![
            (e1, (CompX::new("B"), CompY::new("C"))),
            (e2, (CompX::new("D"), CompY::new("F"))),
        ];

        assert_eq!(<(CompX, CompY)>::query_shallow(&w).count(), want.len());
        for (i, (e, (x, y), (x_n, y_n))) in <(CompX, CompY)>::query_shallow(&w).enumerate() {
            let (want_e, (want_x, want_y)) = want[i].clone();
            assert_eq!(e, want_e);
            assert_eq!(*x, want_x);
            assert_eq!(*y, want_y);
            assert_eq!(x_n, 0);
            assert_eq!(y_n, 0);
            assert_eq!(*w.get_nth::<CompX>(e, x_n).unwrap(), *x);
            assert_eq!(*w.get_nth::<CompY>(e, y_n).unwrap(), *y);
        }
    }

    #[test]
    fn query_shallow_mut() {
        let mut w = World::default();

        let e1 = w.create();
        w.attach::<CompX>(e1, CompX::new("A"));
        w.attach::<CompX>(e1, CompX::new("B"));
        w.attach::<CompY>(e1, CompY::new("C"));
        w.detach_one::<CompX>(e1);

        let e2 = w.create();
        w.attach::<CompX>(e2, CompX::new("D"));
        w.attach::<CompX>(e2, CompX::new("E"));
        w.attach::<CompY>(e2, CompY::new("F"));

        let e3 = w.create();
        w.attach::<CompX>(e3, CompX::new("G"));
        w.attach::<CompX>(e3, CompX::new("H"));

        for (_e, (mut x, mut y), _) in <(CompX, CompY)>::query_shallow_mut(&mut w) {
            x.f = x.f.to_ascii_lowercase();
            y.f = y.f.to_ascii_lowercase();
        }

        let want = vec![
            (e1, (CompX::new("b"), CompY::new("c"))),
            (e2, (CompX::new("d"), CompY::new("f"))),
        ];

        assert_eq!(<(CompX, CompY)>::query_shallow(&w).count(), want.len());
        for (i, (e, (x, y), (x_n, y_n))) in <(CompX, CompY)>::query_shallow(&w).enumerate() {
            let (want_e, (want_x, want_y)) = want[i].clone();
            assert_eq!(e, want_e);
            assert_eq!(*x, want_x);
            assert_eq!(*y, want_y);
            assert_eq!(x_n, 0);
            assert_eq!(y_n, 0);
            assert_eq!(*w.get_nth::<CompX>(e, x_n).unwrap(), *x);
            assert_eq!(*w.get_nth::<CompY>(e, y_n).unwrap(), *y);
        }
    }

    #[test]
    fn query_deep() {
        let mut w = World::default();

        let e1 = w.create();
        w.attach::<CompX>(e1, CompX::new("A"));
        w.attach::<CompX>(e1, CompX::new("B"));
        w.attach::<CompY>(e1, CompY::new("C"));

        let e2 = w.create();
        w.attach::<CompX>(e2, CompX::new("D"));
        w.attach::<CompX>(e2, CompX::new("E"));
        w.attach::<CompY>(e2, CompY::new("F"));
        w.attach::<CompY>(e2, CompY::new("G"));

        let want = vec![
            (e1, (CompX::new("A"), CompY::new("C")), (0, 0)),
            (e1, (CompX::new("B"), CompY::new("C")), (1, 0)),
            (e2, (CompX::new("D"), CompY::new("F")), (0, 0)),
            (e2, (CompX::new("D"), CompY::new("G")), (0, 1)),
            (e2, (CompX::new("E"), CompY::new("F")), (1, 0)),
            (e2, (CompX::new("E"), CompY::new("G")), (1, 1)),
        ];

        assert_eq!(<(CompX, CompY)>::query_deep(&w).count(), want.len());
        for (i, (e, (x, y), (x_n, y_n))) in <(CompX, CompY)>::query_deep(&w).enumerate() {
            let (want_e, (want_x, want_y), (want_x_n, want_y_n)) = want[i].clone();
            assert_eq!(e, want_e);
            assert_eq!(*x, want_x);
            assert_eq!(*y, want_y);
            assert_eq!(x_n, want_x_n);
            assert_eq!(y_n, want_y_n);
            assert_eq!(*w.get_nth::<CompX>(e, x_n).unwrap(), *x);
            assert_eq!(*w.get_nth::<CompY>(e, y_n).unwrap(), *y);
        }
    }

    #[test]
    fn query_deep_mut() {
        let mut w = World::default();

        let e1 = w.create();
        w.attach::<CompX>(e1, CompX::new("A"));
        w.attach::<CompX>(e1, CompX::new("B"));
        w.attach::<CompY>(e1, CompY::new("C"));

        let e2 = w.create();
        w.attach::<CompX>(e2, CompX::new("D"));
        w.attach::<CompX>(e2, CompX::new("E"));
        w.attach::<CompY>(e2, CompY::new("F"));
        w.attach::<CompY>(e2, CompY::new("G"));

        {
            let mut view = <(CompX, CompY)>::query_deep_mut(&mut w);
            let (_, (mut x, mut y), _) = view.nth(2).unwrap();
            // 2nd result is (D,F). Should make every D and F lowercase
            x.f = x.f.to_ascii_lowercase();
            y.f = y.f.to_ascii_lowercase();
        }

        let want = vec![
            (e1, (CompX::new("A"), CompY::new("C")), (0, 0)),
            (e1, (CompX::new("B"), CompY::new("C")), (1, 0)),
            (e2, (CompX::new("d"), CompY::new("f")), (0, 0)),
            (e2, (CompX::new("d"), CompY::new("G")), (0, 1)),
            (e2, (CompX::new("E"), CompY::new("f")), (1, 0)),
            (e2, (CompX::new("E"), CompY::new("G")), (1, 1)),
        ];

        assert_eq!(<(CompX, CompY)>::query_deep(&w).count(), want.len());
        for (i, (e, (x, y), (x_n, y_n))) in <(CompX, CompY)>::query_deep(&w).enumerate() {
            let (want_e, (want_x, want_y), (want_x_n, want_y_n)) = want[i].clone();
            assert_eq!(e, want_e);
            assert_eq!(*x, want_x);
            assert_eq!(*y, want_y);
            assert_eq!(x_n, want_x_n);
            assert_eq!(y_n, want_y_n);
            assert_eq!(*w.get_nth::<CompX>(e, x_n).unwrap(), *x);
            assert_eq!(*w.get_nth::<CompY>(e, y_n).unwrap(), *y);
        }
    }

    #[test]
    #[should_panic]
    fn elements_cannot_coexist() {
        let mut w = World::default();

        let e1 = w.create();
        w.attach::<CompX>(e1, CompX::new("A"));
        w.attach::<CompX>(e1, CompX::new("B"));
        w.attach::<CompY>(e1, CompY::new("C"));

        let e2 = w.create();
        w.attach::<CompX>(e2, CompX::new("D"));
        w.attach::<CompX>(e2, CompX::new("E"));
        w.attach::<CompY>(e2, CompY::new("F"));
        w.attach::<CompY>(e2, CompY::new("G"));

        <(CompX, CompY)>::query_deep_mut(&mut w).collect::<Vec<_>>().into_iter().for_each(drop);
    }
}
