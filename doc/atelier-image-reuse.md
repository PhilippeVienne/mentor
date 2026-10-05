# Proposal for Atelier: build an environment once, reuse it for every sandbox

> **Status: proposal written from reading Atelier's code at commit `be66aa8`, 5 October 2026. Nothing was
> changed in Atelier and nothing below was prototyped.** It answers the main gap found by the real trial
> ([execution-plane.md §7](execution-plane.md#7-first-real-trial)): a second sandbox of the same environment
> took 106 s to reach `Running`, 90 s of which rebuilt an image that already existed.

## 1. Why the image is rebuilt today

Four independent reasons, each sufficient on its own:

| # | Where | What happens |
| --- | --- | --- |
| 1 | `crates/controller/src/reconcile.rs`, the `Running` branch of the reconcile loop | The only thing the controller looks at is **this** Workshop's `status.imageDigest`. When it is absent it creates a build Job for this Workshop. It never asks whether another Workshop already built the same source |
| 2 | `crates/image-builder/src/main.rs`, `main` | The image reference is `<registry>/atelier-workshops/<workshop name>:latest`: named after the Workshop, not after what is built |
| 3 | `crates/builder-vm-init/src/main.rs` | `ENVBUILDER_CACHE_REPO` is derived from that same per-Workshop reference, so even envbuilder's layer cache starts empty for every new Workshop |
| 4 | `crates/image-builder/src/main.rs`, `publish_to_cache` | The cache key is the SHA-256 of the finished ext4 file. Two builds of the same source produce different bytes (file-system identifiers, timestamps), hence different keys: the "content-addressed" cache can never recognise a rebuild |

A fifth cost comes after the build and is paid by every start and every resume: `crates/firecracker/src/vm.rs`
declares the rootfs as a **copied** resource (`MovedResourceType::Copied`) and attaches it writable. The trial's
image was 2.5 GiB; copying it is a plausible share of the 16 s between `Provisioning` and `Running`, though
this was not measured separately.

What is **not** an obstacle: nothing secret or per-Workshop is baked into the image. The SSH authorised key and
the session password are fetched at boot from the pod's metadata server (`crates/net-proxy/src/metadata.rs`),
which is exactly what makes an image shareable.

## 2. Proposal

### 2.1 Key the build by its source

Define the **source key** of an image as a digest of everything that determines its content:

- the repository URL, normalised;
- the **commit** the revision resolves to (today's default revision is `HEAD`, which is not a key: it has to be
  resolved, for instance with `git ls-remote`, before any lookup);
- the path of `devcontainer.json`;
- a version of the image builder's own injections (sshd, ttyd, code-server, init, proxy configuration), so that
  changing them invalidates old images;
- what else enters the rootfs from the cluster: the enterprise CA bundle, the guest architecture.

### 2.2 Make the image a resource of its own

Introduce an image object (a custom resource, say `WorkshopImage`, named after the source key) with a phase
(`Building`, `Ready`, `Failed`) and the resulting digest. The reconcile loop changes from "no digest → build for
me" to:

1. resolve the revision, compute the source key;
2. get or create the image object for that key;
3. `Ready` → copy its digest into `status.imageDigest` and go on to provisioning;
4. `Building` → wait (requeue);
5. it does not exist → create it, and with it the single build Job, **owned by the image, not by a Workshop**.

This also settles concurrency: thirty sandboxes of the same environment created in the same minute, which is a
class starting a lab, wait on one build instead of starting thirty.

The existing pieces keep their role: the PVC and S3 still store files by output digest, and the eviction pass
(`crates/controller/src/eviction.rs`) gains an obvious rule, an image is evictable when no image object refers
to it or when it has not been used for some time.

### 2.3 Name the registry artefacts by source key

Push to `<registry>/atelier-images/<source key>` and use one shared `ENVBUILDER_CACHE_REPO`. Independently of
2.2, this alone lets a *different* revision of the same repository reuse unchanged layers.

### 2.4 Stop copying the rootfs for every VM

Attach the image read-only and give each VM a small writable layer (a second drive used as an overlay upper
directory, or a copy-on-write clone where the storage supports it). Start-up no longer depends on the image
size, and the writable layer's size becomes the session's disk quota. This is the larger change: it touches
the guest's boot sequence and the snapshot format, and deserves its own design.

## 3. What it would give

| | Today (measured) | With 2.1 to 2.3 | With 2.4 as well |
| --- | --- | --- | --- |
| First sandbox of an environment | 106 s | unchanged | unchanged |
| Next sandboxes, same commit | 106 s | about 16 s (no build) | to measure; the copy of the image disappears |
| A burst of sandboxes of one environment | one build each | one build in total | one build in total |

The 16 s is the trial's own figure for provisioning a sandbox whose image exists. Going well below that needs
2.4 and then template snapshots (see [execution-plane.md §6](execution-plane.md#6-what-the-removed-prototype-measured)).

## 4. Things to decide in Atelier

1. **Moving revisions.** A Workshop on `main` should probably keep its image until it is recreated or asked to
   refresh, not rebuild whenever the branch moves. That is a product choice for coding agents, and the opposite
   of what a course wants (pin a commit, change it deliberately).
2. **Build-time egress.** The build goes through the Workshop's `egressAllowlist`. With a shared image, whose
   allow-list applies to the build? An image-level build policy would also answer Mentor's need for labs that
   have network at build time and none at run time.
3. **Private repositories.** Build credentials are resolved per Workshop today (`resolve_git_credentials`). A
   shared image must not let one group reuse an image built with another group's access: the owner group
   probably belongs in the source key, or in an access check on the image object.
4. **The workspace clone.** The builder clones the target repository into the image (`ensure_workspace_clone`)
   and installs a refresh at boot. That stays correct with a commit-keyed image, and should be checked.
5. **Reproducible ext4.** Making the output digest stable (fixed file-system UUID and timestamps) would let
   the existing content-addressed cache deduplicate too. Useful, but not needed once builds are keyed by source.

## 5. Suggested order

1. 2.3, the registry naming: small, isolated, immediately useful.
2. 2.1 and 2.2, the source key and the image object: the change that removes the rebuild.
3. Measure again with the same trial.
4. 2.4, as a separate design.
