extern crate util;

use std::convert::AsRef;
use std::iter;
use std::iter::FromIterator;
use std::ops::Deref;
use util::best::{Best,Smallest};

type Gold = u16;
type HP = u16;

trait Effect {
    fn damage(&self) -> HP { 0 }
    fn defense(&self) -> HP { 0 }
}

#[derive(Debug, PartialEq, Eq)]
struct Item<E: Effect> {
    name: &'static str,
    cost: Gold,
    effect: E,
}
// I'm pretty sure `&for<T: TraitBound> Foo<T>` isn't a thing.  (Yet.)
trait GenItem {
    fn name(&self) -> &'static str;
    fn cost(&self) -> Gold;
    fn effect(&self) -> &Effect;
}
impl<E: Effect> GenItem for Item<E> {
    fn name(&self) -> &'static str { self.name }
    fn cost(&self) -> Gold { self.cost }
    fn effect(&self) -> &Effect { &self.effect }
}

#[derive(Debug, PartialEq, Eq)]
struct Weapon { dmg: HP }
impl Effect for Weapon { fn damage(&self) -> HP { self.dmg } }

#[derive(Debug, PartialEq, Eq)]
struct Armor { ac: HP }
impl Effect for Armor { fn defense(&self) -> HP { self.ac } }

#[derive(Debug, PartialEq, Eq)]
enum Ring {
    PlusDmg(HP),
    PlusDef(HP),
}
impl Effect for Ring {
    fn damage(&self) -> HP { match *self {
        Ring::PlusDmg(plus) => plus,
        Ring::PlusDef(_) => 0,
    }}
    fn defense(&self) -> HP { match *self {
        Ring::PlusDmg(_) => 0,
        Ring::PlusDef(plus) => plus,
    }}
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Loadout<'a> {
    weapon: &'a Item<Weapon>,
    armor: Option<&'a Item<Armor>>,
    rings: UpTo2<&'a Item<Ring>>,
}
impl<'a> Loadout<'a> {
    fn list<F, R>(&self, f: F) -> R
        where F: FnOnce(&mut Iterator<Item=&'a GenItem>) -> R {
        f(&mut
          iter::once(self.weapon as &GenItem)
          .chain(self.armor.iter().map(|&r| r as &GenItem))
          .chain(self.rings.iter().map(|&r| r as &GenItem)))
    }
}

#[derive(Debug, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum UpTo2<T> {
    Zero([T; 0]),
    One([T; 1]),
    Two([T; 2]),
}
impl<T> UpTo2<T> {
    fn zero() -> Self { UpTo2::Zero([]) }
    fn one(x: T) -> Self { UpTo2::One([x]) }
    fn two(x: T, y: T) -> Self { UpTo2::Two([x, y]) }
}
impl<T> Deref for UpTo2<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        match *self {
            UpTo2::Zero(ref xs) => xs as &[T],
            UpTo2::One(ref xs) => xs as &[T],
            UpTo2::Two(ref xs) => xs as &[T],
        }
    }
}
impl<T> FromIterator<T> for UpTo2<T> {
    fn from_iter<I: IntoIterator<Item=T>>(i: I) -> Self {
        let mut i = i.into_iter();
        match i.next() {
            None => Self::zero(),
            Some(x) => match i.next() {
                None => Self::one(x),
                Some(y) => match i.next() {
                    None => Self::two(x, y),
                    Some(_) => panic!("Can't construct UpTo2 from more than 2 elements.")
                }
            }
        }
    }
}
impl<T: Clone> Clone for UpTo2<T> {
    fn clone(&self) -> Self {
        Self::from_iter(self.iter().cloned())
    }
}

// Transplanted from day22.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Damage {
    Physical(HP),
    Magic(HP),
}
impl Damage {
    fn apply(self, armor: HP) -> HP {
        match self {
            Damage::Magic(dmg) => dmg,
            Damage::Physical(dmg) =>
                // 0 damage isn't a thing in the official rules, but might as well handle it.
                dmg.checked_sub(1).map_or(0, |dmg_m1| dmg_m1.saturating_sub(armor) + 1)
        }
    }
    #[allow(dead_code)]
    fn is_magic(self) -> bool {
        match self {
            Damage::Magic(_) => true,
            Damage::Physical(_) => false,
        }
    }
}


