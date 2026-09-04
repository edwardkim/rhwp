# #3309 단계 1 완료 — green head 재사용 fast-pass 구현·정적 검증

- 이슈: [#3309](https://github.com/edwardkim/rhwp/issues/3309)
- 브랜치: `task_m100_3309`
- 기준: `upstream/devel` `f68aa8be5`

## 완료 내용

CI, CodeQL, Render Diff preflight가 trailing review-only commit을 최신순으로 후보화하고, 현재 base를
포함하는 가장 최근 green candidate를 재사용하도록 보정했다. 최신 후보의 check/workflow가 아직 없거나
진행 중이면 더 이전 후보를 확인하고, 가장 최근 완료 후보의 실패·base 불일치·비허용 merge·비허용 경로는
기존처럼 full CI fallback으로 처리한다.

`#3304`의 `bcff621` 뒤 `2042ee0` 사례를 기준으로, 이전에는 비문서 commit만 조회해 full CI가 재실행됐던
경로가 이제 직전 green PR head를 선택한다. Update branch 뒤 이전 SHA의 stale run은 일반 cancel을 먼저
시도하지 않고 force-cancel API를 즉시 사용하도록 절차도 맞췄다.

## mydocs 증적 허용 확인

세 workflow의 `isAllowedReviewPath`는 `mydocs/` 경로를 파일 상태·확장자 검사보다 먼저 허용한다. 따라서
`mydocs/pr/assets` 등에 추가·수정·이동하는 PDF, HWP/HWPX, PNG를 포함한 증적 자료는 문서-only PR의
fast-pass 대상이다. 이 보장을 `review_only_fast_pass.md` 허용 범위에도 명시했다.

## 정적 검증

- `git diff --check` 성공
- `actionlint -ignore 'SC2086' .github/workflows/ci.yml .github/workflows/codeql.yml .github/workflows/render-diff.yml`
  성공. Render Diff의 기존 SC2086 경고는 변경 범위 밖이다.
- Ruby YAML parse 성공: CI, CodeQL, Render Diff
- 각 inline GitHub Script `node --check` 성공
- #3304의 `bcff621 → 2042ee0` current-base 관계와 기존 Build & Test·CodeQL·Render Diff success를 재확인
- mock으로 최신 문서 후보가 미완료일 때 이전 green 후보를 선택하는 경우, 최신 완료 후보 실패와 current-base
  불일치에서는 full CI fallback하는 경우를 세 workflow에서 확인
- `mydocs/` 허용 조건이 `file.status !== 'added'` 검사보다 앞서는지 세 workflow에서 확인

Rust 소스·테스트·fixture는 변경하지 않았으므로 Cargo 테스트는 이 단계의 변경 범위에 해당하지 않는다. workflow
변경 PR의 GitHub full CI와 그 뒤의 review-only 증적 commit fast-pass 실증은 PR 단계에서 수행한다.
