pub use self::Spell::*;
pub use self::Result::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spell {
    MagicMissile,
    Drain,
    Shield,
    Poison,
    Recharge,
}
pub struct AllSpells {
    ctr: u8,
}
impl Iterator for AllSpells {
    type Item = Spell;
    fn next(&mut self) -> Option<Spell> {
        let i = self.ctr;
        self.ctr += 1;
        match i {
            0 => Some(MagicMissile),
            1 => Some(Drain),
            2 => Some(Shield),
            3 => Some(Poison),
            4 => Some(Recharge),
            5 => None,
            _ => panic!("Iterator called again after returning None")
        }
    }
}
impl Spell {
    pub fn all() -> AllSpells { AllSpells { ctr: 0 } }
    pub fn cost(self) -> u16 {
        match self {
            MagicMissile => 53,
            Drain => 73,
            Shield => 113,
            Poison => 173,
            Recharge => 229
        }
    }
}

const MM_DMG: Damage = Damage::Magic(4);
const DR_DMG: Damage = Damage::Magic(2);
const DR_HEA: u16 = 2;
const SH_LEN: u8 = 6;
const SH_ARM: u16 = 7;
const PO_LEN: u8 = 6;
const PO_DMG: Damage = Damage::Magic(3);
const RE_LEN: u8 = 5;
const RE_POW: u16 = 101;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Win {
    pub spent: u32
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Result {
    Ok(State),
    // This isn't a `result::Result` mainly because `Err(Won(...))` looks silly.
    // (Also, treating `Won` and `Lost` the same way is usually not called for.)
    Won(Win),
    // Could distinguish between HP<=0 and invalid spell if needed.
    Lost,
}
impl Result {
    pub fn chain<F>(self, f: F) -> Result where F: FnOnce(State) -> Result {
        match self {
            Ok(state) => f(state),
            Won(w) => Won(w),
            Lost => Lost,
        }
    }
    #[allow(dead_code)]
    pub fn unwrap(self) -> State {
        let verb = match self {
            Ok(state) => return state,
            Won(_) => "won",
            Lost => "lost",
        };
        panic!("The player {}, but we expected the game to continue!", verb);
    }
    #[allow(dead_code)]
    pub fn unwrap_win(self) -> Win {
        let verb = match self {
            Won(w) => return w,
            Lost => "lost",
            Ok(_) => "is still fighting",
        };
        panic!("The player {}, but we expected a win!", verb);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Damage {
    Physical(u16),
    Magic(u16),
}
impl Damage {
    fn apply(self, armor: u16) -> u16 {
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

#[derive(Debug, Clone)]
pub struct World {
    boss_damage: Damage
}
impl World {
    pub fn new(boss_damage: u16) -> World {
        World { boss_damage: Damage::Physical(boss_damage) }
    }
}

// This is small enough to reasonably be `Copy`, but it's too easy to
// accidentally use the wrong state if it's not affine by default.
// Also this could be bit-packed into less space but probably not worth it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    spent: u32,
    mana: u16,
    player_hp: u16,
    boss_hp: u16,
    timers: [u8; 3],
}
const TIDX_SH: usize = 0;
const TIDX_PO: usize = 1;
const TIDX_RE: usize = 2;

impl State {
    // FIXME: use a macro for the boilerplate accessors?
    // Also they'll probably need `#[allow(dead_code)]`.
    pub fn spent(&self) -> u32 { self.spent }
    pub fn mana(&self) -> u16 { self.mana }
    pub fn player_hp(&self) -> u16 { self.player_hp }
    pub fn player_armor(&self) -> u16 { if self.shield_timer() > 0 { SH_ARM } else { 0 } }
    pub fn boss_hp(&self) -> u16 { self.boss_hp }
    pub fn boss_armor(&self) -> u16 { 0 }
    pub fn shield_timer(&self) -> u8 { self.timers[TIDX_SH] }
    pub fn poison_timer(&self) -> u8 { self.timers[TIDX_PO] }
    pub fn recharge_timer(&self) -> u8 { self.timers[TIDX_RE] }

    pub fn new(player_hp: u16, player_mana: u16, boss_hp: u16) -> State {
        // Yes, I could store the HPs biased by -1, but also... I could not.
        assert!(player_hp != 0);
        assert!(boss_hp != 0);
        State {
            spent: 0,
            mana: player_mana,
            player_hp: player_hp,
            boss_hp: boss_hp,
            timers: [0, 0, 0],
        }
    }

    fn recharge(self, gain: u16) -> Result {
        Ok(State {
            mana: self.mana.checked_add(gain).expect("mana pool overflow"),
            ..self
        })
    }
    fn inflict(self, dmg: Damage) -> Result {
        assert!(dmg.is_magic());
        match self.boss_hp.checked_sub(dmg.apply(self.boss_armor())) {
            None | Some(0) => Won(Win { spent: self.spent }),
            Some(boss_hp) => Ok(State {
                boss_hp: boss_hp,
                ..self
            })
        }
    }
    fn suffer(self, dmg: Damage) -> Result {
        match self.player_hp.checked_sub(dmg.apply(self.player_armor())) {
            None | Some(0) => Lost,
            Some(player_hp) => Ok(State {
                player_hp: player_hp,
                ..self
            })
        }
    }
    fn heal(self, boon: u16) -> Result {
        Ok(State {
            player_hp: self.player_hp.checked_add(boon).expect("HP overflow"),
            ..self
        })
    }

    fn start_effect(mut self, idx: usize, turns: u8) -> Result {
        if self.timers[idx] == 0 {
            self.timers[idx] = turns;
            Ok(self)
        } else {
            Lost
        }
    }

    fn spend(self, cost: u16) -> Result {
        match self.mana.checked_sub(cost) {
            None => Lost,
            Some(mana) => Ok(State {
                spent: self.spent.checked_add(cost as u32).expect("spent mana counter overflow"),
                mana: mana,
                ..self
            })
        }
    }
    fn resolve(self, sp: Spell) -> Result {
        match sp {
            MagicMissile => self.inflict(MM_DMG),
            Drain => self.inflict(DR_DMG).chain(|nself| nself.heal(DR_HEA)),
            Shield => self.start_effect(TIDX_SH, SH_LEN),
            Poison => self.start_effect(TIDX_PO, PO_LEN),
            Recharge => self.start_effect(TIDX_RE, RE_LEN),
        }
    }
    fn cast(self, sp: Spell) -> Result {
        self.spend(sp.cost()).chain(|nself| nself.resolve(sp))
    }

    fn effect<F>(mut self, idx: usize, f: F) -> Result where F: FnOnce(State) -> Result {
        if let Some(left) = self.timers[idx].checked_sub(1) {
            self.timers[idx] = left;
            f(self)
        } else {
            Ok(self)
        }
    }
    fn upkeep(self) -> Result {
        Ok(self)
            .chain(|nself| nself.effect(TIDX_SH, |nself| Ok(nself)))
            .chain(|nself| nself.effect(TIDX_PO, |nself| nself.inflict(PO_DMG)))
            .chain(|nself| nself.effect(TIDX_RE, |nself| nself.recharge(RE_POW)))
    }

    fn boss_turn(self, w: &World) -> Result {
        self.suffer(w.boss_damage)
    }

    // TODO: some kind of event/visitor thing to enable the text UI shown. Because why not.
    pub fn round(self, w: &World, sp: Spell) -> Result {
        Ok(self)
            .chain(|nself| nself.upkeep())
            .chain(|nself| nself.cast(sp))
            .chain(|nself| nself.upkeep())
            .chain(|nself| nself.boss_turn(w))
    }
}


#[cfg(test)]
mod tests {
    use super::{Damage,Spell,State,World,Result,Win};
    use super::{MagicMissile,Drain,Shield,Poison,Recharge};

    #[test]
    fn armor() {
        assert_eq!(Damage::Physical(8).apply(3), 5);
        assert_eq!(Damage::Physical(8).apply(300), 1);
        assert_eq!(Damage::Physical(8).apply(0), 8);
        assert_eq!(Damage::Magic(8).apply(3), 8);
    }

    macro_rules! check {
        ($state:expr, { $($meth:ident: $val:expr),* }) => {
            $(assert_eq!($state.$meth(), $val);)*
        }
    }

    #[test]
    fn example1() {
        let start = State::new(10, 250, 13);
        let world = World::new(8);
        let after1 = start.round(&world, Poison).unwrap();
        let exp_spent = Poison.cost() as u32;
        check!(after1, { player_hp: 2, player_armor: 0, mana: 77,
                         boss_hp: 10, poison_timer: 5,
                         spent: exp_spent });
        let exp_spent = exp_spent + MagicMissile.cost() as u32;
        assert_eq!(after1.clone().round(&world, MagicMissile),
                   Result::Won(Win { spent: exp_spent }));
        // in more detail:
        let a1x = after1.upkeep().unwrap();
        check!(a1x, { player_hp: 2, player_armor: 0, mana: 77,
                      boss_hp: 7, poison_timer: 4,
                      spent: Poison.cost() as u32 });
        let a1y = a1x.cast(MagicMissile).unwrap();
        check!(a1y, { player_hp: 2, player_armor: 0, mana: 24,
                      boss_hp: 3, poison_timer: 4,
                      spent: exp_spent });
        assert_eq!(a1y.upkeep(), Result::Won(Win { spent: exp_spent }));
    }

    #[test]
    fn example2() {
        let exp_spent = [Recharge, Shield, Drain, Poison, MagicMissile].iter()
            .map(|sp| sp.cost() as u32)
            .fold(0, |a, b| a + b);

        let start = State::new(10, 250, 14);
        let world = World::new(8);
        let after1 = start.round(&world, Recharge).unwrap();

        check!(after1, { player_hp: 2, player_armor: 0, mana: 122,
                         boss_hp: 14, recharge_timer: 4 });
        let after2 = after1.round(&world, Shield).unwrap();

        check!(after2, { player_hp: 1, player_armor: 7, mana: 211,
                         boss_hp: 14, recharge_timer: 2, shield_timer: 5 });
        let after3 = after2.round(&world, Drain).unwrap();

        check!(after3, { player_hp: 2, player_armor: 7, mana: 340,
                         boss_hp: 12, recharge_timer: 0, shield_timer: 3 });
        let after4 = after3.round(&world, Poison).unwrap();

        check!(after4, { player_hp: 1, player_armor: 7, mana: 167,
                         boss_hp: 9, shield_timer: 1, poison_timer: 5 });

        assert_eq!(after4.round(&world, MagicMissile).unwrap_win().spent, exp_spent);
    }
}
