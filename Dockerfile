FROM rust:alpine as builder

COPY . /location-rs

RUN apk update \
  && apk add git make build-base pkgconfig
RUN rustup target list --installed
RUN cd /location-rs \
  && make release 

FROM alpine 

EXPOSE 7001

# tzdata 安装所有时区配置或可根据需要只添加所需时区

RUN addgroup -g 1000 rust \
  && adduser -u 1000 -G rust -s /bin/sh -D rust \
  && apk add --no-cache ca-certificates tzdata

COPY --from=builder /location-rs/target/release/location /usr/local/bin/location
COPY --from=builder /location-rs/entrypoint.sh /entrypoint.sh

USER rust

WORKDIR /home/rust

HEALTHCHECK --timeout=10s --interval=10s CMD [ "wget", "http://127.0.0.1:7001/ping", "-q", "-O", "-"]

CMD ["location"]

ENTRYPOINT ["/entrypoint.sh"]