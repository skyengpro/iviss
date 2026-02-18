# Base stage
FROM node:20-alpine AS base
WORKDIR /app
COPY frontend/package.json frontend/package-lock.json ./
COPY openapi.json /openapi.json

# Dependencies stage
FROM base AS deps
RUN npm ci

# Development stage
FROM base AS dev
COPY --from=deps /app/node_modules ./node_modules
COPY frontend/ .
# Expose default Vite port
EXPOSE 8080
CMD ["npm", "run", "dev", "--", "--host"]

# Build stage
FROM base AS builder
COPY --from=deps /app/node_modules ./node_modules
COPY frontend/ .
RUN npm run build

# Production stage
FROM nginx:alpine AS prod
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
