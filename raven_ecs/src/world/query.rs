use crate::{ID, Component, Entity};
use crate::world::World;

use std::marker::PhantomData;
use std::cmp::min;
use std::ops::{Deref, DerefMut};

trait Query<'a> {
    type Out;

    fn query(w: &'a World) -> Self::Out;
}

trait QueryMut<'a> {
    type Out;

    fn query_mut(w: &'a World) -> Self::Out;
}

impl<'a, T: Component, U: Component> Query<'a> for (T, U) {
    type Out = View2<'a, T, U>;

    fn query(w: &'a World) -> Self::Out {
        View2::new(w)
    }
}

impl<'a, T: Component, U: Component> QueryMut<'a> for (T, U) {
    type Out = View2Mut<'a, T, U>;

    fn query_mut(w: &'a World) -> Self::Out {
        View2Mut::new(w)
    }
}

struct View2<'a, T: Component, U: Component> {
    w: &'a World,
    entities_ids: Vec<ID>,
    _marker: PhantomData<(&'a T, &'a U)>,
}

struct View2Mut<'a, T: Component, U: Component> {
    w: &'a World,
    entities_ids: Vec<ID>,
    _marker: PhantomData<(&'a mut T, &'a mut U)>,
}

impl<'a, T: Component, U: Component> View2<'a, T, U> {
    fn new(w: &'a World) -> View2<'a, T, U> {
        macro_rules! pool_or_return {
            ($w:expr, $t:ty) => {
                match $w.pool::<$t>() {
                    Some(pool) => pool,
                    None => return View2 {
                        w: $w,
                        entities_ids: Vec::new(),
                        _marker: PhantomData,
                    },
                }
            }
        }

        let pool_t = pool_or_return! {w, T};
        let pool_u = pool_or_return! {w, U};

        let mut min_len = usize::MAX;

        min_len = min(min_len, pool_t.entities_ids().len());
        min_len = min(min_len, pool_u.entities_ids().len());

        let entities_ids = 'entities_ids: {
            if pool_t.entities_ids().len() == min_len {
                break 'entities_ids pool_t.entities_ids();
            }
            if pool_u.entities_ids().len() == min_len {
                break 'entities_ids pool_u.entities_ids();
            }
            unreachable!();
        };

        View2 {
            w,
            entities_ids,
            _marker: PhantomData,
        }
    }
}

impl<'a, T: Component, U: Component> View2Mut<'a, T, U> {
    fn new(w: &'a World) -> View2Mut<'a, T, U> {
        macro_rules! pool_or_return {
            ($w:expr, $t:ty) => {
                match $w.pool::<$t>() {
                    Some(pool) => pool,
                    None => return View2Mut {
                        w: $w,
                        entities_ids: Vec::new(),
                        _marker: PhantomData,
                    },
                }
            }
        }

        let pool_t = pool_or_return! {w, T};
        let pool_u = pool_or_return! {w, U};

        let mut min_len = usize::MAX;

        min_len = min(min_len, pool_t.entities_ids().len());
        min_len = min(min_len, pool_u.entities_ids().len());

        let entities_ids = 'entities_ids: {
            if pool_t.entities_ids().len() == min_len {
                break 'entities_ids pool_t.entities_ids();
            }
            if pool_u.entities_ids().len() == min_len {
                break 'entities_ids pool_u.entities_ids();
            }
            unreachable!();
        };

        View2Mut {
            w,
            entities_ids,
            _marker: PhantomData,
        }
    }
}

impl<'a, T: Component, U: Component> Iterator for View2<'a, T, U> {
    type Item = (Entity, (impl Deref<Target=T>, impl Deref<Target=U>));

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let entity_id: ID = *self.entities_ids.first()?;
            self.entities_ids.remove(0);

            let pool_t = self.w.pool::<T>().unwrap();
            let pool_u = self.w.pool::<U>().unwrap();

            let t = if let Some(t) = pool_t.get(entity_id) { t } else { continue; };
            let u = if let Some(u) = pool_u.get(entity_id) { u } else { continue; };

            let entity = self.w.entity_from_id(entity_id).unwrap();

            break Some((entity, (t, u)));
        }
    }
}

impl<'a, T: Component, U: Component> Iterator for View2Mut<'a, T, U> {
    type Item = (Entity, (impl DerefMut<Target=T>, impl DerefMut<Target=U>));

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let entity_id: ID = *self.entities_ids.first()?;
            self.entities_ids.remove(0);

            let pool_t = self.w.pool::<T>().unwrap();
            let pool_u = self.w.pool::<U>().unwrap();

            let t = if let Some(t) = pool_t.get_mut(entity_id) { t } else { continue; };
            let u = if let Some(u) = pool_u.get_mut(entity_id) { u } else { continue; };

            let entity = self.w.entity_from_id(entity_id).unwrap();

            break Some((entity, (t, u)));
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn query() {
        let mut w = World::new();

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

        {
            let vec = <(i32, &'static str)>::query_mut(&w).collect::<Vec<_>>();
            for (_, (mut n, _)) in vec {
                *n += 1;
            }
        }

        let vec = <(i32, &'static str)>::query(&w).collect::<Vec<_>>();

        assert_eq!(
            vec
                .iter()
                .map(|(entity, (n, s))| (*entity, (n.deref(), s.deref())))
                .collect::<Vec<_>>(),
            vec![
                (e1, (&11, &"A")),
                (e3, (&31, &"C")),
            ]
        );
    }
}
