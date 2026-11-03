// Exercise the real release updaters against read-only repository fixtures.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {createRequire} = require('node:module');
const {test} = require('node:test');
const vm = require('node:vm');
const os = require('node:os');
const {execFileSync} = require('node:child_process');

const root = path.resolve(__dirname, '..');
const releaseRequire = createRequire(path.join(process.env.RELEASE_PLEASE_MODULE, 'package.json'));
const {GitHub, Manifest, setLogger} = releaseRequire('release-please');
const {buildStrategy} = releaseRequire('release-please/build/src/factory');
const {parseConventionalCommits} = releaseRequire('release-please/build/src/commit');
const {TagName} = releaseRequire('release-please/build/src/util/tag-name');
const toml = releaseRequire('@iarna/toml');
const yaml = releaseRequire('yaml');
const semver = releaseRequire('semver');
const {selectVersion, configureManualRelease, manageRelease} = require('./release.cjs');
const read = file => fs.readFileSync(path.join(root, file), 'utf8');
const cargo = toml.parse(read('rust/Cargo.toml'));
const current = cargo.package.version;
const {major, minor, patch} = semver.parse(current);
const suffix = semver.prerelease(current) ? `-${semver.prerelease(current).join('.')}` : '';
const nextMinor = `${major}.${minor + 1}.0${suffix}`;
const nextPatch = `${major}.${minor}.${patch + 1}${suffix}`;
const logger = Object.fromEntries(['info', 'warn', 'error', 'debug', 'trace'].map(key => [key, () => {}]));
setLogger(logger);

async function releaseCandidate(message, file, manualInputs, base = current) {
  const repository = new URL(cargo.package.repository).pathname.slice(1).split('/');
  const github = {
    repository: {owner: repository[0], repo: repository[1], defaultBranch: 'main'},
    getFileJson: async file => file === '.release-please-manifest.json'
      ? {'.': base} : JSON.parse(read(file)),
  };
  const manifest = await Manifest.fromManifest(github, 'main');
  // This workflow and its installer are one product, including root-level files.
  assert.deepEqual(Object.keys(manifest.repositoryConfig), ['.']);
  if (manualInputs) configureManualRelease(manifest, base, manualInputs);
  const strategy = await buildStrategy({
    ...manifest.repositoryConfig['.'], github, targetBranch: 'main', path: '.',
  });
  const commits = parseConventionalCommits(message ? [{
    sha: 'a'.repeat(40), message, files: [file],
  }] : []);
  const candidate = await strategy.buildReleasePullRequest(commits, {
    tag: TagName.parse(`v${base}`), sha: 'b'.repeat(40), notes: '',
  });
  return {candidate, strategy};
}

test('Cargo, lockfile, release manifest, and version marker agree', () => {
  const lock = toml.parse(read('rust/Cargo.lock'));
  assert.equal(lock.package.find(item => item.name === cargo.package.name).version, current);
  assert.equal(JSON.parse(read('.release-please-manifest.json'))['.'], current);
  assert.equal(read('version.txt').trim(), current);
  assert.match(read('CHANGELOG.md'), new RegExp(`^## \\[?${current.replaceAll('.', '\\.')}\\]?`, 'm'));
});

for (const file of ['rust/src/install.rs', 'install.sh']) {
  test(`features in ${file} produce a separate minor release PR`, async () => {
    const {candidate} = await releaseCandidate('feat: improve workflow updates', file);
    assert.ok(candidate);
    assert.equal(candidate.version.toString(), nextMinor);
    assert.equal(candidate.title.toString(), `chore(main): release ${nextMinor}`);
    assert.deepEqual(candidate.updates.map(update => update.path).sort(),
      ['CHANGELOG.md', 'version.txt', 'rust/Cargo.toml', 'rust/Cargo.lock'].sort());
  });
}

test('installer fixes produce patch releases', async () => {
  const {candidate} = await releaseCandidate('fix: handle an installer failure', 'install.sh');
  assert.equal(candidate.version.toString(), nextPatch);
});

