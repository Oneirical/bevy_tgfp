use bevy::{
    color::{
        palettes::css::{
            ANTIQUE_WHITE, BURLYWOOD, DARK_SALMON, DARK_SEA_GREEN, LIGHT_BLUE, LIGHT_GRAY, LIME,
            MAGENTA, ORANGE_RED, PINK, VIOLET, WHITE, YELLOW,
        },
        Color,
    },
    log::info,
    text::TextColor,
};

use crate::{
    creature::{Soul, Species},
    spells::Axiom,
};

use regex::Regex;

pub const LORE: &[&str] = &[
"Unknown.",
"Its melee attacks cause it to heal itself for 1 HP.",
"Resilient, yet slow, acting once every two turns.",
"It moves erratically, and sculpts sentries from walls. These crumble into dust once their creator is slain.",
"It charges up as it moves, empowering its next melee attack with 1 bonus damage every 5 steps.",
"Frail, but fast, acting twice every turn.",
"It hungers, devouring nearby walls to regenerate.",
"It opens once all hostile creatures in its connected room are slain.",
"It blocks movement, but is vulnerable to magical effects.",
"It blocks movement.",
"It's you.",
"It strikes at foes which approach it and is incredibly robust, but crumbles once its creator is slain.",
"You, and all adjacent creatures, heal for 2 HP.",
"You cannot take damage next turn. Instantaneous.",
"Places a trap at your feet. The next creature to step on it will cause it to fire 2 damage beams in all 4 cardinal directions.",
"Fires 4 beams in all diagonal directions, dealing 2 damage.",
"Dashes 5 tiles in the direction you are facing, attacking all creatures adjacent to your path with 1 damage. Creatures struck at the end are knocked backwards.",
"The next time you strike with a melee attack, deal 6 damage.",
"[y]Arrow Keys[w] or [y]WASD[w]: Move or melee attack one step in the cardinal directions.\n[y]Space[w] or [y]Q[w]: Draw one Soul on the Soul Wheel.\n[y]1-8[w]: Cast a spell corresponding to the chosen slot on the Soul Wheel.\n[y]C[w]: Enter Cursor mode to learn more about the 6 enemy types.\n[y]E[w]: Enter Caste mode to learn more about the 6 available spells.\n[y]Z[w] or [y]X[w]: Reset the game.",
"Press [y]1-6[w] to learn about the 6 different spells.",
"[b]Haughty as the Saints Were[w]\n[l]Form[w]\n\n[r]Target[w] the tile on which the caster stands.\n\n[p]@In a realm where sheer belief draws the line between what is and what is not, pride is omnipotence. The Saints guarded fiercely this primordial truth.",
"[i]Hechaton, Ribbon-Adorned Gardener[w]\n[o]Function[w]\n\nAll [r]Targeted Creatures[w] transform into a [s]Terracotta Sentry[w].\n\n[p]@Finding anything but the sappiest praise for Hechaton's sculptures is difficult. His few critics have become indistinguishable from the art they used to bash, their limbs petrified and their stone eyes keeping watch over the botanical gardens.",
"[g]Mark History Where One Passes[w]\n[m]Mutator[w]\n\nUntil the end of this spell, all [r]Targeted Creatures[w] will leave behind a linear trail when they [m]Teleport[w]. All tiles in the trail's path become [r]Targets[w].\n\n[p]@As Saints turned the impossible to the always-has-been with each new desire, Old World historians struggled to trace back each ruin, each monument and each ideology to its origin. In the end, they accepted that time is not a line, nor a circle, but a tangled knot of pure chaos.",
"[y]A Click, Then a Flash[w]\n[o]Function[w]\n\nSkip all remaining Axioms. On all [r]Targets[w], place down a trap, storing all skipped Axioms within. When a creature steps onto the trap, trigger all stored Axioms and remove the trap.\n\n[p]@Where the Artistic galleries do not have guards, they have traps. A careless step, and would-be looters become the looted, their every memory sold to the highest bidder on the hivemind's network.",
"[^]Aspha, Nemesis of the Unseen[w]\n[l]Form[w]\n\n[r]Target[w] all tiles on the outskirts of a circle centered on the caster. Its radius is [y]4[w] tiles.\n\n[p]@At one point in the history of Old World warfare, use of camouflage and invisibility technology became omnipresent. The solution was, of course, to build a very energy-hungry robot which considered air itself to be its mortal enemy. Overkill? The stealth-bots - or, at the very least, what remains of them - would disagree.",
"[y]Yearnings Crossed Out[w]\n[l]Form[w]\n\nShoot beams in all four diagonal directions, each stopping when a creature is met. [r]Target[w] all tiles they pass through, as well as the obstacles which stopped them.\n\n[p]@The Unhinged swore away control and domination, living according to whim and impulse. Were they truly free, or controlled by the ideology of freedom?",
"[g]Terror and Thirst, Focused[w]\n[l]Form[w]\n\nShoot a linear beam in the direction of the caster's last step, which stops upon hitting a creature or solid tile.\n\n[r]Target[w] all tiles traversed by the beam, as well as the obstacle that stopped it.\n\n[p]@All Old World denizens know not to trust the Feral's collars and chains. One twitch, one snap, and their bodies surge out like a bullet, stopped by anything but reason.","[l]Steps Shift The Mind[w]\n[y]Contingency[w]\n\nWhen the caster [m]Teleports[w], cast this spell.\n\nTo walk and let the mind wander is a dangerous thing. A thought pulls harder than the rest, one's gait softens into the grace of a Saint, tears turn to bright smiles, and before one knows it, one is no more.",
"The head of a gigantic mechanical snake, its blazing red eyes burning away the retinas of organics whom would dare stare too long. Its gold and chrome frills act as an attestation of the superiority of metal over muscle.\n\n[r]MELTDOWN[w] - Each turn, if this [y]Creature[w] is adjacent to 4 [y]Creatures[w], it gains one [l]Meltdown[w]. Upon reaching 5 [l]Meltdown[w], it immediately [r]Concedes[w].",

"Cyan Floods Wash Away Scorn - If possessed, Inject 1 Serene Soul into each Targeted Creature. Targeted Creatures become Charmed for Pride x 10 turns.",
"Steps Aligned, Minds United - Each Targeted Creature becomes Synchronized with the Caster for Grace x 10 turns.",
"One's Self, Hollow As A Costume - If the Caster possesses the Reality Anchor, it is given to the first Targeted Creature. After Glamour x 10 turns, it is given back to the Caster.",
"Imitate the Glorious, So They May Be Crushed - The Caster changes its Species to match that of the last Targeted Creature. After Discipline x 10 turns, it changes back to its old form.",
"Focused Thought Pierces the Veil - Form\nThe Caster shoots a linear beam in the direction of its Momentum, stopping at the first Creature hit. All Tiles touched, including the contacted Creature, are Targeted.",
];

