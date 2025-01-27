use bevy::{
    color::{
        palettes::css::{
            ANTIQUE_WHITE, BURLYWOOD, DARK_SALMON, DARK_SEA_GREEN, LIGHT_BLUE, LIME, MAGENTA,
            ORANGE_RED, VIOLET, WHITE, YELLOW,
        },
        Color,
    },
    log::info,
    text::TextColor,
};

use crate::creature::{Soul, Species};

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
