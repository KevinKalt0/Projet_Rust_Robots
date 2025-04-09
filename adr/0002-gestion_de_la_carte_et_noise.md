# 0002 - Gestion de la carte et génération par bruit

## Status
Accepté

## Context
La carte doit contenir des obstacles, des ressources (énergie, minerais, sites scientifiques). Le sujet impose une génération via bruit, avec seed reproductible.

## Decision
La bibliothèque `noise-rs` est utilisée avec `Perlin::new(seed)` pour créer une carte cohérente. Les ressources sont ensuite placées à des positions valides générées aléatoirement avec cette seed.

## Consequences
- Génération déterministe de la carte avec obstacles.
- Les ressources sont cohérentes avec le terrain.
- Fonction `generate_map` centralise cette logique.

## Alternatives considered
- Cartes codées en dur : non évolutives.
- Génération 100% aléatoire sans bruit : résultats peu réalistes.
