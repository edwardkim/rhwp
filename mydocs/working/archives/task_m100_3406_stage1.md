# #3406 단계 1 완료 — devel 동일 저장소 PR stale run reaper 구현

- 이슈: [#3406](https://github.com/edwardkim/rhwp/issues/3406)
- 브랜치: `task/3406-ci-force-cancel`
- 기준: `upstream/devel` `91bd61758`

## 완료 내용

`devel` 대상의 동일 저장소 PR에서 Update branch가 `synchronize` event를 만들면, 전용 reaper가 같은
PR 번호의 active `pull_request` run 중 live head SHA와 다른 run을 force-cancel하도록 구현했다. workflow는
PR source를 checkout하거나 shell·PR 제공 script를 실행하지 않으며 GitHub API만 사용한다.

취소 직전 live PR head를 다시 읽으므로, 연속 Update branch에서 먼저 시작된 reaper가 이후 최신 SHA run을
취소하지 않는다. 기존 CI·CodeQL·Render Diff의 concurrency 설정과 job 구성은 변경하지 않았다.

`main`은 메인터너 전용 릴리즈 브랜치이므로 변경하지 않는다. GitHub가 external fork의 `pull_request` token
write 권한을 읽기 전용으로 낮추는 경우에는 job을 의도적으로 skip하며, 이 경우 review 절차의 수동
force-cancel API를 사용한다.

## 정적 검증

- `actionlint .github/workflows/cancel-stale-pr-runs.yml` 성공
- Ruby YAML parse 성공
- embedded GitHub Script `node --check` 성공
- `git diff --check` 성공

Rust·WASM·frontend·fixture 변경은 없으므로 Cargo·WASM·시각 검증은 이 단계 범위에 해당하지 않는다. 다음
단계에서 `jangster77/rhwp`의 동일 저장소 head 격리 PR로 실제 `pull_request` write-token reaper를
Update branch와 함께 재검증한다.
