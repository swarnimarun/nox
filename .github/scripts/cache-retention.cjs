module.exports = async ({ github, context, core }) => {
  const { owner, repo } = context.repo;
  const caches = await github.paginate(github.rest.actions.getActionsCacheList, {
    owner, repo, per_page: 100,
  });
  const owned = caches.filter((cache) => cache.key.startsWith('nox-build-'));
  owned.sort((a, b) => b.created_at.localeCompare(a.created_at) || b.id - a.id);
  for (const cache of owned.slice(1)) {
    await github.rest.actions.deleteActionsCacheById({ owner, repo, cache_id: cache.id });
  }
  core.info(`Retained ${Math.min(owned.length, 1)} Nox cache; removed ${Math.max(owned.length - 1, 0)}`);
};
