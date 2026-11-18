FROM node:24-alpine AS runtime
WORKDIR /app
COPY package*.json ./
RUN npm install --omit=dev
COPY src ./src
EXPOSE 8085
HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
  CMD wget -q --spider http://127.0.0.1:8085/health || exit 1
CMD ["npm", "start"]
