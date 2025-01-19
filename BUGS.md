# Major

- Very occasionally, the game will crash when passing through a door due to the door recovering tangibility on top of another creature.

# Minor

- The "defeated/victorious" title flickers for a brief moment when spawned.
- Doors keep their fading out effect if opened too fast.
- Very rarely, the game does not accept any input on startup and must be restarted

# Bad code

- `assign_species_components` and its partner deleter are very bad. I am considering giving creatures 2 child entities, one with the species components and the other with the status effect components.
- The chain spawning code in `ui.rs` is super spammy.
- Anything with "HACK" comments in the codebase.
