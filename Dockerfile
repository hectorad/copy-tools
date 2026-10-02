# syntax=docker/dockerfile:1
FROM --platform=$BUILDPLATFORM golang:1.27 AS build
ARG TARGETOS TARGETARCH
WORKDIR /src
COPY go.mod main.go ./
RUN CGO_ENABLED=0 GOOS=$TARGETOS GOARCH=$TARGETARCH go build -trimpath -ldflags="-s -w" -o /copy-tools .

# No shell or libc; runs as nonroot (uid 65532).
FROM gcr.io/distroless/static-debian13:nonroot
COPY --from=build /copy-tools /copy-tools
# The binaries to deliver. Swap in your own; they must be static or match the app image's libc.
COPY --from=busybox:1.37-musl /bin/busybox /tools/busybox
ENTRYPOINT ["/copy-tools"]
CMD ["/tools", "/opt/tools"]
