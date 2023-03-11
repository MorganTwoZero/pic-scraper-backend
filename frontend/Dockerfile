FROM node:lts-alpine as frontend_builder

WORKDIR /app

COPY ./babel.config.js .
COPY ./jsconfig.json .
COPY ./vue.config.js .

COPY package*.json ./
RUN npm install

FROM frontend_builder as frontend

COPY ./src ./src

ARG VUE_APP_BACKEND_URL
ENV VUE_APP_BACKEND_URL=$VUE_APP_BACKEND_URL

RUN npm run build

FROM nginx:alpine

COPY --from=frontend /app/dist /app