test('release updaters change only the crate version and preserve dependencies', async () => {
  const {candidate} = await releaseCandidate('feat: improve workflow updates', 'rust/src/install.rs');
  for (const file of ['rust/Cargo.toml', 'rust/Cargo.lock']) {
    const original = read(file);
    const update = candidate.updates.find(update => update.path === file);
    const actual = update.updater.updateContent(original, logger);
    const before = toml.parse(original);
    const after = toml.parse(actual);
    if (file.endsWith('.lock')) {
      after.package.find(item => item.name === cargo.package.name).version = current;
    } else {
      after.package.version = current;
    }
    assert.deepEqual(after, before);
    assert.ok(actual !== original, `${file} must receive a version bump`);
    const expected = file.endsWith('.lock')
      ? original.replace(`name = "${cargo.package.name}"\nversion = "${current}"`,
        `name = "${cargo.package.name}"\nversion = "${nextMinor}"`)
      : original.replace(`version = "${current}"`, `version = "${nextMinor}"`);
    assert.equal(actual, expected);
  }
  const changelog = candidate.updates.find(update => update.path === 'CHANGELOG.md');
  const updated = changelog.updater.updateContent(read('CHANGELOG.md'), logger);
  assert.ok(updated.includes(nextMinor));
  assert.ok(updated.includes(read('CHANGELOG.md').split('\n').slice(1).join('\n')));
  const marker = candidate.updates.find(update => update.path === 'version.txt');
  assert.equal(marker.updater.updateContent(read('version.txt')), `${nextMinor}\n`);
});

for (const message of ['docs: clarify configuration', 'chore: update tooling']) {
  test(`${message} alone does not create a release PR`, async () => {
    const {candidate} = await releaseCandidate(message, 'README.md');
    assert.equal(candidate, undefined);
  });
}

test('a merged version PR tags its own commit without a component prefix', async () => {
  const {candidate, strategy} = await releaseCandidate('fix: handle an installer failure', 'install.sh');
  const sha = 'c'.repeat(40);
  assert.equal(candidate.headRefName.toString(), 'release-please--branches--main');
  const release = await strategy.buildRelease({
    title: candidate.title.toString(), body: candidate.body.toString(),
    headBranchName: candidate.headRefName.toString(), baseBranchName: 'main',
    number: 1, labels: ['autorelease: pending'], files: [], sha,
  });
  assert.equal(release.tag.toString(), `v${nextPatch}`);
  assert.equal(release.sha, sha);
});

const workflow = yaml.parse(read('.github/workflows/release-please.yml'));
const repository = 'example/project';
const versionPR = {
  merged: true,
  title: 'chore(main): release 0.3.0',
  head: {ref: 'release-please--branches--main', repo: {full_name: repository}},
  labels: [{name: 'autorelease: pending'}],
};

// Evaluate this workflow's boolean guard against event fixtures. Expand the
// one Actions object filter to its equivalent JavaScript array projection.
function evaluateCondition(expression, github) {
  const source = expression.trim().replace(/^\$\{\{\s*|\s*\}\}$/g, '')
    .replace('github.event.pull_request.labels.*.name',
      'github.event.pull_request.labels.map(label => label.name)');
  return vm.runInNewContext(source, {
    github,
    startsWith: (value, prefix) => value.startsWith(prefix),
    contains: (values, value) => values.includes(value),
  });
}

test('release initiation has no push trigger and finalizes only closed PRs on main', () => {
  assert.equal(workflow.on.push, undefined);
  assert.ok(Object.hasOwn(workflow.on, 'workflow_dispatch'));
  assert.deepEqual(workflow.on.pull_request, {branches: ['main'], types: ['closed']});
});

