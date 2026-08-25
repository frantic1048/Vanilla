FROM debian:trixie-slim@sha256:d7e12182ce18b85b93007c1dedf31f2d29e01ccf3182cc4017c709b6259bc132

LABEL org.opencontainers.image.source="https://github.com/frantic1048/Vanilla"
LABEL org.opencontainers.image.title="blend"
LABEL org.opencontainers.image.description="blend: dotfiles manager with Nickel DSL"
LABEL org.opencontainers.image.url="https://github.com/frantic1048/Vanilla/tree/master/blend"
LABEL org.opencontainers.image.documentation="https://github.com/frantic1048/Vanilla/tree/master/blend#readme"
LABEL org.opencontainers.image.licenses="MIT"

COPY blend /usr/local/bin/blend
ENTRYPOINT ["/usr/local/bin/blend"]
