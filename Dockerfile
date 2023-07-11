FROM rust:1.70 AS builder

WORKDIR /root/

COPY md2html /root/md2html

RUN cd md2html && cargo build

COPY data /root/data
RUN mkdir html/ && /root/md2html/target/debug/md2html /root/data/

FROM nginx:1.25-alpine3.17-slim
COPY --from=builder /root/html /usr/share/nginx/html
