extern crate util;
mod rules;

use std::cell::Cell;
use std::env;
use util::best;
use rules::{World,State,Spell,Won,Lost,Ok,Win};

fn parse_args<I>(mut i: I) -> (World, State)
    where I: Iterator<Item=String> {
    let mut player_hp = 50u16;
    let mut player_mana = 500u16;
    let mut boss_hp = None::<u16>;
    let mut boss_dmg = None::<u16>;
    while let Some(word) = i.next() {
        let value = i.next().expect("expected key/value pairs as arguments but got odd number");
        let word = word.to_lowercase();
        let is_player = word.contains("player");
        let is_hp = word.contains("hp") || (word.contains("hit") && word.contains("point"));
        let is_mana = word.contains("mana");
        let is_boss = word.contains("boss");
        let is_dmg = word.contains("damage") || word.contains("dmg");
        if is_player && is_hp {
            player_hp = value.parse().unwrap();
        } else if is_mana {
            player_mana = value.parse().unwrap();
        } else if is_boss && is_hp {
            boss_hp = Some(value.parse().unwrap())
        } else if is_boss && is_dmg {
            boss_dmg = Some(value.parse().unwrap())
        } else {
            panic!("unrecognized key word {}", word);
        }
    }
    let boss_hp = boss_hp.expect("boss HP not specified");
    let boss_dmg = boss_dmg.expect("boss damage not specified");
    (World::new(boss_dmg), State::new(player_hp, player_mana, boss_hp))
}

// ...why did I not typedef the numbers in rules.rs?  Oh well.
type Best = best::Best<u32, Vec<Spell>, best::Smallest>;

fn search(w: &World, st: &State, tr: &mut Vec<Spell>, b: &mut Best, limit: Option<usize>) {
    if limit == Some(0) {
        return;
    }
    if let Some((best, _)) = b.get() {
        if best <= st.spent() {
            return;
        }
    }
    for sp in Spell::all() {
        tr.push(sp);
        match st.clone().round(w, sp) {
            Lost => (),
            Won(w) => b.add(w.spent, tr),
            Ok(nst) => search(w, &nst, tr, b, limit.map(|n| n-1)),
        }
        let _sp = tr.pop();
        debug_assert_eq!(_sp, Some(sp));
    }
}

fn full_search(w: &World, st: &State) -> (u32, Vec<Spell>) {
    let mut b = Best::new(best::Smallest);
    let mut depth = 5;
    loop {
        search(w, st, &mut Vec::new(), &mut b, Some(depth));
        if b.get().is_some() {
            search(w, st, &mut Vec::new(), &mut b, None);
            return b.unwrap();
        }
        depth += 3;
    }
}

fn main() {
    let (world, state) = parse_args(env::args().skip(1));
    let (cost, spells) = full_search(&world, &state);
    println!("Minimal cost: {}", cost);
    println!("Spell sequence: {:?}", spells);
}
