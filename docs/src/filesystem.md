# Filesystem

As noted in other chapters, the bootc project inherits
a lot of code from the [ostree project](https://github.com/ostreedev/ostree/).

However, bootc is intending to be a "fresh, new container-native interface".

First, it is strongly recommended that bootc consumers use the ostree
[composefs backend](https://ostreedev.github.io/ostree/composefs/); to do this,
ensure that you have a `/usr/lib/ostree/prepare-root.conf` that contains at least

```ini
[composefs]
enabled = true
```

This will ensure that the entire `/` is a read-only filesystem.

## Understanding container build/runtime vs deployment

When run *as a container* (e.g. as part of a container build), the
filesystem is fully mutable in order to allow derivation to work.
For more on container builds, see [build guidance](building/guidance.md).

The rest of this document describes the state of the system when
"deployed" to a physical or virtual machine, and managed by `bootc`.

## Timestamps

bootc uses ostree, which currently [squashes all timestamps to zero](https://ostreedev.github.io/ostree/repo/#content-objects).
This is now viewed as an implementation bug and will be changed in the future.
For more information, see [this tracker issue](https://github.com/containers/bootc/issues/20).

## Understanding physical vs logical root with `/sysroot`

When the system is fully booted, it is into the equivalent of a `chroot`.
The "physical" host root filesystem will be mounted at `/sysroot`.
For more on this, see [filesystem: sysroot](filesystem-sysroot.md).

This `chroot` filesystem is called a "deployment root". All the remaining
filesystem paths below are part of a deployment root which is used as a
final target for the system boot.  The target deployment is determined
via the `ostree=` kernel commandline argument.

## `/usr`

The overall recommendation is to keep all operating system content in `/usr`,
with directories such as `/bin` being symbolic links to `/usr/bin`, etc.
See [UsrMove](https://fedoraproject.org/wiki/Features/UsrMove) for example.

However, with composefs enabled `/usr` is not different from `/`;
they are part of the same immutable image.  So there is not a fundamental
need to do a full "UsrMove" with a bootc system.

### `/usr/local`

The OSTree upstream recommendation suggests making `/usr/local` a symbolic
link to `/var/usrlocal`. But because the emphasis of a bootc-oriented system is
on users deriving custom container images as the default entrypoint,
it is recommended here that base images configure `/usr/local` be a regular
directory (i.e. the default).

Projects that want to produce "final" images that are themselves
not intended to be derived from in general can enable that symbolic link
in derived builds.

## `/etc`

The `/etc` directory contains mutable persistent state by default; however,
it is suppported to enable the [`etc.transient` config option](https://ostreedev.github.io/ostree/man/ostree-prepare-root.html).

When in persistent mode, it inherits the OSTree semantics of [performing a 3-way merge](https://ostreedev.github.io/ostree/atomic-upgrades/#assembling-a-new-deployment-directory)
across upgrades.  In a nutshell:

- The *new default* `/etc` is used as a base
- The diff between current and previous `/etc` is applied to the new `/etc`
- Locally modified files in `/etc` different from the default `/usr/etc` (of the same deployment) will be retained

The implmentation of this defaults to being executed by `ostree-finalize-staged.service`
at shutdown time, before the new bootloader entry is created.

The rationale for this design is that in practice today, many components of a Linux system end up shipping
default configuration files in `/etc`.  And even if the default package doesn't, often the software
only looks for config files there by default.

Some other image-based update systems do not have distinct "versions" of `/etc` and
it may be populated only set up at a install time, and untouched thereafter.  But
that creates "hysteresis" where the state of the system's `/etc` is strongly
influenced by the initial image version.  This can lead to problems
where e.g. a change to `/etc/sudoers.conf` (to give on simple example)
would require external intervention to apply.

For more on configuration file best practices, see [Building](building/guidance.md).

## `/var`

Content in `/var` persists by default; it is however supported to make it or subdirectories
mount points (whether network or `tmpfs`).  There is exactly one `/var`.  If it is
not a distinct partition, then "physically" currently it is a bind mount into
`/ostree/deploy/$stateroot/var` and shared across "deployments" (bootloader entries).

As of OSTree v2024.3, by default [content in /var acts like a Docker VOLUME /var](https://github.com/ostreedev/ostree/pull/3166/commits/f81b9fa1666c62a024d5ca0bbe876321f72529c7).

This means that the content from the container image is copied at initial installation time, and *not updated thereafter*.

Note this is very different from the handling of `/etc`.   The rationale for this is
that `/etc` is relatively small configuration files, and the expected configuration
files are often bound to the operating system binaries in `/usr`.

But `/var` has arbitrarily large data (system logs, databases, etc.).  It would
also not be expected to be rolled back if the operating system state is rolled
back.  A simple example is that an `apt|dnf downgrade postgresql` should not
affect the physical database in general in `/var/lib/postgres`.  Similarly,
a bootc update or rollback should not affect this application data.

Having `/var` separate also makes it work cleanly to "stage" new
operating system updates before applying them (they're downloaded
and ready, but only take effect on reboot).

In general, this is the same rationale for Docker `VOLUME`: decouple the application
code from its data.

A common case is for applications to want some directory structure (e.g. `/var/lib/postgresql`) to be pre-created.
It's recommended to use [systemd tmpfiles.d](https://www.freedesktop.org/software/systemd/man/latest/tmpfiles.d.html)
for this.  An even better approach where applicable is [StateDirectory=](https://www.freedesktop.org/software/systemd/man/latest/systemd.exec.html#RuntimeDirectory=)
in units.

## Other directories

It is not supported to ship content in `/run` or `/proc` or other [API Filesystems](https://www.freedesktop.org/wiki/Software/systemd/APIFileSystems/) in container images.

Besides those, for other toplevel directories such as `/usr` `/opt`, they will be lifecycled with the container image.

### `/opt`

In the default suggested model of using composefs (per above) the `/opt` directory will be read-only, alongside
other toplevels such as `/usr`.

Some software (especially "3rd party" deb/rpm packages) expect to be able to write to
a subdirectory of `/opt` such as `/opt/examplepkg`.

See [building images](building/guidance.md) for recommendations on how to build
container images and adjust the filesystem for cases like this.

However, for some use cases, it may be easier to allow some level of mutability.
There are two options for this, each with separate trade-offs: transient roots
and state overlays.

#### Enabling transient root

This feature enables a fully transient writable rootfs by default.
To do this, set the

```toml
[root]
transient = true
```

option in `prepare-root.conf`.  In particular this will allow software to
write (transiently, i.e. until the next reboot) to all top-level directories,
including `/usr` and `/opt`, with symlinks to `/var` for content that should
persist.

#### Enabling state overlays

This feature enables a writable overlay on top of `/opt` (or really, any
toplevel or subdirectory baked into the image that is normally read-only).
Changes persist across reboots but during updates, new files from the container
image override any locally modified version. All other files persist.

To enable this feature, simply instantiate the `ostree-state-overlay@.service`
unit template on the target path. For example, for `/opt`:

```
RUN systemctl enable ostree-state-overlay@opt.service
```
