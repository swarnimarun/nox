const test = require('node:test');
const assert = require('node:assert/strict');
const retain = require('../../.github/scripts/cache-retention.cjs');
const cleanup = require('../../.github/scripts/release-cleanup.cjs');
const context = { repo: { owner: 'owner', repo: 'nox' } };
const core = { info() {} };

test('retention keeps one latest snapshot across refs and preserves unrelated caches', async () => {
  const deleted = [];
  const list = [
    { id: 1, key: 'nox-build-v1-old', created_at: '2026-01-01', ref: 'refs/heads/main' },
    { id: 2, key: 'nox-build-v1-new', created_at: '2026-02-01', ref: 'refs/heads/main' },
    { id: 3, key: 'nox-build-v0-old', created_at: '2026-01-01', ref: 'refs/pull/10/merge' },
    { id: 4, key: 'unrelated', created_at: '2026-03-01' },
  ];
  const github = {
    paginate: async () => list,
    rest: { actions: { getActionsCacheList() {}, deleteActionsCacheById: async ({ cache_id }) => deleted.push(cache_id) } },
  };
  await retain({ github, context, core });
  assert.deepEqual(deleted.sort(), [1, 3]);
});

test('empty cache list is safe', async () => {
  await retain({ github: { paginate: async () => [], rest: { actions: {} } }, context, core });
});

function releaseFixture({ draft = false, incomplete = false } = {}) {
  const assets = ['hyprland', 'niri', 'wsl', 'container'].flatMap((kind) => {
    const ext = { hyprland: 'iso', niri: 'iso', wsl: 'wsl', container: 'oci.tar.gz' }[kind];
    return [`nox-${kind}-x86_64-linux.${ext}`, `nox-${kind}-x86_64-linux.${ext}.sha256`, `nox-${kind}-x86_64-linux.provenance.json`].map((name) => ({ name, size: 1 }));
  });
  if (incomplete) assets.pop();
  const current = { id: 5, tag_name: 'v0.2.0-alpha.5', draft, prerelease: true, assets };
  const deleted = [];
  const repos = {
    getReleaseByTag: async () => ({ data: current }),
    listReleases() {},
    deleteRelease: async ({ release_id }) => deleted.push(['release', release_id]),
  };
  const actions = { listArtifactsForRepo() {}, deleteArtifact: async ({ artifact_id }) => deleted.push(['artifact', artifact_id]) };
  const github = {
    rest: { repos, actions },
    paginate: async (method) => method === repos.listReleases ? [
      current,
      { id: 1, prerelease: true, tag_name: 'v0.2.0-alpha.1' },
      { id: 2, prerelease: false, tag_name: 'v0.2.0' },
      { id: 3, prerelease: true, tag_name: 'unrelated-preview' },
    ] : [{ id: 1, name: 'nox-old-image' }, { id: 2, name: 'other-project' }],
  };
  return { github, deleted };
}

for (const state of [{ draft: true }, { incomplete: true }]) {
  test(`cleanup rejects ${JSON.stringify(state)} without deleting anything`, async () => {
    const { github, deleted } = releaseFixture(state);
    await assert.rejects(cleanup({ github, context, core, tag: 'v0.2.0-alpha.5' }));
    assert.deepEqual(deleted, []);
  });
}

test('cleanup only deletes old Nox previews and Nox artifacts after complete publication', async () => {
  const { github, deleted } = releaseFixture();
  await cleanup({ github, context, core, tag: 'v0.2.0-alpha.5' });
  assert.deepEqual(deleted, [['release', 1], ['artifact', 1]]);
});

test('cleanup accepts a complete multipart image with its manifest and checksums', async () => {
  const { github, deleted } = releaseFixture();
  const { data: current } = await github.rest.repos.getReleaseByTag();
  const name = 'nox-hyprland-x86_64-linux.iso';
  current.assets = current.assets.filter((asset) => asset.name !== name);
  current.assets.push(...[`${name}.parts.json`, `${name}.part-000`, `${name}.part-001`].map((name) => ({ name, size: 1 })));
  await cleanup({ github, context, core, tag: 'v0.2.0-alpha.5' });
  assert.deepEqual(deleted, [['release', 1], ['artifact', 1]]);
});

test('cleanup rejects missing image parts before removing old releases', async () => {
  const { github, deleted } = releaseFixture();
  const { data: current } = await github.rest.repos.getReleaseByTag();
  const name = 'nox-hyprland-x86_64-linux.iso';
  current.assets = current.assets.filter((asset) => asset.name !== name);
  current.assets.push(...[`${name}.parts.json`, `${name}.part-000`].map((name) => ({ name, size: 1 })));
  await assert.rejects(cleanup({ github, context, core, tag: 'v0.2.0-alpha.5' }));
  assert.deepEqual(deleted, []);
});
