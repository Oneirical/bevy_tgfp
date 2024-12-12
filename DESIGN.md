# The Games Foxes Play

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

# Coding

- For NPC AI, enter a game state where events get routed to systems that don't actually affect the game state but calculate heuristics.

# Cool Musical Words

Tempo (ease of containment): Andante, Allegro, Presto, Prestissimo
Volume (impact) Pianissimo, Piano, Forte, Fortissimo


# Tutorial
- Mention the Color imprements Copy oddity from bevy discord.
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
