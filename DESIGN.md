# The Games Foxes Play

# Checklist

- Epsilon rooms and segment walls
- Descriptions in E menu
- more axioms
- fill up more text for existing axioms
- check weird superposition with E/C menu and soul-painting

## The redesigned edition

The player starts in the World Seed, Terminal's cage, with 10 active Disgust, and the ability to swap between Introvert and Extrovert modes. 

There are doors on all 4 sides, leading into an empty void.

Each door can be **Revealed** in **Lockdown** or **Low Security** mode by using the buttons neighbouring each door.

## Revealing

Revealing a door will spawn a new room on the other side of that door, containing friends, foes, traps, tricks, anything! The door will stay in **Lockdown** (red light) or **Low Security** (green light) depending on the mode chosen.

## Door access levels

A door in **Lockdown** cannot be opened. A door in **Low Security** can be opened by bumping into it, and can be freely passed through.

These modes can be swapped by using the buttons on the side of the now **Revealed** door.

**Revealing** in **Lockdown** mode is safer. If the room on the other side contains a threat, it will be unable to escape containment and attack the player. **Revealing** in **Low Security** can result in enemies escaping!

Why would one ever want to use **Low Security**?

## Stealth

The Harmony is constantly watching over Faith's End and monitoring any suspicious activity.

Interacting with security buttons is a suspicious activity. Pressing either **Lockdown** or **Low Security** will generate one **Alert**.

Therefore, opening doors in **Lockdown** can be wasteful, as if the room it leads to is safe, this means the following:

- 1 Alert for **Revealing** in **Lockdown**
- 1 Alert for switching to **Low Security** so the room can be entered
- 1 Alert for entering the room

Yes, passing through any doorway also generates one Alert.

Every time 4 Alert has been generated, the Harmony will launch an **Inquisition**.

# Inquisitions

Inquisitions are random events. They can be:

- Sending Harmonic squads of varied purpose, from investigation, suppression or even assimilation
- Dispensing beneficial effects to active squads
- Triggering traps, blocking paths
- Even more self-damage if the player has **Exerted**

# Exertion

The player can lower Alert by 1 by self-suppressing their own Identity. This is basically self-damage to get 1 extra action.

# Room types

- Empty rooms.
- Checkpoints. Just a room with an extra door, wasting time.
- Emotion Light Containment. Some basic enemies with only a single Identity.
- Avatar Heavy Containment. A special boss representing a certain Identity with very high threat. (Ataixa, Anisychia, Rose...)
- 



# Ideas


- Make one of the "inmates" something that takes the form of a seemingly innocuous creature somewhere. It can be noticed through one of its erratic behaviours.
- Spawn all new room tiles "moving from" a main "reality generator" room.
- "When a soul is drawn, draw a soul, give status effect that makes you lose 1 soul from the wheel next turn". A way to peek at the future at the cost of faster churn.

- Instead of "UntargetCaster", make a mutator that makes every form untarget tiles instead of targeting them - "untarget" + "ego" instead of "untargetcaster".

- A cave where you must push around a little cart on a railroad, "cold fire" themed
- A racetrack with many obstacles on the way against an NPC

# Coding

- For NPC AI, enter a game state where events get routed to systems that don't actually affect the game state but calculate heuristics.

# Cool Musical Words

Tempo (ease of containment): Andante, Allegro, Presto, Prestissimo
Volume (impact) Pianissimo, Piano, Forte, Fortissimo


# Tutorial
- Mention the Color implements Copy oddity from bevy discord.
- Mention the exclusive system (&mut World) and associated weird bevy errors.
- A Gimp tutorial for the spritesheet, inspired by the broughlike one.
- Is the ̀`Map` too weird? Maybe it should just fetch the entity with the correct Position?

# Random writing

Kinisi
Volume: Presto (xxx)
Tempo: Piano (xx)

*A dark blue tulip inside a simple clay pot. Its petals twist and turn like muscle, performing a pitiful caricature of nearby creatures' motions. One tempted to laugh at this display should take note of its roots, which pierce reality to draw nourishment from other dimensions.* 

"It's accurate when you realize all language is a way to dumb down concepts in such a way that any mind could understand it"
> link this with Harmony


# Old Itch.io page

This is still a rough demo. Do not expect a high level of polish.

The Games Foxes Play

A nontraditional traditional roguelike about critters and soul harvesting.

Crumble under dozens of stowaway spectral parasites, sell them for profit to a narcissistic hivemind, use laser beams to defeat hordes of cybernetically enhanced snails, die, reincarnate at the edge of spacetime itself, then do it all over again!

This game is in active development, view https://github.com/Oneirical/The-Games-Foxes-Play for information! I post weekly in https://www.reddit.com/r/roguelikedev about my progress. A collection of all these posts can be viewed here!

Controls

I tried to make it so 7 extra arms, 3 tentacles and a few robotic cybernetics are not required to play the game.

w/a/s/d - Move/attack/interact with NPCs in the 4 cardinal directions. There is no diagonal movement in TGFP.

q - Draw Souls from your storage into your hotbar. It's kind of like reloading a gun with multicoloured bullets. It stands for "Quiver". On top of a Soul Cage, it will retrieve the caged soul.

1-8 - Invoke Souls from your wheel. It's kind of like firing said gun. On top of a Soul Cage slot, it will cage the soul instead.  You can also click on the souls to cast them.

l - Open your inventory. You may see the effects of all six basic Soul types, and equip Axioms you have crafted.

f - Open the Research menu. This contains a ton of tutorial information sorted in neat pages, as well as various patterns available for crafting.

c - Enter examine mode, to read tile descriptions and creature abilities. It stands for "Cursor". You can then move your mouse around to inspect things!

Design goals:

    Failure is as valuable as success - Death of the physical body is unimportant - only faith matters. Failing encounters causes a peculiar infection to spread further, unlocking great power at the cost of one's very identity...

    An unusual alternative to experience points and spellcasting - Slaying creatures awards you with Souls. Souls can be invoked for minor powers, or fuelled by Axioms to unleash even greater effects, fully customizable with an in-game spell crafting system!

    Map generation that responds to player interaction - The dungeons in TGFP are dreamscapes, moulded by those who imagine them. Depending on which Souls you select and where you place them in the Soul Cage, the denizens populating the illusory worlds below can range from fully mechanized snakes to eerily staring cat statues.

    Extensive lore presented through nonmodal elements - Should you ever start wondering if the setting actually makes sense or if I just injected metric tons of illegal psychedelics in my bloodstream to make this game, you can learn all my worldbuilding through flavour text and dialogue, but you will never be locked out of key-mashing and combat. Those who don't care about the "why" and just want to smash some robotic critters can easily do so without interruption. 

Version history:

    Version 0.4.3: 3 new spell components & bug fixes!
    Version 0.4.2: "Brush & Canvas" Soul Cage crafting UI rework. (4th of May 2023)
    Version 0.4.1: Rework of Soul Cage Crafting algorithm to be more predictable and polished.
    Version 0.4.0: Introduction of Research Tree, Soul Cage and associated crafting mechanics.
    Version 0.3.0: Map generation.
    Version 0.2.0: Overhaul of UI and Legendary Soul system.
    Version 0.1.0: First playable demo.
