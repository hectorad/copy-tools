FROM --platform=$BUILDPLATFORM rust:1-alpine AS build
ARG TARGETARCH
WORKDIR /src
COPY main.rs .
RUN rustc --test -o /test main.rs && /test
RUN case "$TARGETARCH" in \
      amd64) target=x86_64-unknown-linux-musl ;; \
      arm64) target=aarch64-unknown-linux-musl ;; \
      *) echo "unsupported architecture: $TARGETARCH" >&2; exit 1 ;; \
    esac \
 && rustup target add "$target" \
 && rustc --target "$target" -C linker=rust-lld -C opt-level=s -C panic=abort -C strip=symbols -o /copy-tools main.rs

# scratch holds no OS packages, shell, or libc, so the image is only the files copied below.
FROM scratch
COPY --from=build /copy-tools /copy-tools
# The binaries to deliver. Swap in your own; they must be static or match the app image's libc.
COPY --from=busybox:1.37-musl /bin/busybox /tools/busybox
USER 65532:65532
ENTRYPOINT ["/copy-tools"]
CMD ["/tools", "/opt/tools"]
