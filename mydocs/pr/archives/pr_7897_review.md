---
kind: report
status: archived
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-14
---

# PR #7897 self-review — 대형 표 문서의 초기 조판·스크롤·셀 조회 성능 개선

- PR: https://github.com/edwardkim/rhwp/pull/7897 · 작성자 edwardkim · base devel.
- base route: `collaborator_self_merge.md`.
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`, `review_only_fast_pass.md`, `post_merge.md`.
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`와 위 기본·보조 문서.
- 검증한 code head: `b076788bf345927ee236e00c8f3071ea391b40a9`.
- 최신 base: `fea344f1fbed64737862e7f9f2c56287911d1256`. code head의 merge-tree는 충돌 없이 종료했고 최신 base의 주석·문서 변경을 보존한다.
- 상세 구현·명령·시각 자료: [기존 계획 및 결과](../../plans/issue_7639_remaining_performance.md).

## 구현 주장과 호출 경로 검토

| 변경 | 실제 소비 경로와 검토 결과 | 검증 및 한계 |
| --- | --- | --- |
| rowspan·선언 너비 탐색 | 원본 셀을 행별로 모아 기존 source 순서·안정적 열 정렬·반열림 범위·허용 오차를 유지한다. `compute_column_widths`와 layout 너비 소비 조건을 바꾸지 않는다. | 순수 source 질의 계약 및 기존 분할 집중 검사 12건 통과. 새 검사는 조판 위치·쪽수 기준값을 추가하지 않는다. |
| 불변 Table 행 색인 | block prepare, continuation drain/step, whole/partial paint가 같은 source 색인을 소비한다. 약한 참조 cache와 immutable borrow를 가진 guard의 명시적 Drop으로 살아 있는 색인의 수명을 제한한다. | guard 생존 중 mutation 거절 compile probe, 실제 행 삽입→Undo/Redo→문서 교체 확인. 원격 저장 프레임 높이 `remember_stored_frame_rows`를 그대로 보존한다. |
| 현재 쪽 셀 경계 API | 기존 collector의 셀 소속·좌표·순서·rounding을 현재 page tree에서만 조회한다. hover/resize는 page API, 전체 선택·F5/Tab과 nested ByPath는 기존 경로다. | 0/80/130/마지막 쪽을 기존 전체 결과의 page filter와 비교해 동일함을 확인했다. 새 API의 empty/error를 전체 조회로 확대하지 않는다. 구 WASM의 API 미제공 fallback만 유지한다. nested 경로 성능 개선은 주장하지 않는다. |
| 동기 Canvas font 해석 | 동기 paint callback 안에서만 정확한 CSS font 문자열의 해석을 재사용하고 finally에서 scope를 복원한다. 실제 font setter/메트릭 준비와 await 이후 상태를 생략하지 않는다. | Studio 동작 검사 및 전쪽 Canvas raw RGBA 동일성 확인. font 공급·한컴 조판 정책은 변경하지 않는다. |

문서 ID·쪽수·우연한 속성에 따른 배치 예외, 좌표 clamp, 기존 기준값 완화는 추가하지 않았다. 새 bbox 경로의 수정 전 동작 실패/수정 후 통과와 guard 수명 반례는 기존 계획의 실행 증거로 연결했다. source 질의 검사는 기존 의미의 보존 검사이며 과거 조판 결함 검출로 승격하지 않는다.

## 완료한 검증

