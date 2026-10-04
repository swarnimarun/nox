module.exports = async ({ github, context, core, tag }) => {
  const { owner, repo } = context.repo;
  let release;
  try {
    ({ data: release } = await github.rest.repos.getReleaseByTag({ owner, repo, tag }));
  } catch (error) {
    if (error.status === 404) return;
    throw error;
  }
  if (!release.draft) {
    core.info('Release is already published; preserving it after an ambiguous failure');
    return;
  }
  await github.rest.repos.deleteRelease({ owner, repo, release_id: release.id });
  // GitHub does not create a tag for an unpublished draft.
  core.info(`Removed failed draft ${tag}`);
};
