# Migration vers Scaleway — OpenCab

## Contexte et choix d'architecture

### Pourquoi migrer depuis Supabase + Cloud Run ?

- **Supabase free tier** : limite à 500 MB de disque. Le WAL et les fichiers système PostgreSQL consomment ~240 MB même avec seulement 30 MB de données réelles. La limite sera atteinte sans croissance des données.
- **Simplicité** : une seule infrastructure à gérer au lieu de trois (Supabase, Cloud Run, Scaleway).
- **Coût** : le serveur Scaleway est déjà payé.
- **Latence DB** : l'app et la base de données sur le même serveur = connexion localhost.
- **Pas de cold start** : le processus tourne en continu, contrairement à Cloud Run avec `min-instances: 0`.

### Architecture cible

```
[Internet]
    │ HTTPS
    ▼
[Caddy]  ← déjà en place, gère les certificats Let's Encrypt
    │ HTTP (localhost)
    ▼
[opencab container]  ← app Rust, Docker
    │ TCP localhost
    ▼
[postgres container]  ← PostgreSQL 15.8, Docker volume sur Block Storage

[Cloudflare R2]  ← fichiers uploadés (signatures) + backups PostgreSQL
    ▲
    └── opencab (API S3-compatible, 10 GB gratuits, zéro egress)

[Cron quotidien]  ← backup pg_dump → Cloudflare R2
```

### Infrastructure

| Composant | Service | Détail |
|---|---|---|
| Serveur | Scaleway STARDUST1-S, WAW2 | 1 core, 1 GB RAM, IPv6 only |
| Disque | Block Storage 5K, 10 GB | Disque système — OS + Docker + volumes |
| Fichiers & Backups | Cloudflare R2 | 10 GB gratuits, zéro frais d'egress, API S3 |

### Le Block Storage 10 GB

C'est le **disque dur du serveur** : OS, Docker, images de containers, et volumes Docker (dont `pgdata`). Il n'est **pas** adapté pour stocker des fichiers uploadés par les utilisateurs car :
- Il est partagé avec l'OS et Docker
- Si le serveur est recréé, les données seraient perdues sans snapshot
- 10 GB se remplit vite avec l'OS (~2-3 GB) + images Docker (~1-2 GB)

Pour les signatures, documents et backups, utilise **Cloudflare R2** : 10 GB gratuits, aucun frais de sortie de données, et API S3-compatible (zéro changement de logique dans le code).

---

## Étapes de migration

### Étape 1 — Préparer le serveur

SSH sur le serveur :
```bash
ssh root@2001:bc8:1d90:2250:dc00:ff:fe2b:cd4f
```

Créer l'arborescence :
```bash
mkdir -p /opt/opencab
mkdir -p /opt/scripts
```

### Étape 2 — docker-compose.prod.yml

Créer `/opt/opencab/docker-compose.yml` :

```yaml
services:
  postgres:
    image: postgres:15.8
    container_name: postgres
    environment:
      POSTGRES_USER: opencab
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
      POSTGRES_DB: opencab
    volumes:
      - pgdata:/var/lib/postgresql/data
    restart: unless-stopped
    # Pas exposé sur Internet

  opencab:
    image: lethib/opencab:latest
    container_name: opencab
    environment:
      ENVIRONMENT: production
      APP__DATABASE__URL: postgresql://opencab:${POSTGRES_PASSWORD}@postgres:5432/opencab
      SMTP_SERVER_HOST: ${SMTP_SERVER_HOST}
      SMTP_SERVER_PORT: ${SMTP_SERVER_PORT}
      SMTP_AUTH_USER: ${SMTP_AUTH_USER}
      SMTP_AUTH_PASSWORD: ${SMTP_AUTH_PASSWORD}
      APP__JWT__SECRET: ${APP__JWT__SECRET}
      SSN_SALT_KEY: ${SSN_SALT_KEY}
      SSN_ENCRYPTION_KEY: ${SSN_ENCRYPTION_KEY}
      # Remplacer SUPABASE_* par Cloudflare R2
      S3_ENDPOINT: https://<account-id>.r2.cloudflarestorage.com
      S3_BUCKET: opencab-signatures
      S3_ACCESS_KEY: ${R2_ACCESS_KEY}
      S3_SECRET_KEY: ${R2_SECRET_KEY}
    depends_on:
      - postgres
    restart: unless-stopped
    ports:
      - "127.0.0.1:8080:5150"  # Caddy → container, non exposé à l'extérieur

volumes:
  pgdata:
```

Créer `/opt/opencab/.env` (ne pas committer) :
```
POSTGRES_PASSWORD=...
APP__JWT__SECRET=...
SSN_SALT_KEY=...
SSN_ENCRYPTION_KEY=...
SMTP_SERVER_HOST=smtp.gmail.com
SMTP_SERVER_PORT=465
SMTP_AUTH_USER=...
SMTP_AUTH_PASSWORD=...
R2_ACCESS_KEY=...
R2_SECRET_KEY=...
```

### Étape 3 — Migrer les données depuis Supabase

Sur ta machine locale, exporter depuis Supabase :
```bash
pg_dump "postgresql://postgres:[password]@db.fvmcwmxuugbbzayhqnyw.supabase.co:5432/postgres" \
  --no-owner \
  --no-acl \
  -f dump.sql
```

