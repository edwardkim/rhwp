# PR #8016 리뷰 — 공통 검증 지침 분리

## 최종 판정

**승인**. 문서 head `d3b6f0cee9341a5398711f8f8c8d908ac8467c3f`의 로컬 검증과 GitHub Actions가 완료·성공했고 정상 squash merge를 확인했다.
사용자는 지침 분리 PR 생성, CI 모니터링·정상 병합·후속 처리를 승인했다. 기록은 승인된 옵션 2의 별도 후속 PR로 반영한다.

## 접수와 범위

- [PR #8016](https://github.com/edwardkim/rhwp/pull/8016), 작성자 `jangster77`, collaborator self-review. reviewer를 요청하지 않았다.
- base `6ed1682c3267ea786734eb2e75252916a861d1fd`, head `d3b6f0cee9341a5398711f8f8c8d908ac8467c3f`.
- 지침 Markdown13개,88추가/19삭제. 리베이스 반영 PR의 기능 보존 검증과 devel 대비 실루엣 하락 **0.1%p 이상** 재검토를 명확히 했다.
- 90% 최소 gate, 쪽수·내용 보존·직접 판독 및 완료 페이지98% px 회귀 조건을 함께 적용한다.
- 원 작업 브랜치의78bfe501327b9f113e65e191acfadca6db9bae6a에서13개 지침을 원base로 복원했다. 코드·사용자 초안 등207개 미커밋 파일의 내용을 보존했다.
- [#7445](https://github.com/edwardkim/rhwp/issues/7445)의 렌더링 보정·검증 완료를 주장하지 않으며 해당 이슈를 닫지 않는다.

## 검증

- 변경13문서의 `git diff --check` 및 내부 Markdown 링크 검사 통과. 변경 manual10개 메타데이터 직접 검사 통과.
- 고정 base/head의 `git merge-tree --write-tree` 종료 코드0. source/test/fixture 변경이 없어 Cargo·Visual Sweep·px 회귀 추가는 비해당이다.
- 조판 원칙 준수 검토: 검증 지침만 바꾸며 조판 생산·측정·paint 코드는 포함하지 않는다.
- 리베이스 반영 기능 보존: 최신devel에서 만든 문서 전용 브랜치이며 제품 기능 변경은 없다.
- 정확한 head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/39110180107), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39110180116),
  [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39110180192), [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39110180226),
  [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/39110180171)가 모두 완료·성공했다. 미완료·실패 check0건을 확인하고 병합했다.
- 메인CI는 문서전용으로 heavy Rust/frontend lane을 생략했다. Adapter·Proptest는 개별 allowlist에 루트 `AGENTS.md`·`CLAUDE.md`·`CONTRIBUTING.md`가 빠져 실제 실행됐다.
  각각 `full-code-or-non-mydocs-change`와 `code-or-non-mydocs-change`를 확인했으며 두 실행 모두 성공했다. 이를 두 workflow의 fast-pass 성공으로 표시하지 않는다.

- [병합 후 duration 갱신](https://github.com/edwardkim/rhwp/actions/runs/39110633483)은 성공했으나 `ready=false`, `no-verified-pr-duration-measurements`다. 문서전용으로 Rust worker가 생략되어 검증된 측정 자료가 없으므로 갱신 보류로 기록하며 검증CI를 재실행하지 않았다.

## 병합과 후속 처리

- 실제 merge SHA: [`a93b68586dc368e311a5220c52d016b1ad754995`](https://github.com/edwardkim/rhwp/commit/a93b68586dc368e311a5220c52d016b1ad754995), 병합시각 `2026-10-18T08:13:39Z`.
- 원 지침13개가 merge tree에 포함됨을 확인했다. 이 기록 PR은 review와 오늘할일만 추가한다.
- 후속 기록 PR의 정확한 head CI·mergeability 확인과 정상 merge 뒤 local devel fast-forward, 소유 branch/worktree 정리를 실행한다.
- 공유 `target/pr-review`, 원 렌더링 작업장과 사용자 초안은 보존한다. `/Users/tsjang/rhwp/output`에는 진행 중인 #7445의 원본·글꼴·동결 producer·TSV·PNG가 있어 전체 삭제를 수행할 수 없다. output 정리는 미완료이며 활성 자료를 유지한다.
- 이번 PR은 source contributor PR 또는 닫을 issue가 없다. 후속 기록 PR 자체에는 중복 오늘할일·comment·재귀 후속 PR을 만들지 않는다.

## Merge 후 contributor PR comment 계획

이 기록이 devel에 반영된 뒤 원 PR에 한국어 존댓말로 실제 merge SHA, 정확한 head의 CI URL·결과,
문서13개 범위와 링크·metadata 검사, source/visual 비해당, devel 동기화·소유 정리 결과를 게시한다.
CodeQL·Adapter·Proptest의 실제 실행 결과와 #7445 미해결 범위를 구분하고 이 review를 merge SHA 고정 링크로 연결한다.
