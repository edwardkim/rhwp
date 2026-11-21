'use strict';

const { MARKER, selectRuns, summary } = require('./pr-ci-summary.cjs');
const { WORKFLOW_PATHS } = require('./ci-impact-policy.cjs');

async function publish({github, context}) {
  const trigger = context.payload.workflow_run;
  if (trigger.event !== 'pull_request' || WORKFLOW_PATHS[trigger.name] !== trigger.path) return;
  const {owner, repo} = context.repo;
  const api = github.rest;
  const params = {owner, repo};
  const matches = p => p.state === 'open' && p.base.ref === 'devel' &&
    p.base.repo.full_name === owner + '/' + repo && p.head.repo?.full_name === trigger.head_repository?.full_name &&
    p.head.ref === trigger.head_branch;
  const linked = trigger.pull_requests || [];
  let candidates;
  if (linked.length) {
    if (linked.length !== 1) return;
    candidates = [(await api.pulls.get({...params,pull_number:linked[0].number})).data];
  } else {
    if (!trigger.head_repository?.owner?.login) return;
    candidates = await github.paginate(api.pulls.list, {...params, state:'open', base:'devel',
      head:trigger.head_repository.owner.login + ':' + trigger.head_branch, per_page:100});
  }
  const pulls = candidates.filter(matches);
  if (pulls.length !== 1) return;
  const pull = pulls[0];
  const issue = {...params,issue_number:pull.number};
  async function currentRuns() {
    const runs = await github.paginate(api.actions.listWorkflowRunsForRepo,
      {...params, head_sha:pull.head.sha, event:'pull_request', per_page:100});
    return selectRuns(pull, runs);
  }
  const reports = await currentRuns();
  const signature = items => items.map(({run}) =>
    [run?.id, run?.run_attempt, run?.status, run?.conclusion].join(':')).join('|');
  // A superseded event can be the last queued wake-up. Always recompute the live head.
  if (!reports.some(({run}) => run)) return;
  for (const item of reports) {
    if (!item.run) continue;
    item.jobs = await github.paginate(api.actions.listJobsForWorkflowRunAttempt, {...params,
      run_id:item.run.id, attempt_number:item.run.run_attempt, per_page:100});
  }
  const body = summary(pull, reports);
  const comments = await github.paginate(api.issues.listComments, {...issue,per_page:100});
  const own = comments.find(c => c.user?.login === 'github-actions[bot]' && c.body?.startsWith(MARKER+'\n'));
  if (own?.body === body) return;
  if (signature(reports) !== signature(await currentRuns())) return;
  const latest = (await api.pulls.get({...params,pull_number:pull.number})).data;
  if (!matches(latest) || latest.head.sha !== pull.head.sha) return;
  if (own) await api.issues.updateComment({...params,comment_id:own.id,body});
  else await api.issues.createComment({...issue,body});
}

module.exports = publish;
