# IVISS — Aperçu du Déploiement et des Versions

---

## 1. Infrastructure d'hébergement

IVISS est hébergé sur **AWS Lightsail** — un serveur cloud géré par Amazon Web Services. L'application fonctionne dans des conteneurs isolés, ce qui la rend portable, sécurisée et facile à mettre à jour.

| Composant | Détails |
|---|---|
| Fournisseur cloud | AWS Lightsail |
| Serveur | 2 vCPUs, 2 Go de RAM, 60 Go SSD |
| Système d'exploitation | Ubuntu 22.04 LTS |
| API Backend | Application Rust conteneurisée |
| Frontend | Application React conteneurisée, servie via Nginx |
| Base de données | PostgreSQL avec stockage persistant |

L'ensemble de l'infrastructure serveur est défini sous forme de code — ce qui signifie que l'environnement peut être reproduit ou migré de manière fiable, sans configuration manuelle.

---

## 2. Stratégie de déploiement

Chaque mise à jour de l'application passe par un pipeline entièrement automatisé. Une fois qu'un changement est validé et approuvé par l'équipe de développement, il est déployé sur le serveur sans aucune intervention manuelle.

Le pipeline suit cette séquence :

1. **Revue de code** — les modifications sont examinées par l'équipe avant d'être acceptées
2. **Vérifications automatiques** — les tests et contrôles qualité s'exécutent automatiquement
3. **Création d'une version** — un nouveau numéro de version est attribué et publié
4. **Empaquetage** — l'application est conditionnée en images Docker et stockée dans un registre privé
5. **Déploiement** — le serveur récupère les nouvelles images et redémarre avec la version mise à jour

Cette approche garantit que seul du code revu et testé atteint le serveur de production.

---

## 3. Gestion des versions

Chaque déploiement produit une version numérotée selon le standard **Semantic Versioning** — une convention largement adoptée dans l'industrie du logiciel. Les numéros de version suivent le format `MAJEUR.MINEUR.CORRECTIF` (par exemple `v1.2.3`).

- Une version **correctif** (ex. `v0.1.1`) signifie qu'un bug a été corrigé
- Une version **mineure** (ex. `v0.2.0`) signifie qu'une nouvelle fonctionnalité a été ajoutée
- Une version **majeure** (ex. `v1.0.0`) signifie qu'un changement significatif a été apporté au système

Les numéros de version sont attribués automatiquement en fonction de la nature des modifications — aucune décision manuelle n'est requise. Toutes les versions sont publiées sur la page GitHub du projet avec la liste complète des changements incluse.

---

## 4. Sécurité et configuration

Toutes les configurations sensibles — notamment les identifiants de base de données, les clés d'authentification et les clés d'API tierces — sont stockées de manière sécurisée sous forme de secrets chiffrés dans le système CI/CD. Ces valeurs ne sont jamais stockées dans le code source et ne sont injectées sur le serveur qu'au moment du déploiement.

La plateforme prend en charge plusieurs fournisseurs pour les notifications SMS et e-mail, configurables sans modification du code :

- **SMS :** Orange Cameroun, Twilio ou Vonage
- **E-mail :** SMTP (Gmail, Outlook ou serveur personnalisé) ou Resend

---

*IVISS — Documentation Technique Interne*
