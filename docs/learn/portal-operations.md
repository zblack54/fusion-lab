# Keeping the knowledge portal useful

The portal is built from version-controlled Markdown in `docs/`. It is prepared
for local use. Public GitHub Pages publication is opt-in and is not triggered by
merging a documentation PR.

## Read locally

From the repository, run:

```sh
python scripts/docs.py serve
```

The helper prepares the pinned documentation environment and prints the local
preview URL. To check a change without a server, run
`python scripts/docs.py build`; the strict build writes `site/`.

Start with the [15-minute path](index.md), then use the
[math/code tour](math-and-code.md) and [research guide](research.md). Generate
the current Rust API reference with `cargo doc --locked --no-deps --open`.

## Transfer knowledge after an iteration

Each meaningful behavior, mathematical, architectural or research change gets a
short [knowledge handoff](iteration-handoff.md) linked from its issue and PR.
It should answer: what changed, why, how equations map to code, what to run,
what the evidence actually shows, and what remains uncertain. Include one
checkable exercise for substantial changes. Routine maintenance can use a brief
rationale and verification note.

Keep the learning path focused on implemented behavior. Put raw execution logs
and detailed verification catalogs behind the concise explanation. Preserve
negative experiment results and the protocol chosen before looking at data.
Update the [roadmap](../roadmap.md) and milestone status as work is verified.

## Optional GitHub Pages hosting

The repository includes `.github/workflows/docs-pages.yml`. It has no push or
pull-request deployment trigger. After the owner chooses public publication:

1. In repository **Settings → Pages**, select **GitHub Actions** as the source.
2. In **Actions → Publish documentation portal**, select the `main` branch and
   explicitly set `publish` to true.
3. Inspect the build and deployment result. The `github-pages` environment
   records the actual published URL; a successful local build alone does not
   establish that hosting is active.

The workflow builds the strict MkDocs site and Rust API documentation, placing
the latter under `api/`. Publishing documentation does not deploy the experiment
service or grant it remote access. This preparation uses the existing repository
and GitHub Pages; it does not configure paid model APIs or paid hosting.

See [GitHub's custom Pages workflow guide](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages)
for the hosting model and deployment permissions. Once published, future portal
updates can use the same explicit dispatch after their reviewed PRs merge.
