FROM node:20-alpine as base
WORKDIR /app
COPY package*.json ./
RUN npm install

FROM base as prod
COPY . .
RUN npm run build
CMD ["npm", "start"]
    
FROM base as dev
COPY . .
EXPOSE 3001
CMD ["npm", "run", "dev"]