Copier sur le serveur et importer :
```bash
scp dump.sql root@[ipv6]:/opt/opencab/
ssh root@[ipv6] "docker exec -i postgres psql -U opencab opencab < /opt/opencab/dump.sql"
```

### Étape 4 — Migrer les fichiers depuis Supabase Storage

Créer deux buckets dans Cloudflare R2 : `opencab-signatures` et `opencab-backups`.

Configurer `rclone` sur ta machine locale pour les deux côtés :

```ini
# /root/.config/rclone/rclone.conf

[supabase]
type = s3
provider = Other
access_key_id = <SUPABASE_ACCESS_KEY>
secret_access_key = <SUPABASE_SECRET_KEY>
endpoint = <SUPABASE_S3_ENDPOINT>

[r2]
type = s3
provider = Cloudflare
access_key_id = <R2_ACCESS_KEY>
secret_access_key = <R2_SECRET_KEY>
endpoint = https://<account-id>.r2.cloudflarestorage.com
```

Copier les fichiers :
```bash
rclone copy supabase:signatures r2:opencab-signatures
```

### Étape 5 — Modifier le code : Supabase Storage → R2

Dans le code Rust, remplacer les appels à l'API Supabase Storage par des appels S3 standard (via `aws-sdk-s3` ou `s3` crate). Seules les URLs et credentials changent, la logique reste identique.

### Étape 6 — Modifier le pipeline GitHub Actions

Remplacer `gcloud run deploy` par un déploiement SSH :

```yaml
- name: Deploy to Scaleway
  uses: appleboy/ssh-action@v1
  with:
    host: 2001:bc8:1d90:2250:dc00:ff:fe2b:cd4f
    username: root
    key: ${{ secrets.SCALEWAY_SSH_KEY }}
    script: |
      docker pull lethib/opencab:latest
      cd /opt/opencab
      docker compose up -d --no-deps --force-recreate opencab

- name: Run migrations
  uses: appleboy/ssh-action@v1
  with:
    host: 2001:bc8:1d90:2250:dc00:ff:fe2b:cd4f
    username: root
    key: ${{ secrets.SCALEWAY_SSH_KEY }}
    script: |
      docker run --rm --network opencab_default \
        --env-file /opt/opencab/.env \
        lethib/opencab:latest /app/migrate
```

### Étape 7 — Backups automatiques

Installer `rclone` sur le serveur :
```bash
curl https://rclone.org/install.sh | bash
```

Configurer pour Cloudflare R2 (`/root/.config/rclone/rclone.conf`) :
```ini
[r2]
type = s3
provider = Cloudflare
access_key_id = <R2_ACCESS_KEY>
secret_access_key = <R2_SECRET_KEY>
endpoint = https://<account-id>.r2.cloudflarestorage.com
```

Créer le script `/opt/scripts/backup-postgres.sh` :
```bash
#!/bin/bash
set -e

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="/tmp/opencab_${TIMESTAMP}.sql.gz"

echo "[$(date)] Démarrage du backup..."

docker exec postgres pg_dump -U opencab opencab | gzip > "$BACKUP_FILE"
rclone copy "$BACKUP_FILE" r2:opencab-backups/daily/
rm "$BACKUP_FILE"

# Supprimer les backups de plus de 30 jours
rclone delete r2:opencab-backups/daily/ --min-age 30d

echo "[$(date)] Backup terminé."
```

```bash
chmod +x /opt/scripts/backup-postgres.sh
```

Ajouter le cron (`/etc/cron.d/opencab-backup`) :
```
0 2 * * * root /opt/scripts/backup-postgres.sh >> /var/log/opencab-backup.log 2>&1
```

---

## Récapitulatif des secrets GitHub à mettre à jour

| Secret actuel | Action |
|---|---|
| `GCP_SA_KEY` | Supprimer |
| `SUPABASE_URL` | Supprimer |
| `SUPABASE_SERVICE_ROLE_KEY` | Supprimer |
| `APP__DATABASE__URL` | Mettre à jour vers PostgreSQL Scaleway |
| `SCALEWAY_SSH_KEY` | Ajouter (clé SSH privée pour le déploiement) |
| `R2_ACCESS_KEY` | Ajouter (Cloudflare R2) |
| `R2_SECRET_KEY` | Ajouter (Cloudflare R2) |

## Ordre recommandé

1. [ ] Créer les buckets Cloudflare R2 (`opencab-signatures`, `opencab-backups`)
2. [ ] Mettre en place `docker-compose.yml` sur le serveur Scaleway
3. [ ] Migrer les données Supabase → PostgreSQL local (`pg_dump` / `psql`)
4. [ ] Migrer les fichiers Supabase Storage → Cloudflare R2 (`rclone`)
5. [ ] Modifier le code Rust : Supabase Storage → R2 (S3-compatible)
6. [ ] Mettre à jour le pipeline GitHub Actions (SSH au lieu de `gcloud`)
7. [ ] Configurer les backups automatiques (`rclone` + cron)
8. [ ] Tester en production
9. [ ] Supprimer le projet Supabase et désactiver Cloud Run
