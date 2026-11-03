// Merge only the generated version PR at the exact commit that passed CI.
const fs = require('node:fs');
const {execFileSync} = require('node:child_process');

const SHA = /^[a-f0-9]{40}$/i;
const VERSION_FILES = new Set([
  'rust/Cargo.toml', 'rust/Cargo.lock', 'CHANGELOG.md',
  'version.txt', '.release-please-manifest.json',
]);

function githubRequest(method, endpoint, fields = {}) {
  const args = ['api', '--method', method, '-H', 'Cache-Control: no-cache', endpoint];
  for (const [key, value] of Object.entries(fields)) args.push('-f', `${key}=${value}`);
  return JSON.parse(execFileSync('gh', args, {encoding: 'utf8'}));
}

function mergeReleasePR({repository, number, sha}, request = githubRequest) {
  if (!/^[\w.-]+\/[\w.-]+$/.test(repository) || !/^[1-9][0-9]*$/.test(String(number)) || !SHA.test(sha)) {
    throw Error('Invalid repository, release PR number, or tested commit');
  }
  const endpoint = `repos/${repository}/pulls/${number}`;
  const pr = request('GET', endpoint);
  if (pr.base?.ref !== 'main' || pr.head?.repo?.full_name !== repository ||
      pr.head?.ref !== 'release-please--branches--main' || pr.head?.sha !== sha ||
      !pr.title?.startsWith('chore(main): release ') || pr.draft ||
      !pr.labels?.some(label => label.name === 'autorelease: pending')) {
    throw Error('The release PR no longer matches the tested version commit');
  }
  const files = request('GET', `${endpoint}/files?per_page=100`);
  if (!Array.isArray(files) || files.length === 0 || files.length > VERSION_FILES.size ||
      files.some(file => !VERSION_FILES.has(file.filename) ||
        (file.previous_filename && !VERSION_FILES.has(file.previous_filename)))) {
    throw Error('Automatic merging is limited to version and changelog files');
  }
  if (pr.merged === true && SHA.test(pr.merge_commit_sha)) {
    return {number: String(number), sha: pr.merge_commit_sha};
  }
  if (pr.state !== 'open') throw Error('The release PR is closed without being merged');
  const result = request('PUT', `${endpoint}/merge`, {merge_method: 'rebase', sha});
  if (result.merged !== true || !SHA.test(result.sha)) {
    throw Error('The release PR was not merged; stop before tagging or publishing');
  }
  return {number: String(number), sha: result.sha};
}

module.exports = {mergeReleasePR};
if (require.main === module) {
  try {
    const result = mergeReleasePR({repository: process.env.GITHUB_REPOSITORY,
      number: process.env.RELEASE_PR_NUMBER, sha: process.env.RELEASE_PR_SHA});
    for (const [key, value] of Object.entries(result)) {
      fs.appendFileSync(process.env.GITHUB_OUTPUT, `${key}=${value}\n`);
    }
    console.log(`Merged release PR #${result.number} using rebase.`);
  } catch (error) {
    console.error(`Release merge failed: ${error.message}`);
    process.exitCode = 1;
  }
}
