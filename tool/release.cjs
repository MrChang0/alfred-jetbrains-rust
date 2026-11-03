// Manual version selection for the pinned Release Please manifest workflow.
const fs = require('node:fs');
const path = require('node:path');
const {createRequire} = require('node:module');
const {execFileSync} = require('node:child_process');

const releaseRequire = createRequire(path.join(process.env.RELEASE_PLEASE_MODULE, 'package.json'));
const {GitHub, Manifest, registerReleaseType} = releaseRequire('release-please');
const {Simple} = releaseRequire('release-please/build/src/strategies/simple');
const {Version} = releaseRequire('release-please/build/src/version');
const semver = releaseRequire('semver');

// Explicit selections also allow promoting a prerelease without new code changes.
registerReleaseType('manual-simple', options => {
  class ManualRelease extends Simple {
    buildReleasePullRequest(commits, latestRelease, draft, labels) {
      return super.buildReleasePullRequest(commits, latestRelease, draft, labels,
        {newVersion: Version.parse(options.releaseAs)});
    }
  }
  return new ManualRelease(options);
});

function selectVersion(current, {kind = 'auto', prereleaseId = 'beta', version = ''} = {}) {
  if (!semver.valid(current) || current.includes('+')) throw Error('Invalid current version');
  if (kind === 'auto') {
    if (version.trim()) throw Error('Select custom to specify an exact version');
    return undefined;
  }
  const kinds = ['patch', 'minor', 'major', 'prepatch', 'preminor', 'premajor', 'prerelease', 'custom'];
  if (!kinds.includes(kind)) throw Error('Unknown release kind');
  if (kind !== 'custom' && version.trim()) throw Error('Select custom to specify an exact version');
  if (kind.startsWith('pre') && !['alpha', 'beta', 'rc'].includes(prereleaseId)) {
    throw Error('Prerelease channel must be alpha, beta, or rc');
  }
  const next = kind === 'custom' ? version.trim().replace(/^v/, '')
    : semver.inc(current, kind, prereleaseId, '1');
  if (!next || semver.valid(next) !== next || next.includes('+')) {
    throw Error('Specify a valid version without build metadata, e.g. 1.0.0-rc.1');
  }
  if (!semver.gt(next, current)) throw Error('The next version must be greater than the current version');
  return next;
}

function configureManualRelease(manifest, current, inputs) {
  const baseline = manifest.releasedVersions?.['.']?.toString();
  if (baseline && baseline !== current) {
    throw Error('The version on main changed while CI ran; start a new manual run');
  }
  const config = manifest.repositoryConfig['.'];
  const selected = selectVersion(current, inputs);
  if (selected) {
    config.releaseAs = selected;
    config.releaseType = 'manual-simple';
  }
  return selected;
}

async function manageRelease(manifest, {eventName, currentVersion, ...inputs}) {
  if (eventName === 'workflow_dispatch') {
    const selected = configureManualRelease(manifest, currentVersion, inputs);
    console.log(`Preparing release PR: ${selected || 'automatic version from commits'}`);
    return {prs: (await manifest.createPullRequests()).filter(Boolean), releases: []};
  }
  if (eventName === 'pull_request') {
    // Stable 0.x versions are normal releases; only a SemVer suffix means preview.
    manifest.repositoryConfig['.'].prerelease = Boolean(semver.prerelease(currentVersion));
    return {prs: [], releases: (await manifest.createReleases()).filter(Boolean)};
  }
  throw Error('Releases require a manual run or a merged version PR');
}

async function main() {
  const [owner, repo] = process.env.GITHUB_REPOSITORY.split('/');
  const github = await GitHub.create({owner, repo, token: process.env.GH_TOKEN,
    defaultBranch: 'main', apiUrl: process.env.GITHUB_API_URL});
  const manifest = await Manifest.fromManifest(github, 'main');
  const root = path.resolve(__dirname, '..');
  const metadata = JSON.parse(execFileSync('cargo', [
    'metadata', '--manifest-path', path.join(root, 'rust/Cargo.toml'),
    '--locked', '--no-deps', '--format-version', '1',
  ], {encoding: 'utf8'}));
  const {prs, releases} = await manageRelease(manifest, {
    eventName: process.env.GITHUB_EVENT_NAME,
    currentVersion: metadata.packages[0].version,
    kind: process.env.RELEASE_KIND || 'auto',
    prereleaseId: process.env.PRERELEASE_ID || 'beta',
    version: process.env.RELEASE_VERSION || '',
  });
  if (prs.length > 1 || releases.length > 1) throw Error('Expected a single root release');
  const outputs = {release_created: String(releases.length > 0)};
  if (prs[0]) outputs.pr = JSON.stringify(prs[0]);
  if (releases[0]) outputs.tag_name = releases[0].tagName;
  for (const [key, value] of Object.entries(outputs)) {
    if (/[\r\n]/.test(value)) throw Error(`Invalid output: ${key}`);
    fs.appendFileSync(process.env.GITHUB_OUTPUT, `${key}=${value}\n`);
  }
}

module.exports = {selectVersion, configureManualRelease, manageRelease};
if (require.main === module) main().catch(error => {
  console.error(`Release failed: ${error.message}`);
  process.exitCode = 1;
});
