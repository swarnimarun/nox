const test = require('node:test');
const assert = require('node:assert/strict');
const cleanup = require('../../.github/scripts/failed-draft.cjs');
const context = { repo: { owner: 'owner', repo: 'nox' } };
const core = { info() {} };

test('draft deletion works without a tag and only removes the draft release', async () => {
  const deleted = [];
  const github = { rest: { repos: {
    getReleaseByTag: async () => ({ data: { id: 1, draft: true } }),
    deleteRelease: async ({ release_id }) => deleted.push(release_id),
  } } };
  await cleanup({ github, context, core, tag: 'preview' });
  assert.deepEqual(deleted, [1]);
});

test('a published release is preserved after an ambiguous publishing error', async () => {
  const github = { rest: { repos: {
    getReleaseByTag: async () => ({ data: { id: 1, draft: false } }),
    deleteRelease: async () => assert.fail('must preserve published release'),
  } } };
  await cleanup({ github, context, core, tag: 'preview' });
});

test('missing draft is already clean; other API errors propagate', async () => {
  const github = { rest: { repos: {
    getReleaseByTag: async () => { throw { status: 404 }; },
  } } };
  await cleanup({ github, context, core, tag: 'preview' });
  github.rest.repos.getReleaseByTag = async () => { throw new Error('API failure'); };
  await assert.rejects(cleanup({ github, context, core, tag: 'preview' }), /API failure/);
});
