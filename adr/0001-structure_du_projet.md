# 0001 - Structure du projet

## Status
Accepté

## Context
Le projet 'EREEA' vise à simuler un essaim de robots pour explorer des terrains extraterrestres. Afin de garantir la lisibilité, la maintenabilité, et la modularité, il est crucial de définir une structure de projet claire et idiomatique pour le langage Rust.

## Decision
Le projet est structuré comme suit :

- `src/main.rs` : point d’entrée principal de l’application, lance la simulation.
- `src/lib.rs` : permet de regrouper les modules exportés.
- `src/robots.rs` : contient toute la logique liée aux robots, à la carte, et à l’environnement.
- `tests/tests.rs` : tests d’intégration pour les fonctions critiques (A*, génération, collisions).
- `assets/` : images des entités pour le rendu Bevy.

## Consequences
- Structure claire et lisible, adaptée à un projet pédagogique.
- Permet des extensions futures avec d'autres modules (station, communication, etc.).

## Alternatives considered
- Utiliser plusieurs fichiers séparés pour chaque type de robot ou entité. Rejeté pour limiter la complexité.
- Architecture ECS personnalisée : Bevy intègre déjà un système suffisant.
