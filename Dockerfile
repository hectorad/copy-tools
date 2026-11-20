# A tools image meant to be mounted as a Kubernetes image volume (see example.yaml).
# Nothing in it ever runs as a container, so it holds only the files to deliver.
FROM scratch
# Swap in your own binaries; they must be static or match the app image's libc.
COPY --from=busybox:1.37-musl /bin/busybox /busybox