for (const [name, event, ref, modifyPR, allowed] of [
  ['manual main run', 'workflow_dispatch', 'refs/heads/main', () => {}, true],
  ['manual feature branch run', 'workflow_dispatch', 'refs/heads/feature', () => {}, false],
  ['ordinary main push', 'push', 'refs/heads/main', () => {}, false],
  ['merged version PR', 'pull_request', 'refs/heads/main', () => {}, true],
  ['closed unmerged version PR', 'pull_request', 'refs/heads/main', pr => {pr.merged = false;}, false],
  ['merged feature PR', 'pull_request', 'refs/heads/main', pr => {pr.head.ref = 'feature';}, false],
  ['fork PR', 'pull_request', 'refs/heads/main', pr => {pr.head.repo.full_name = 'fork/project';}, false],
  ['PR with another title', 'pull_request', 'refs/heads/main', pr => {pr.title = 'feat: update';}, false],
  ['PR without the release label', 'pull_request', 'refs/heads/main', pr => {pr.labels = [];}, false],
]) {
  test(`${name}: release flow is ${allowed ? 'allowed' : 'skipped'}`, () => {
    const pr = structuredClone(versionPR);
    modifyPR(pr);
    const github = {repository, event_name: event, ref,
      event: event === 'pull_request' ? {pull_request: pr} : {}};
    assert.equal(evaluateCondition(workflow.jobs.test.if, github), allowed);
  });
}

test('manual runs prepare PRs and merge events publish without preparing another PR', async () => {
  for (const event of ['workflow_dispatch', 'pull_request']) {
    const calls = [];
    const manifest = {repositoryConfig: {'.': {}},
      createPullRequests: async () => {calls.push('prepare'); return [];},
      createReleases: async () => {calls.push('publish'); return [];}};
    await manageRelease(manifest, {eventName: event, currentVersion: current});
    assert.deepEqual(calls, [event === 'workflow_dispatch' ? 'prepare' : 'publish']);
  }
});

for (const [base, kind, prereleaseId, version, expected] of [
  ['0.2.0', 'auto', 'beta', '', undefined],
  ['0.2.0', 'patch', 'beta', '', '0.2.1'],
  ['0.2.0', 'minor', 'beta', '', '0.3.0'],
  ['0.2.0', 'major', 'beta', '', '1.0.0'],
  ['0.2.0', 'prepatch', 'alpha', '', '0.2.1-alpha.1'],
  ['0.2.0', 'preminor', 'beta', '', '0.3.0-beta.1'],
  ['0.2.0', 'premajor', 'rc', '', '1.0.0-rc.1'],
  ['0.2.0', 'prerelease', 'beta', '', '0.2.1-beta.1'],
  ['0.3.0-beta.1', 'prerelease', 'beta', '', '0.3.0-beta.2'],
  ['0.3.0-beta.2', 'prerelease', 'rc', '', '0.3.0-rc.1'],
  ['0.3.0-beta.2', 'patch', 'beta', '', '0.3.0'],
  ['0.2.0', 'custom', 'beta', 'v2.1.0-rc.3', '2.1.0-rc.3'],
]) {
  test(`${kind} selects ${expected || 'automatic'} from ${base}`, () => {
    assert.equal(selectVersion(base, {kind, prereleaseId, version}), expected);
  });
}

for (const inputs of [
  {kind: 'custom', version: ''},
  {kind: 'custom', version: '0.2.0'},
  {kind: 'custom', version: '0.1.0'},
  {kind: 'custom', version: '1.0.0+build'},
  {kind: 'custom', version: '1.0.0/other'},
  {kind: 'patch', version: '1.0.0'},
  {kind: 'auto', version: '1.0.0'},
  {kind: 'preminor', prereleaseId: 'unknown'},
  {kind: 'unknown'},
]) {
  test(`invalid version selection is rejected: ${JSON.stringify(inputs)}`, () => {
    assert.throws(() => selectVersion('0.2.0', inputs));
  });
}

