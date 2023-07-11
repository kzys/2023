FROM rust:1.70-alpine AS builder

WORKDIR /root/

COPY md2html /root/md2html

RUN apk add --no-cache musl-dev
RUN cd md2html && cargo build

COPY data /root/data
RUN mkdir html/ && /root/md2html/target/debug/md2html --site-url https://2023.8-p.info /root/data/

FROM nginx:1.25-alpine3.17-slim
COPY --from=builder /root/html /usr/share/nginx/html
