# syntax=docker/dockerfile:1
# check=skip=SecretsUsedInArgOrEnv
# (VITE_CLERK_PUBLISHABLE_KEY is a public, client-safe key — inlined into the
#  bundle by design, so the secrets-in-ARG heuristic is a false positive here.)

# ---- Build stage ---------------------------------------------------------
FROM node:20-alpine AS builder

WORKDIR /app

# Install deps from the lockfile (includes devDeps: tsc + vite are needed to build).
COPY package.json package-lock.json ./
RUN npm ci

COPY . .

# Vite inlines VITE_* vars at build time, so the Clerk publishable key must be
# supplied as a build arg (it is a public, client-safe key — not a secret).
ARG VITE_CLERK_PUBLISHABLE_KEY
ENV VITE_CLERK_PUBLISHABLE_KEY=${VITE_CLERK_PUBLISHABLE_KEY}

# Runs `tsc --noEmit && vite build` -> static assets in /app/dist.
RUN npm run build

# ---- Runtime stage -------------------------------------------------------
FROM nginx:1.27-alpine AS runtime

# Backend origin nginx proxies /api/ to. The nginx image's entrypoint runs
# envsubst over /etc/nginx/templates/*.template at start, so this is overridable
# at runtime (e.g. `BACKEND_URL` in docker-compose) without rebuilding.
ENV BACKEND_URL=http://backend:8080

# SPA history-fallback + API proxy config, templated with ${BACKEND_URL}.
COPY nginx.conf.template /etc/nginx/templates/default.conf.template
COPY --from=builder /app/dist /usr/share/nginx/html

EXPOSE 80

# nginx:alpine's default CMD already runs nginx in the foreground (after the
# entrypoint expands the templates into /etc/nginx/conf.d/).