test('a prerelease channel cannot move backwards on the same base version', () => {
  assert.throws(() => selectVersion('0.3.0-rc.1', {kind: 'prerelease', prereleaseId: 'beta'}));
});

test('stale version selections fail before changing a remote PR', async () => {
  let called = false;
  const manifest = {repositoryConfig: {'.': {}}, releasedVersions: {'.': '0.3.0'},
    createPullRequests: async () => {called = true; return [];}};
  await assert.rejects(manageRelease(manifest, {
    eventName: 'workflow_dispatch', currentVersion: '0.2.0', kind: 'minor',
  }), /version on main changed/);
  assert.equal(called, false);
});

for (const [base, inputs, expected] of [
  ['0.2.0', {kind: 'major'}, '1.0.0'],
  ['0.2.0', {kind: 'preminor'}, '0.3.0-beta.1'],
  ['0.3.0-beta.1', {kind: 'prerelease'}, '0.3.0-beta.2'],
  ['0.3.0-beta.2', {kind: 'patch'}, '0.3.0'],
]) {
  test(`explicit ${expected} generates a version-only PR even without new commits`, async () => {
    const {candidate} = await releaseCandidate('', 'README.md', inputs, base);
    assert.equal(candidate.version.toString(), expected);
    const update = candidate.updates.find(update => update.path === 'rust/Cargo.toml');
    assert.equal(toml.parse(update.updater.updateContent(read('rust/Cargo.toml'))).package.version, expected);
  });
}

for (const version of ['0.3.0', '1.0.0', '0.3.0-beta.2', '1.0.0-rc.1']) {
  test(`${version} receives the correct GitHub prerelease flag`, async () => {
    let marked;
    const manifest = {repositoryConfig: {'.': {}}, createReleases: async () => {
      marked = manifest.repositoryConfig['.'].prerelease;
      return [];
    }};
    await manageRelease(manifest, {eventName: 'pull_request', currentVersion: version});
    assert.equal(marked, Boolean(semver.prerelease(version)));
  });
}

test('an explicit release updates the manifest even when there are no unreleased commits', async () => {
  const sha = 'b'.repeat(40);
  const github = {
    repository: {owner: 'example', repo: 'project', defaultBranch: 'main'},
    getFileJson: async file => JSON.parse(read(file)),
    releaseIterator: async function* () {yield {tagName: `v${current}`, sha, notes: ''};},
    mergeCommitIterator: async function* () {yield {sha, message: 'chore(main): release', files: []};},
  };
  const manifest = await Manifest.fromManifest(github, 'main');
  const selected = configureManualRelease(manifest, current, {kind: 'major'});
  const candidates = await manifest.buildPullRequests();
  assert.equal(candidates.length, 1);
  const update = candidates[0].updates.find(item => item.path === '.release-please-manifest.json');
  assert.equal(JSON.parse(update.updater.updateContent(read('.release-please-manifest.json')))['.'], selected);
});

for (const [kind, isPreview] of [['minor', false], ['preminor', true]]) {
  test(`the real Release Please publisher flags ${kind} correctly`, async () => {
    const {candidate} = await releaseCandidate('', 'README.md', {kind});
    const merged = {title: candidate.title.toString(), body: candidate.body.toString(),
      headBranchName: candidate.headRefName.toString(), baseBranchName: 'main',
      number: 7, labels: ['autorelease: pending'], files: [], sha: 'c'.repeat(40)};
    let actual;
    const github = {
      repository: {owner: 'example', repo: 'project', defaultBranch: 'main'},
      getFileJson: async file => JSON.parse(read(file)),
      pullRequestIterator: async function* () {yield merged;},
      createRelease: async (release, options) => {
        actual = options.prerelease;
        return {tagName: release.tag.toString(), url: 'https://example.invalid/release'};
      },
      commentOnIssue: async () => {}, removeIssueLabels: async () => {}, addIssueLabels: async () => {},
    };
    const manifest = await Manifest.fromManifest(github, 'main');
    const {releases} = await manageRelease(manifest, {
      eventName: 'pull_request', currentVersion: candidate.version.toString(),
    });
    assert.equal(releases.length, 1);
    assert.equal(actual, isPreview);
  });
}

