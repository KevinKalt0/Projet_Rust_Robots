# Projet de Simulation de Robots Explorateurs

## Présentation

Ce projet est une simulation interactive de robots explorateurs et mineurs dans un environnement 2D développée avec le moteur de jeu Bevy en Rust. La simulation met en scène un explorateur qui recherche des ressources et des mineurs qui les collectent pour les ramener à une base centrale.

## Fonctionnalités

### Environnement
- **Génération procédurale** : Terrain généré avec du bruit de Perlin pour créer un environnement unique à chaque lancement
- **Obstacles variés** : Des obstacles et des murs qui doivent être contournés par les robots
- **Ressources** : Deux types de ressources (Énergie et Minéraux) réparties sur la carte

### Robots
- **Explorateur** :
  - Se déplace aléatoirement dans l'environnement
  - Détecte automatiquement les ressources à proximité
  - Évite intelligemment les obstacles
  
- **Mineurs** :
  - Attendent à la base jusqu'à ce qu'une ressource soit découverte
  - Se déplacent en groupe vers les ressources découvertes
  - Collectent les ressources et les ramènent à la base
  - Effectuent deux allers-retours à chaque fois, en raison de la quantité abondante de ressources
  - Utilisent un algorithme avancé pour contourner les obstacles

### Mécanismes de jeu
- **Détection de ressources** : L'explorateur identifie automatiquement les ressources proches
- **Collecte collaborative** : Les mineurs travaillent ensemble pour collecter les ressources
- **Temps de collecte** : Un délai de 2 secondes pour simuler le temps nécessaire à l'extraction
- **Cycle complet** : Exploration → Découverte → Extraction → Retour à la base

### Techniques implémentées
- **Évitement d'obstacles** : Algorithme sophistiqué permettant aux robots de contourner les obstacles
- **Détection de collision** : Système précis qui empêche les robots de traverser les obstacles
- **Génération procédurale** : Création dynamique de l'environnement avec différentes densités d'obstacles
- **Synchronisation multi-entités** : Coordination entre l'explorateur et les mineurs

## Architecture technique

Le projet est structuré autour du pattern ECS (Entity-Component-System) de Bevy :

- **Entités** : Explorateur, Mineurs, Base, Ressources, Obstacles
- **Composants** : Position, Vitesse, État (Idle, Actif, Retour)
- **Systèmes** : Déplacement, Détection, Collecte, Génération de carte

### Structure du code
- `main.rs` : Point d'entrée qui configure l'application Bevy
- `robots.rs` : Implémentation de toute la logique de simulation
- `tests/` : Tests unitaires pour valider les fonctionnalités

## Comment jouer

1. **Installation**
   ```bash
   git clone https://github.com/KevinKalt0/Projet_Rust_Robots.git
   cd rust_robot
   cd simulation_robots
   cargo run
   ```

2. **Observations**
   - Robot vert : Explorateur qui découvre les ressources
   - Robots orange : Mineurs qui collectent les ressources
   - Carré bleu : Base où retournent les mineurs
   - Points jaunes : Ressources d'énergie
   - Points bleus : Ressources minérales
   - Blocs gris : Obstacles

## Tests

Le projet inclut une suite de tests unitaires qui vérifient le bon fonctionnement des éléments clés :
- Détection des obstacles
- Algorithme de contournement
- Génération de la carte
- Rotation des vecteurs

Pour exécuter les tests :
```bash
cargo test
```

## Technologies utilisées

- **Rust** : Langage de programmation performant et sûr
- **Bevy** : Moteur de jeu moderne avec architecture ECS
- **Noise** : Bibliothèque pour la génération de bruit procédural
- **Rand** : Génération de nombres aléatoires

## Perspectives d'évolution

- Ajout de nouveaux types de robots avec des capacités spécifiques
- Implémentation d'un système d'inventaire et de ressources
- Création de niveaux avec des objectifs précis
- Amélioration des graphismes avec des sprites et des animations

---

*Ce projet a été développé dans le cadre d'un exercice de programmation en Rust pour explorer les concepts de simulation, d'intelligence artificielle simple et de génération procédurale.*