static WEAPONS: [Item<Weapon>; 5] = [
    Item { name: "Dagger",     cost: 8,  effect: Weapon { dmg: 4 } },
    Item { name: "Shortsword", cost: 10, effect: Weapon { dmg: 5 } },
    Item { name: "Dagger",     cost: 25, effect: Weapon { dmg: 6 } },
    Item { name: "Dagger",     cost: 40, effect: Weapon { dmg: 7 } },
    Item { name: "Dagger",     cost: 75, effect: Weapon { dmg: 8 } }];

static ARMOR: [Item<Armor>; 5] = [
    Item { name: "Leather",    cost: 13,  effect: Armor { ac: 1 } },
    Item { name: "Chainmail",  cost: 31,  effect: Armor { ac: 2 } },
    Item { name: "Splintmail", cost: 53,  effect: Armor { ac: 3 } },
    Item { name: "Bandedmail", cost: 75,  effect: Armor { ac: 4 } },
    Item { name: "Platemail",  cost: 102, effect: Armor { ac: 5 } }];

static RINGS: [Item<Ring>; 6] = [
    Item { name: "Damage +1",  cost: 25,  effect: Ring::PlusDmg(1) },
    Item { name: "Damage +1",  cost: 50,  effect: Ring::PlusDmg(2) },
    Item { name: "Damage +1",  cost: 100, effect: Ring::PlusDmg(3) },
    Item { name: "Defense +1",  cost: 20, effect: Ring::PlusDef(1) },
    Item { name: "Defense +2",  cost: 40, effect: Ring::PlusDef(2) },
    Item { name: "Defense +3",  cost: 80, effect: Ring::PlusDef(3) }];

fn all_loadouts<C>(mut co_iter: C) where C: FnMut(&Loadout<'static>) {
    for weapon in WEAPONS.iter() {
        let mut lo = Loadout { weapon: weapon, armor: None, rings: UpTo2::zero() };
        co_iter(&lo);
        for armor in ARMOR.iter() {
            lo.armor = Some(armor);
            co_iter(&lo);
            for (i, ring0) in RINGS.iter().enumerate() {
                lo.rings = UpTo2::one(ring0);
                co_iter(&lo);
                for ring1 in RINGS[..i].iter() {
                    lo.rings = UpTo2::two(ring0, ring1);
                    co_iter(&lo);
                }
            }
        }
    }
}

struct Scenario {
    boss_hp: HP,
    boss_armor: HP,
    boss_dmg: HP,
    player_hp: HP
}
impl Scenario {
    fn can_win(&self, player_armor: HP, player_dmg: HP) -> bool {
        let suffered = Damage::Physical(self.boss_dmg).apply(player_armor);
        let inflicted = Damage::Physical(player_dmg).apply(self.boss_armor);
        let ttl = (self.player_hp + suffered - 1) / suffered;
        inflicted * ttl >= self.boss_hp
    }
}

fn solve(s: &Scenario) -> Option<(Gold, Loadout)> {
    let mut b = Best::new(Smallest);
    all_loadouts(|loadout| {
        let (def, dmg) = loadout.list(|stuff| {
            stuff.map(|thing| thing.effect())
                .fold((0 as HP, 0 as HP), |(dmg, def), e| {
                    (def + e.defense(), dmg + e.damage())
                })
        });
        if s.can_win(def, dmg) {
            let cost = loadout.list(|stuff| stuff.fold(0 as HP, |a, thing| a + thing.cost()));
            b.add(cost, loadout);
        }
    });
    b.finish()
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::{solve, Scenario, Loadout};

    #[test]
    fn example() {
        let s = Scenario {
            boss_hp: 12,
            boss_armor: 2,
            boss_dmg: 7,
            player_hp: 8,
        };
        assert!(s.can_win(5, 5));
        // Not the example, but close:
        assert!(!s.can_win(4, 5));
        assert!(!s.can_win(5, 4));
    }
}
