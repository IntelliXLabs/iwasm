FROM golang:1.22-bullseye AS go
FROM rust:1.82.0-bullseye

ENV PATH=$PATH:/usr/local/go/bin
COPY --from=go /usr/local/go /usr/local/go
