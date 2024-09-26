EGO: Self target
BEAM: Beam in direction of last move
SMOOCH: Melee, dir last move
ALL: All entities on screen
SELECT: All entities on screen of that type
FLOOD: All connected air tiles
RESET: Remove all targets
CIRCLET: all adjacent tiles

JOLTZAZON: Targets spread into adjacent entities
TRAIL: also target everything in the way of the moving marked entities
SPREAD: every entity spreads orthogonally
REPEAT: the following praxes are executed 5 times in a row

PARACEON: Invincibility for 1 turn
GYVJI: Bash into walls in direction of last move
RASEL: Anidead effect (on foes)
DEATHCLICK: Lay down trap with the rest of the axioms, it triggers when something dies on it
SOULSTEAL: grabs repressed souls, even of ghosts
HARM: do dmg
BLINK: random blink
GRAVITY: pin everyone to obey the laws of gravity for X turns
LOCK: Lock doors
FREEZE: turn air into ice, which makes you slide and bash yourself if ends in solid
SETHARM: forcefully set an HP value
REGEN: give a regen effect
INVISIBLE: give invis effect
ROOMTELE: teleport into an adjacent room (momentum)

SMOOCH SOULSTEAL "melee spellsteal + max hp reducer"

EGO PARACEON GYVJI CIRCLET HARM "self rocket blast"

BEAM JOLTZAZON GYVJI RASEL "chain lightning with knockback + anidead"

EGO DEATHCLICK EGO SUMMON(FELIDOL) SOULSTEAL "jade shards that steal souls"

EGO TRAIL DASH SPREAD HARM "dash, then hit everything that was close to the dash"

REPEAT EGO BLINK RESET CIRCLET HARM "blink slash blink slash blink slash"

Anisychia:

ALL GYVJI GRAVITY "vertigo"

SELECT(DOOR) LOCK "lockdown"

FLOOD FREEZE "force slipperiness"

CIRCLET SETHARM(1) REGEN(10) "panic set you to 1 hp then regen"

ALL INVISIBLE "isolate you by making everything invisible"

Rose:

BEAM (like a ribbon) TWIN "makes you start converting your own allies"

EGO ROOMTELE "goes into the ground and follows you"

NEAREST-AIR GAS ASSIMILATE (one of Rose's other spells) "spreads gas and gives you Rose spells until you turn"