test('a rebased version PR publishes the new main commit instead of the original head', async () => {
  const originalHead = 'a'.repeat(40);
  const rebasedSha = 'd'.repeat(40);
  const {candidate} = await releaseCandidate('', 'README.md', {kind: 'minor'});
  const client = await GitHub.create({owner: 'example', repo: 'project', defaultBranch: 'main',
    octokitAPIs: {octokit: {paginate: {iterator: async function* (route, options) {
      assert.equal(route, 'GET /repos/{owner}/{repo}/pulls');
      assert.equal(options.state, 'closed');
      assert.equal(options.base, 'main');
      yield {data: [{number: 7, title: candidate.title.toString(), body: candidate.body.toString(),
        base: {ref: 'main'}, head: {ref: candidate.headRefName.toString(), sha: originalHead},
        merged_at: '2026-11-03T00:00:00Z', merge_commit_sha: rebasedSha,
        labels: [{name: 'autorelease: pending'}],
      }]};
    }}},
    request: async () => {throw Error('Unexpected request');},
    graphql: async () => {throw Error('Unexpected GraphQL request');}}});
  let taggedSha;
  const github = {repository: client.repository,
    getFileJson: async file => JSON.parse(read(file)),
    pullRequestIterator: client.pullRequestIterator.bind(client),
    createRelease: async release => {
      taggedSha = release.sha;
      return {tagName: release.tag.toString(), url: 'https://example.invalid/release'};
    },
    commentOnIssue: async () => {}, removeIssueLabels: async () => {}, addIssueLabels: async () => {},
  };
  const manifest = await Manifest.fromManifest(github, 'main');
  const {releases} = await manageRelease(manifest, {
    eventName: 'pull_request', currentVersion: candidate.version.toString(),
  });
  assert.equal(releases.length, 1);
  assert.equal(taggedSha, rebasedSha);
  assert.notEqual(taggedSha, originalHead);
});

for (const tag of ['v0.3.0', 'v0.3.0-beta.1']) {
  test(`tag-only publishing marks ${tag} correctly`, () => {
    const fixture = fs.mkdtempSync(path.join(os.tmpdir(), 'jb-release-publish-'));
    try {
      fs.mkdirSync(path.join(fixture, 'bin'));
      fs.mkdirSync(path.join(fixture, 'dist'));
      const log = path.join(fixture, 'arguments');
      fs.writeFileSync(path.join(fixture, 'bin/gh'),
        '#!/bin/sh\n[ "$2" != view ] || exit 1\nprintf \'%s\\n\' "$@" > "$TEST_GH_ARGS"\n', {mode: 0o755});
      for (const file of ['jb.tar.gz', 'workflow.alfredworkflow', 'SHA256SUMS']) {
        fs.writeFileSync(path.join(fixture, 'dist', file), 'fixture');
      }
      const publishing = yaml.parse(read('.github/workflows/release.yml'));
      const script = publishing.jobs.publish.steps.find(step => step.name === 'Publish installers and binaries').run;
      execFileSync('/bin/bash', ['-e', '-o', 'pipefail', '-c', script], {cwd: fixture,
        env: {PATH: path.join(fixture, 'bin') + path.delimiter + process.env.PATH,
          HOME: fixture, RELEASE_TAG: tag, TEST_GH_ARGS: log}});
      const args = fs.readFileSync(log, 'utf8').trim().split('\n');
      assert.deepEqual(args.slice(0, 3), ['release', 'create', tag]);
      assert.equal(args.includes('--prerelease'), tag.includes('-'));
      assert.equal(args.includes('--latest=false'), tag.includes('-'));
    } finally {
      fs.rmSync(fixture, {recursive: true, force: true});
    }
  });
}
