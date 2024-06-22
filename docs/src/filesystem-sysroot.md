# Filesystem: Physical /sysroot

The bootc project uses [ostree](https://github.com/ostreedev/ostree/) as a backend,
and maps fetched container images to a [deployment](https://ostreedev.github.io/ostree/deployment/).

## stateroot

The underlying `ostree` CLI and API tooling expose a concept of `stateroot`, which
is not yet exposed via `bootc`.  The `stateroot` used by `bootc install`
is just named `default`.

The stateroot concept allows having fully separate parallel operating
system installations with fully separate `/etc` and `/var`, while
still sharing an underlying root filesystem.

In the future, this functionality will be exposed and used by `bootc`.

## /sysroot mount

When booted, the physical root will be available at `/sysroot` as a
read-only mount point and the logical root `/` will be a bind mount
pointing to a deployment directory under `/sysroot/ostree`.  This is a
key aspect of how `bootc upgrade` operates: it fetches the updated
container image and writes new files to `/sysroot/ostree`.

Beyond that and debugging/introspection, there are few use cases for tooling to
operate on the physical root.

## Expanding the root filesystem

One notable use case that *does* need to operate on `/sysroot`
is expanding the root filesystem.

Some higher level tools such as e.g. `cloud-init` may (reasonably)
expect the `/` mount point to be the physical root.  Tools like
this will need to be adjusted to instead detect this and operate
on `/sysroot`.

### Growing the block device

Fundamentally bootc is agnostic to the underlying block device setup.
How to grow the root block device depends on the underlying
storage stack, from basic partitions to LVM.  However, a
common tool is the [growpart](https://manpages.debian.org/testing/cloud-guest-utils/growpart.1.en.html)
utility from `cloud-init`.

### Growing the filesytem

The systemd project ships a [systemd-growfs](https://www.freedesktop.org/software/systemd/man/latest/systemd-growfs.html#)
tool and corresponding `systemd-growfs@` services.  This is
a relatively thin abstraction over detecting the target
root filesystem type and running the underlying tool such as
`xfs_growfs`.

At the current time, most Linux filesystems require
the target to be mounted writable in order to grow.  Hence,
an invocation of `system-growfs /sysroot` or `xfs_growfs /sysroot`
will need to be further wrapped in a temporary mount namespace.

Using a `MountFlags=slave` drop-in stanza for `systemd-growfs@sysroot.service`
is recommended, along with an `ExecStartPre=mount -o remount,rw /sysroot`.

### Detecting bootc/ostree systems

For tools like `cloud-init` that want to operate generically,
conditionally detecting this situation can be done via e.g.:

- Checking for `/` being an `overlay` mount point
- Checking for `/sysroot/ostree`


