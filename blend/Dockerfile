FROM debian:trixie-slim@sha256:4e401d95de7083948053197a9c3913343cd06b706bf15eb6a0c3ccd26f436a0e

LABEL org.opencontainers.image.source="https://github.com/frantic1048/Vanilla"
LABEL org.opencontainers.image.description="blend — dotfiles manager with Nickel DSL"
LABEL org.opencontainers.image.licenses="MIT"

COPY blend /usr/local/bin/blend
ENTRYPOINT ["/usr/local/bin/blend"]
