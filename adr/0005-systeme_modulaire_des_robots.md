# 0005 - Système modulaire des robots

## Status
Accepté

## Context
Le sujet prévoit une architecture modulaire pour permettre à chaque robot d’être spécialisé (forage, imagerie, etc.).

## Decision
Actuellement, le code définit des types fixes de robots (explorateur, mineur), sans système de modules dynamiques. Chaque comportement est codé dans une structure ou système distinct (Bevy ECS).

## Consequences
- Plus simple à maintenir pour l’instant.
- Ne permet pas encore d’ajouter dynamiquement des capacités à un robot.
- Extension future possible via des composants ECS.

## Alternatives considered
- Utiliser des `Box<dyn Trait>` pour chaque module : jugé prématuré et complexe.
