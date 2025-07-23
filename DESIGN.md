# The Games Foxes Play

## The redesigned edition

# Level 1: The Quarry

The Quarry is "mining out" the denizens of the world below in chunks to fill Terminal up with their souls. The Harmony hopes to concentrate the entire population in one body so it is easier to harmonize everyone all at once, and without needing to descend on the surface where the population is highly hostile to outsiders.

Gameplay:

- Cages arrive on the treadmill one by one in increasing difficulty. These challenges are solved using the 6 basic souls.
- Occasionally, a miniboss spawns with a special spell, and it rewards the player with a new pattern to use on the Canvas.
- The player is invited to try their hand at crafting with these patterns.
- If the player dies, a Harmonic Nurse will drag them out and inject them with a Serene Soul to revive them (it boosts their ego so much they will themselves alive again)
    - This must eventually happen. If the player stays alive for too long, put them against an impossible miniboss but reward them with a nice rare pattern to compensate if they perform well against it. This should be a "Harmonic Anaesthetist" who is timing the injection of the Serene soul precisely with the Quarry's expected completion date.

After some time, one of the cages will be booby-trapped by Epsilon's squad. It will explode out of the quarry area and send out a militia to take over the facility. The Harmony will attempt to force them to inhale their gaseous forms, then realize that they are unbreathing mechanical beings.

The player is warned to stay out of the way of these insurgents. If the player challenges them, they should be faced with an extremely difficult challenge which, if lost, ends up with them attached as an Epsilon segment.
If they comply, Epsilon informs the player that they are infected by a terrible disease, and that they can be cured while still maintaining some level of autonomy if they journey down to the Panopticon. The way is opened to the Stations.

# Level 2: The Stations

This level takes place in the deeply frozen area which the Ordered spawned an ice age on in order to cool their server farms. As the tunnels are extremely cold, the main gimmick involves pushing a flaming cart across a railroad to keep warmth while battling swarms of robotic denizens trying to put out the flame.

If the flame is put out, the player becomes another mining robot. If the infection progresses too much, they are beamed up by the Harmony in an epic showdown against Epsilon.

The player will eventually find a train leading them to the Panopticon.

# Level 3: The Panopticon

Currently, this is where the game will end. The player becomes another mechanical member of the Panopticon. The infection is gone, but so is their identity as Terminal. Later developments should allow for some way to escape the Panopticon.

Railways Where Fire Turns Upside Down, Ever Growing (O) (gimmick: anti-entropy)
Gardens Where Roses Become Peonies, Then Return (S) (gimmick: constant swapping around and shapeshifting)
Forests Where Grass Cuts Flesh, With Blood Unshed (F) (gimmick: illusions and traps everywhere)
Hives Where All That Was Wrong, Is Made Right (U) (gimmick: very large and very small creatures)
Ships Where Life is Well Worth Free Will, Floating in Void (V) (gimmick: no direct fights, only allies)
Towers Where Cats Pat At Dust, Floating Among Sunrays (A) (gimmick: gravity)

# Ideas


- Make one of the "inmates" something that takes the form of a seemingly innocuous creature somewhere. It can be noticed through one of its erratic behaviours.
- "When a soul is drawn, draw a soul, give status effect that makes you lose 1 soul from the wheel next turn". A way to peek at the future at the cost of faster churn.
- Instead of "UntargetCaster", make a mutator that makes every form untarget tiles instead of targeting them - "untarget" + "ego" instead of "untargetcaster".
- A racetrack with many obstacles on the way against an NPC
- Multiple soul cage sizes and shapes, like a donut or a triangle.
- An enemy that does more damage when it attacks you on your side (Momentum)

# Coding

- Make text messages only appear if what they are reporting is visible to the player.
- For NPC AI, enter a game state where events get routed to systems that don't actually affect the game state but calculate heuristics.
- Use Bevy's Focus for UI keyboard input

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
