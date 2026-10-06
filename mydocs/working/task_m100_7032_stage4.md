# #7032 Stage 4 — 최종 보고 및 PR 제출 준비

- Issue: #7032
- 승인: “다음 절차를 진행합니다.” — 최종 보고·로컬 PR 준비. 원격 쓰기는 별도 승인.
- 확인 시각: 2026-09-11 23:41 KST
- 결과: 최종 보고서와 본문 초안 작성, 최신 base 병합 위험 점검 완료. push·PR 생성 전.
- [최종 보고서](../report/task_m100_7032_report.md)

## 1. 라우팅 및 현재 상태

기본 경로는 `collaborator_self_merge`, 보조 문서는 `intake_and_review`, `local_validation`,
`visual_fixture_evidence`, `rework_and_exceptions`다. 최신 base 전진에 대해서는 원격 현행
`review_only_fast_pass.md` A.0을 확인했다. PR 번호는 아직 없으므로 번호를 예측한 review 문서와
오늘할일을 만들지 않았다. 이슈는 OPEN, 이 작업 branch의 열린 PR은 0건이다.

- source branch: `task_m100_7032`, 점검 HEAD `df32b9dd5bb474142eeb36cc184aa62e32d14163`
- 최종 전체 nextest 검증: `4b74cf0329c6025017d2ef437d56e10a18f54b75`
- lint/Skia/WASM 기준: `63fe746c7`; 이후 `src`, `tests/cases`, Cargo.toml/lock 차이 없음 재확인.
- 변경은 11파일 +1,164/-6(이번 보고서 추가 전). 새 integration source는 `tests/cases/`만 포함.
- generated suite·manifest·Cargo target 변경 없음. root tracked tree는 시작 시 clean.
- `git diff --check upstream/devel...HEAD` PASS. 전체 테스트 최종 로그와 manifest PASS 재확인.

## 2. 최신 devel 통합 위험

`git fetch upstream devel` 결과 `d408532ce9` → `b5549292d8f6854fbc86ce825580dbe9496a567a`.
`git rev-list --left-right --count upstream/devel...HEAD`는 base-only 10 / task-only 20이다.
`git merge-tree --write-tree HEAD upstream/devel` exit 0,
tree `d439c590c5113a1e9365e2d678690e088504b26b`를 얻었다. branch나 worktree를 전환하지 않았다.

- #7038: 캡션 소유자·구역 주소. 겹치는 파일은 table_layout/table_partial이며
  기존 캡션 호출에 `CaptionOwner` 인자를 전달한다. 이번 빈 문단 소유권 수정과 텍스트 충돌 없음.
- #7039: CI post-merge 재사용·fork duration 신뢰 계약 변경. 이번 PR은 workflow를 수정하지 않는다.
- #7041: 관련 배포 검증 문서.

시뮬레이션 tree의 Cargo·Studio 실행은 하지 않았다. 기존 branch의 검증을 이 tree의 성공으로
기재하지 않는다. 최신 현행 절차는 base 전진만을 이유로 반복 merge/rebase하지 않도록 한다.
따라서 현재 검증한 source branch를 유지하고, PR의 최신 CI·충돌·필수 최신화 조건을 확인한다.
통합 변경이 실제 필요해지면 #7032·#5551 focused, Rust lint 및 영향 범위 검증을 적용한다.

## 3. 제출 자료

로컬 본문 초안: `output/7032/stage4/pr-body.md`.
제목: `fix(layout): preserve empty paragraph flow in table cells (#7032)`.
최신 source/fixture 기준 전체 회귀, 신규 IR baseline의 의미, clipping 미검증, WASM 시각 판정 경계를
본문에 구분했다. PR 번호를 받은 뒤 대표 시각 증적을 영구 경로로 옮기고 self-review에 연결한다.

승인 후 실행할 명령이며 아직 실행하지 않았다:

```bash
git push upstream HEAD:task_m100_7032
gh pr create --repo edwardkim/rhwp --base devel --head task_m100_7032 \
  --title 'fix(layout): preserve empty paragraph flow in table cells (#7032)' \
  --body-file output/7032/stage4/pr-body.md
```

PR 생성 후 실제 번호로 triage·self-review 문서·필요 증적을 준비한다. self PR의 reviewer를
자신에게 지정하지 않으며, 로컬 성공을 GitHub CI 성공으로 간주하지 않는다. 병합은 별도 승인 후다.