- 공용 `target/pr-review`에서 fmt·Native/WASM/workspace all-target Clippy·workspace build·고정 base manifest/unit-tier 검사를 통과했다. 최신 base `fea344f1…` 비교도 통과했다.
- 전체 Rust nextest **10,285 통과·실패 0·skip 45**, Native Skia lib **4,041 통과·ignored 12**, placeholder 2건·direct PDF 4건을 통과했다.
- TypeScript 및 Studio **1,840 통과·실패 0·skip 2**를 통과했다.
- Native와 **Docker locked wrapper WASM**을 재빌드했다. 실제 Studio 응답과 root pkg/public WASM SHA-256은 `2615f003fe0c05bcfac7642dc9c8e19a4981a22eee34d0315649b24a0f7bd90a`로 같다.
- 전체 161쪽 Native SVG 바이트와 실제 WASM Canvas raw RGBA는 각 변경 전 대조군과 동일했다. 통합 source/test 해시 변경은 없었다.
- 실제 Studio 첫 표 클릭 **71ms**, page bbox **0.6ms**, 실제 마우스 휠 최대 RAF 간격 **93.6ms**를 관측했다. 메인터너는 “지연이 개선되었습니다”라고 판정했다. 호스트 부하와 단계가 다른 측정값을 모든 환경의 상한으로 주장하지 않는다.
- ignored `output/pr-review/issue7639-perf-20261013/devel-merge-20261014/`의 `validation.json`, `base-advance.json`, `final-source-drift.json`, `browser/`, `svg-parity.json`, `canvas-parity.json`에 명령·원문·전후 대조를 보존했다.

## 독립 시각 근거와 승인된 예외

원본 [issue2063 HWP](../../../samples/issue2063_huge_cellbreak_table.hwp)의 SHA-256은 `47de1ce82472050a4c5f33ccab2a3eac2c7e55d0e2a688febd085f0b4c8320ca`다. 같은 원본의 [한컴 2020 Print PDF](../../working/assets/issue7639/hancom-2020-print.pdf)는 162쪽이며 SHA-256 `32ecd01c0e82abb5d89b42edf29465f252d922ef78f98cd23a13edfd0b524ddf`다.

Native/fresh WASM은 모두 161쪽으로 전쪽 Sweep이 중단되어 전체 TSV는 완성되지 않았다. 자동 gate는 `re_review_required`다. 대표 1쪽은 두 경로 모두 2px 실루엣 **99.74962%**, strict pixel **94.01578%**였고 review·standalone overlay를 직접 열어 표 격자·본문 소속과 얇은 획의 raster 차이를 확인했다. 대표 결과를 전쪽 통과로 확대하지 않는다.

메인터너는 2026-10-14에 **“성능 보정 범위의 시각 게이트 예외 수용”**을 명시했다. 이 PR은 성능 보정·변경 전후 출력 보존 범위로 수용하며, 기존 161/162쪽 조판 차이와 전쪽 독립 Print 일치는 미해결/미검증으로 남긴다. 기존 검사·golden·쪽수 기대값은 유지한다. #7639는 `Refs`로 연결하고 이 review에서 이슈 종료를 선언하지 않는다.

## 정확한 code head GitHub CI

`b076788bf345927ee236e00c8f3071ea391b40a9`의 check 35개는 모두 completed이고 success 또는 정당한 skipped였다. 실패·대기 check는 없었다.

- [CI / Build & Test](https://github.com/edwardkim/rhwp/actions/runs/38490326034): 전체 archive workers, Native Skia, lint, frontend package gates 성공.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/38490326019): Rust·JavaScript/TypeScript·Actions·Python Analyze 및 GHAS CodeQL 성공.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/38490326022), [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/38490326035), [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/38490326141): 성공.

## 판정과 남은 병합 조건

**승인 — 사용자 지정 성능 범위의 시각 게이트 예외를 적용한 self-review.** 실행으로 확인한 회귀나 코드 검토상 병합을 막는 새 결함은 확인하지 못했다. 독립 전쪽 조판 일치의 부족은 위 예외 범위로 명시하며 CI 통과로 해결됐다고 하지 않는다.

사용자는 “CI가 완료되었습니다. self-review 후 머지처리합니다”라고 승인했다. 이 기록과 오늘할일만 single-parent trailing commit으로 반영한다. push 전 현재 base/head merge-tree, 공백·실제 merge tree의 링크·기존 오늘 기록 보존을 검증하고, 원격 SHA 재조회 후 push한다. **trailing head의 실제 fast-pass 및 required checks 성공을 확인한 뒤** 정확한 SHA로 정상 squash merge한다. 이 문서 작성 시 trailing 검사·merge는 아직 수행 전이며, 완료값은 실행 증적과 GitHub 병합 기록으로 확정한다.