pub fn match_species_with_description(species: &Species) -> &str {
    LORE[match species {
        Species::Hunter => 1,
        Species::Apiarist => 2,
        Species::Tinker => 3,
        Species::Oracle => 4,
        Species::Shrike => 5,
        Species::Second => 6,
        Species::Airlock => 7,
        Species::WeakWall => 8,
        Species::Wall => 9,
        Species::Player => 10,
        Species::Abazon => 11,
        _ => 0,
    }]
}

pub fn match_soul_with_description(soul: &Soul) -> &str {
    LORE[match soul {
        Soul::Saintly => 12,
        Soul::Ordered => 13,
        Soul::Artistic => 14,
        Soul::Unhinged => 15,
        Soul::Feral => 16,
        Soul::Vile => 17,
        Soul::Empty => 19,
        _ => 0,
    }]
}

pub fn match_axiom_with_description(axiom: &Axiom) -> &str {
    LORE[match axiom {
        Axiom::Ego => 20,
        Axiom::Transform { species: _ } => 21,
        Axiom::Trace => 22,
        Axiom::PlaceStepTrap => 23,
        Axiom::Halo { radius: 4 } => 24,
        Axiom::XBeam => 25,
        Axiom::MomentumBeam => 26,
        Axiom::WhenMoved => 27,
        _ => 0,
    }]
}

pub fn split_text(text: &str) -> Vec<(String, TextColor)> {
    let re = Regex::new(r"\[([^\]]+)\]").unwrap();

    let mut split_text = Vec::new();
    let mut colors = Vec::new();
    let mut last_end = 0;

    for cap in re.captures_iter(text) {
        let start = cap.get(0).unwrap().start();
        let end = cap.get(0).unwrap().end();
        let tag = cap.get(1).unwrap().as_str().chars().next();
        colors.push(match_char_code_with_color(tag));
        split_text.push(&text[last_end..start]);
        last_end = end;
    }
    split_text.push(&text[last_end..]);

    let mut output = Vec::new();

    for i in 0..split_text.len() {
        let color = if i == 0 { Color::WHITE } else { colors[i - 1] };
        output.push((split_text[i].to_owned(), TextColor(color)));
    }
    output
}

fn match_char_code_with_color(some_char: Option<char>) -> Color {
    match some_char {
        Some(char) => match char {
            'p' => VIOLET.into(),
            'r' => ORANGE_RED.into(),
            'y' => YELLOW.into(),
            'w' => WHITE.into(),
            'l' => LIME.into(),
            'c' => LIGHT_BLUE.into(),
            'm' => MAGENTA.into(),
            'd' => DARK_SEA_GREEN.into(),
            'b' => BURLYWOOD.into(),
            's' => DARK_SALMON.into(),
            'a' => ANTIQUE_WHITE.into(),
            'i' => PINK.into(),
            '^' => LIGHT_GRAY.into(),
            'o' => Color::srgb(0.94, 0.55, 0.38),
            'g' => Color::srgb(0.66, 0.82, 0.11),
            _ => {
                info!("Warning, an invalid color tag was used.");
                Color::WHITE
            }
        },
        None => panic!("There was no character in the text split!"),
    }
}
