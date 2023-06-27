FROM golang:1.20-alpine AS builder

WORKDIR /root/

COPY md2html /root/md2html
COPY data /root/data


RUN cd md2html && go build
RUN mkdir html/ && /root/md2html/md2html /root/data/

FROM nginx:1.25-alpine3.17-slim
COPY --from=builder /root/html /usr/share/nginx/html
