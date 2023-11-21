---
nav_order: 4
---

# Frequently Asked Questions

## How do users include their own packages/binaries in a custom "bootc compatible" container?

The "bootc compatible" containers are OCI container images, so you can customize them in the same way you build containers today. This means using a Containerfile to customize your image and build tools like `buildah`, `podman build`, or `docker build` to generate your customized "bootc compatible" container image.

For examples of how use build a "bootc compatible" base image, see the [centos-boot repo](https://github.com/CentOS/centos-boot) as a starting point. For examples of how to use a Containerfile to build a customized "bootc compatible" image, see the [centos-boot-layered repo](https://github.com/CentOS/centos-boot-layered).

## How does the use of OCI artifacts intersect with this effort?

The "bootc compatible" images are OCI container images; they do not rely on the [OCI artifact specification](https://github.com/opencontainers/image-spec/blob/main/artifacts-guidance.md) or [OCI referrers API](https://github.com/opencontainers/distribution-spec/blob/main/spec.md#enabling-the-referrers-api).

It is foreseeable that users will need to produce "traditional" disk images (i.e. raw disk images, qcow2 disk images, Amazon AMIs, etc.) from the "bootc compatible" container images using additional tools. Therefore, it is reasonable that some users may want to encapsulate those disk images as an OCI artifact for storage and distribution. However, it is not a goal to use `bootc` to produce these "traditional" disk images nor to facilitate the encapsulation of those disk images as OCI artifacts.
