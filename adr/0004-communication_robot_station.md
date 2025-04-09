# 0004 - Communication entre robots et station

## Status
Accepté

## Context
Le sujet demande que les robots partagent leurs informations uniquement lorsqu’ils retournent à la station. Ce modèle de synchronisation n’est pas encore implémenté dans le code actuel.

## Decision
À ce stade du projet, la station n’est pas une entité active. Les robots (explorateur et mineurs) agissent en autonomie locale. Une synchronisation explicite avec une entité 'station' est prévue dans les versions futures.

## Consequences
- Permet de se concentrer d’abord sur les mécaniques de déplacement et collecte.
- La logique de fusion des connaissances reste à définir.

## Alternatives considered
- Ajout immédiat de la station avec communication : aurait alourdi la première itération du projet.
