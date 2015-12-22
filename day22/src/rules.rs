
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
    fn all() -> AllSpells { AllSpells { ctr: 0 } }
    fn cost(self) -> u16 {
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
pub enum Result {
    Ok(State),
    // This isn't a `result::Result` mainly because `Err(Won)` looks silly.
    // (Also, treating `Won` and `Lost` the same way is usually not called for.)
    Won,
    // Could distinguish between HP<=0 and invalid spell if needed.
    Lost,
}
impl Result {
    pub fn chain<F>(self, f: F) -> Result where F: FnOnce(State) -> Result {
        match self {
            Ok(state) => f(state),
            Won => Won,
            Lost => Lost,
        }
    }
}

#[derive(Debug, Clone)]
pub struct World {
    boss_damage: Damage
}
impl World {
    fn new(boss_damage: u16) -> World {
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

    pub fn new(player_hp: u16, boss_hp: u16) -> State {
        // Yes, I could store the HPs biased by -1, but also... I could not.
        assert!(player_hp != 0);
        assert!(boss_hp != 0);
        State {
            spent: 0,
            mana: 0,
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
            None | Some(0) => Won,
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

    pub fn round(self, w: &World, sp: Spell) -> Result {
        Ok(self)
            .chain(|nself| nself.upkeep())
            .chain(|nself| nself.cast(sp))
            .chain(|nself| nself.upkeep())
            .chain(|nself| nself.boss_turn(w))
    }
}
