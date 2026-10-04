module.exports = async ({ github, context, core, tag }) => {
  const { owner, repo } = context.repo;
  const { data: current } = await github.rest.repos.getReleaseByTag({ owner, repo, tag });
  const required = ['hyprland', 'niri', 'wsl', 'container'].flatMap((kind) => {
    const extension = { hyprland: 'iso', niri: 'iso', wsl: 'wsl', container: 'oci.tar.gz' }[kind];
    const prefix = `nox-${kind}-x86_64-linux`;
    const image = `${prefix}.${extension}`;
    const direct = current.assets.some((asset) => asset.name === image && asset.size > 0);
    const parts = current.assets.filter((asset) => asset.name.startsWith(`${image}.part-`) && asset.size > 0);
    if (!direct && parts.length < 2) throw new Error(`Missing image or complete parts: ${image}`);
    return [direct ? image : `${image}.parts.json`, `${image}.sha256`, `${prefix}.provenance.json`];
  });
  if (current.draft || required.some((name) => !current.assets.some((asset) => asset.name === name && asset.size > 0))) {
    throw new Error('Refusing cleanup before a complete release is published');
  }
  const releases = await github.paginate(github.rest.repos.listReleases, { owner, repo, per_page: 100 });
  for (const release of releases) {
    if (release.id === current.id || !release.prerelease || !/^v0\.2\.0-alpha\./.test(release.tag_name)) continue;
    await github.rest.repos.deleteRelease({ owner, repo, release_id: release.id });
    core.info(`Removed old Nox preview images: ${release.tag_name}`);
  }
  const artifacts = await github.paginate(github.rest.actions.listArtifactsForRepo, { owner, repo, per_page: 100 });
  for (const artifact of artifacts) {
    // Only image-workflow artifacts are owned by this cleanup.
    if (!artifact.name.startsWith('nox-')) continue;
    await github.rest.actions.deleteArtifact({ owner, repo, artifact_id: artifact.id });
  }
};
