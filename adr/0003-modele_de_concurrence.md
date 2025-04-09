# 0003 - Modèle de concurrence

## Status
Accepté

## Context
Le sujet propose l’utilisation de modèles concurrents pour simuler le comportement indépendant des robots et de la station. Il est important de choisir un modèle cohérent avec l’approche Bevy utilisée dans le projet.

## Decision
Le projet utilise Bevy, qui repose sur un moteur ECS (Entity-Component-System) avec gestion interne des systèmes parallèles. Aucune gestion manuelle avec `std::thread`, `tokio`, ou `async` n’est utilisée.

## Consequences
- Code plus simple et mieux intégré au moteur Bevy.
- La parallélisation des systèmes est gérée automatiquement par Bevy.
- Moins de contrôle manuel mais plus de sécurité pour un projet pédagogique.

## Alternatives considered
- Threads manuels ou async/await : plus complexe et non nécessaire avec Bevy.
- Acteurs ou worker pools : possible mais non implémenté ici.
