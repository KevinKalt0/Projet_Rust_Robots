# 0006 - Persistance et seed pour reproductibilité

## Status
Accepté

## Context
Le sujet impose que la génération de carte soit reproductible avec une seed. Cela est essentiel pour les tests et le débogage.

## Decision
La seed est passée en paramètre à la fonction `generate_map`. Elle peut être codée en dur ou générée aléatoirement et affichée au lancement pour être réutilisée manuellement.

## Consequences
- Reproductibilité assurée à chaque exécution.
- Permet d’écrire des tests fiables.
- Génération future de seed via argument CLI possible.

## Alternatives considered
- Génération sans seed : impossible à tester proprement.
- Seed uniquement pour les obstacles : limiterait la reproductibilité des ressources.
