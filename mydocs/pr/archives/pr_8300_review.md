# PR #8300 리뷰 — 표 칸 안에 표 만들기

## 최종 판정

**메인터너 보정 후 수용 가능**. 원 head는 최신 devel과 자동 병합하면 컴파일되지 않지만,
원 contributor commit을 보존한 보정 code head `199950071643d6245f6ff379276f61dcc6d392bb`는 필수 로컬 검증과 GitHub Full CI를 모두 통과했다.
작업지시자는 원 PR 보정 push·새 head CI·self-review·정상 병합·후속 댓글을 승인했다. code push는 완료했으며,
이 문서와 오늘할일만 추가한 trailing head의 필수 검사·병합 상태를 확인한 뒤 승인 리뷰와 정상 병합을 진행한다.

- PR: [#8300](https://github.com/edwardkim/rhwp/pull/8300), `emptinessform`의 첫 rhwp 기여.
- 목적: [#8262](https://github.com/edwardkim/rhwp/issues/8262)의 표 칸 안 표 생성.
- 원 source head: `dc171d224bd6d1f5702554d7e171af0961dab755`.
- 검증 base: `2adaa0859f94f30f7dc3b994ee27c34a872906ba` (`upstream/devel`).
- source/base 병합 commit: `58510d1df30140f189c481742133e4d75b36f3b2`.
- 로컬 보정 commit: `199950071643d6245f6ff379276f61dcc6d392bb`.
- 로컬 가시성 브랜치: `review/pr-8300-table-in-cell`.
- 원격 처리 전에는 source head·fork ref·devel을 다시 대조한다.

## 검토 경로

- 기본 경로: maintainer 일반. 첫 기여자 원 source 직접 보정 특칙을 적용했다.
- 보조 경로: 접수·리뷰, 로컬 검증, 첫 기여자, 시각·fixture, 최신 devel 호환 보정, review-only trailing, merge 후속 처리.
- 권위 문서: `pr_review_workflow.md`, `pr_review/README.md`, `maintainer_general.md`,
  `intake_and_review.md`, `local_validation.md`, `first_time_contributor.md`,
  `visual_fixture_evidence.md`, `collaborator_external_pr.md`, `review_only_fast_pass.md`, `post_merge.md`,
  `github_operations.md` 및 저장소 진입 권위 문서를 읽었다.
- 에이전트 self-review에서 최종 로컬 blocker는 없었다. 기여자의 원 구현과 메인터너의 추가 변경을 분리해 대조했다.

## 기존 해결 여부와 원 기여

검증 base의 Studio `table:create`는 셀 안 실행을 막으며 `createTableInCellByPath`도 없다.
따라서 이 PR의 새 기능은 최신 devel에서 아직 제공되지 않는다. 단순한 파일 열기나 렌더링 수정이
아니라 실제 편집 명령을 추가하는 기여다.

기여자는 셀 경로를 따라 깊이에 관계없이 표를 생성하는 Native/WASM API, Studio 명령 배선,
셀 안 대화창의 ‘글자처럼 취급’ 고정, Undo/Redo 등록, Rust 7건과 Studio 5건의 검사를 추가했다.
앞 본문·표·뒤 본문으로 문단을 나누며, 반환 경로로 캐럿을 새 표의 첫 셀에 둔다.

## 최신 devel 호환 보정

Git의 자동 병합은 성공하지만 그 결과를 실제 빌드하면 공통 `build_inline_table` 안의
`section_idx`·`para_idx`·`char_offset`이 범위에 없어 실패한다. 원 PR의 과거 head CI 성공으로
현재 base의 호환성을 대신할 수 없다.

선행 변경은 `c9cbe8ad6d62e9905a239acd27669554f5d9842b`, 통합 PR #8319,
merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`다. 이 변경은 새 셀의 빈 줄 높이와 기준선을
현재 캐럿 서식의 공통 `reflow_line_segs`로 계산한다. 이 기능을 제거하지 않고 다음처럼 연결했다.

| 경로 | 생산 → 소비 |
| --- | --- |
| 본문 표 | 기존 `creation_line::for_cursor` → 공통 builder 인자 → 최소 행 높이와 새 셀 LineSeg |
| 셀 안 표 | 실제 캐럿 셀 문단·글자 모양·셀 안쪽 폭 → 같은 `for_empty`/`reflow_line_segs` → 같은 builder 소비 |
| 표를 소유하는 문단 | 원 PR의 초기화와 TAC 재조판 → 실제 셀 문단 배치. 빈 줄 LineSeg를 호스트 초기값으로 복사하지 않음 |

값의 경로는 `creation_line.rs::for_empty` → `table.rs::build_inline_table`의 최소 행 높이·새 셀 저장 줄 → `height_measurer.rs::measure_table_impl_with_storage` 및 `layout/table_layout.rs::resolve_row_heights_with_common_fit` → `layout_table`의 행 원점과 `table_cell_content.rs`의 중첩 배치다. 셀 문단 소유 정보와 실제 원점은 `reflow_cell_paragraph_by_path`·`recalculate_cell_paragraph_vpos_by_path`에서 갱신한다. 최종 배치 이후의 여백도 새 렌더 트리 관계 검사와 독립 Print 확대 판독으로 확인했다.

로직 보정은 두 Rust 파일이고, 새 API의 JS 바인딩과 선언 파일도 동일한 fresh package로 동기화했다. 빌드 wrapper·CI 정책은 바꾸지 않았다. 보정은 renderer/backend별 좌표 예외를 추가하지 않았다.

중첩 표의 여백은 기존
셀 안 여백과 원본 표 바깥 여백 계약을 따른다. pagination·rowspan·continuation 규칙은 변경하지 않는다.

초기 검토 보정에서 빈 줄의 구현 태그까지 표 호스트에 상속한 시도는 실제 생성 결과가
한컴 Print 대비 81.48%여서 폐기했다. 최종 후보는 원 PR의 호스트 초기화·재조판을 유지한다.
같은 셀 상단과 표 상단의 간격 관계는 폐기 후보에서 FAIL, 최종 후보에서 PASS다.
이 과정을 원 기여의 결함으로 분류하지 않는다.

## 실제 생성·저장 및 시각 검증

정식 증거는 `mydocs/pr/assets/pr8300-maintainer-20261101/`에 있다. `visual-summary.json`은
전쪽 Native/fresh WASM TSV 원본, 입력·PDF 해시와 정확한 측정 시각을 연결한다.
96 DPI, Print 프로필, 2022 호환 모드, 경계 보정 전 2px raw 실루엣을 사용했다.

| 대상 | 쪽수 | base Native/WASM 최저 | 후보 Native 최저 | 후보 fresh WASM 최저 |
| --- | --- | --- | --- | --- |
| 제출 HWP의 빈 셀 중첩 표 | 1 | 100.00% / 100.00% | 100.00% | 100.00% |
| 제출 HWPX의 빈 셀 중첩 표 | 1 | 100.00% / 100.00% | 100.00% | 100.00% |
| 제출 HWP의 본문 사이 삽입 | 1 | 100.00% / 100.00% | 100.00% | 100.00% |
| API로 새로 생성한 빈 셀 HWP | 1 | 100.00% / 100.00% | 100.00% | 100.00% |
| API로 새로 생성한 빈 셀 HWPX | 1 | 100.00% / 100.00% | 100.00% | 100.00% |
| API로 새로 생성한 본문 사이 HWP | 1 | 100.00% / 100.00% | 100.00% | 100.00% |
| API로 새로 생성한 본문 사이 HWPX | 1 | 100.00% / 100.00% | 100.00% | 100.00% |

새 생성 입력 4개도 같은 고정 base의 immutable Native/WASM으로 저장 재열기·출력하여 추가로 대조했다.
8개 base/head PNG 쌍이 각각 바이트까지 같았고 전쪽 raw 하락은 0%p다. base 산출 TSV 8개도
이미 커밋된 `created/{native,wasm}/*/silhouette.tsv` 각각과 바이트까지 같아 동일 raw 파일을 재사용했다.
공통 TSV SHA-256은 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`이다.
base Native 실행 파일 SHA-256은 `0edb323e56cb96717b73d51b92ceb6e06875e83f5b020f272fa1a8792880e678`,
base WASM은 `a65db888312503f7c16ed1c3dfad59d30c01972106bacfdcdae7286a82b1b28f`이다.
base 제품 내용의 재사용 근거는 [앞선 base 결합 검증](../assets/pr8296-maintainer-20261101/combined-validation.json)이다.
입력·PDF 매핑은 [Print 실행 기록](../assets/pr8300-maintainer-20261101/print-execution.json)의 동일 4개 파일이다.
명령은 기존 `scripts/visual_sweep.py --file-target <key> <input> <pdf> --rhwp-bin <immutable-base-bin>`에
`--rasterizer png --compat 2022 --dpi 96 --page-workers 2 --silhouette-only`를 사용했고, WASM에는
`--wasm-pkg <immutable-base-pkg>`를 추가했다. 이 base 측정은 기존 파일 열기 보존이며 생성 기능의 존재를 뜻하지 않는다.

- [검증 입력·측정 요약](../assets/pr8300-maintainer-20261101/README.md)
- [후보 전쪽 TSV 목록](../assets/pr8300-maintainer-20261101/visual-summary.json)
- [로컬 검사·원본 로그](../assets/pr8300-maintainer-20261101/validation.json)
- [빈 셀 표 확대](../assets/pr8300-maintainer-20261101/created-nested-empty-hwp-detail.png) /
  [본문 사이 표 확대](../assets/pr8300-maintainer-20261101/created-nested-mid-hwp-detail.png)

기존 제출 3개는 base에서도 양쪽 모두 100.00%다. 이는 기존 파일 열기 기능의 보존 근거이며,
새 표 생성 기능은 아래 실제 명령 실행과 새로 저장한 파일로 별도 검증했다.

실제 Studio의 명령 라우터와 표 선택기·상세 대화창을 사용해 빈 셀과 `가나다라` 사이에 2×2 표를
만들었다. 본문 대화창의 기존 선택권, 셀 대화창의 inline 고정, 원 표의 행·열 수 유지,
캐럿 깊이 1→2, Undo/Redo, 깊이 2→3 생성과 반복 history, 새 중첩 셀의 실제 한글 키보드 입력, 잘못된 `treatAsChar:false`의
문서 무변경 거절을 확인했다. Canvas2D 화면과 실제 HWP/HWPX 저장본을 보존했다.

Studio 저장본 4개는 Native API 생성 입력과 SHA-256이 모두 같다. 그 정확한 4개 입력을
한/글 2020 `11.0.0.9136`의 Print(인쇄 방법 0, 한 쪽에 한 쪽)로 출력했다.
`print-execution.json`에 각 job ID·입력/PDF SHA-256·인쇄 실행 정보를 기록했고,
Native와 fresh WASM에서 저장 후 재열기·출력한 결과를 해당 PDF와 비교했다.

표 외곽·내부 선과 부모 셀 안쪽 여백, ‘가나–표–다라’의 순서와 간격을 동일 배율로 확대 판독했다.
한컴 Print의 내장 글꼴은 HCRBatang이고 Canvas2D는 Noto Serif KR 대체 글꼴을 실제 사용했다. `font-receipt.json`과 쪽별 WASM 완료 manifest에 실제 family·폰트 해시·Chrome 버전을 기록했다. 글꼴 및 선의 안티앨리어싱 차이는 있으나 여백 침범·글자 누락·표와 뒤 문단 겹침은 없다. 수식은 대상에 없다.

fresh 최적화 WASM은 `8545edf30ed554a6bd0a964ac4f55774ae2fcdd548654ffb1bf2b3503fa5782b`이고
실제 browser가 소비한 응답 SHA와 일치한다. 빌드는 로컬 `24484260bb`와 명시한 두 source 변경으로
수행했으며 `wasm-build-manifest.json`에 source 파일 해시를 기록했다. 이후 test·증거 문서 변경을
제품 바이너리의 source 변경으로 표현하지 않는다.

## 회귀·lint와 검사 보강

최소 글꼴 격리 환경의 전체 회귀 12,261 PASS / 0 FAIL / 45 skip, Native 라이브러리 4,040 PASS / 0 FAIL / 12 ignored, placeholder 2건과 직접 PDF 22건을 통과했다. 보강한 #8262 최종 Native 7건도 다시 통과했다. Studio는 생성 바인딩 동기화 후 2,710 PASS / 0 FAIL / 2 skip이며 TypeScript 검사도 통과했다. fmt·Native/WASM Clippy·workspace build/all-target Clippy·suite manifest·diff 검사를 모두 통과했다. CI에서 제외하는 `llm-verifier-verdict-protocol`도 로컬에서 별도 build/all-target Clippy로 확인해 workspace 전체를 포함했다. 초기 suite drift는 테스트 파일 보강 후 파생 harness를 재생성하여 해결했으며 정책은 바꾸지 않았다. `validation.json`과 정식 로그에 모든 실행 결과를 기록했다.

기존 검사는 이미 구조·소유·문단 순서·저장 보존을 검사하며, 폭과 속성의 고정값은 한컴 원본의
HWPUNIT 계약이므로 유지했다. 절대 px 핀은 없다. 실제 렌더 트리에서 부모 셀의 padding과
중첩 표 margin을 보존하는 포함 관계, 표 뒤 문단이 여백 뒤에 놓이는 관계를 같은 검사에 보완했다.
OS에 상용 글꼴이 설치되어 있다는 가정이나 조용한 skip을 추가하지 않았다.

## GitHub Full CI와 최종 self-review

정확한 code head `199950071643d6245f6ff379276f61dcc6d392bb`에서 아래 최신 실행이 모두 성공했다. 각 제품 workflow의 최신 성공 attempt는 2다.

| 검사 | 최신 성공 실행 |
| --- | --- |
| CI | [run 41292524971, attempt 2](https://github.com/edwardkim/rhwp/actions/runs/41292524971/attempts/2) |
| CodeQL | [run 41292525039, attempt 2](https://github.com/edwardkim/rhwp/actions/runs/41292525039/attempts/2) |
| Render Diff | [run 41292524990, attempt 2](https://github.com/edwardkim/rhwp/actions/runs/41292524990/attempts/2) |
| Adapter inter-diff | [run 41292524970, attempt 2](https://github.com/edwardkim/rhwp/actions/runs/41292524970/attempts/2) |
| Proptest roundtrip | [run 41292524968, attempt 2](https://github.com/edwardkim/rhwp/actions/runs/41292524968/attempts/2) |

[최종 CI Impact Policy 감사](https://github.com/edwardkim/rhwp/actions/runs/41294478173)도 성공했다. `mode=full;rfp=0`이며 base는 `2adaa0859f94f30f7dc3b994ee27c34a872906ba`다.
check 40건은 성공 34·조건부 skip 6으로 모두 완료됐고 실패는 없었다. A–E archive 빌드·회귀, Native Skia, frontend package, lint가 완료됐다.

self-review에서는 실제 캐럿 서식의 공통 생산·소비, 호스트 TAC 재조판 보존, 셀 경로·문단 나눔·dirty/cache/event,
Studio snapshot·Undo/Redo·새 캐럿과 생성 바인딩을 다시 대조했다. 원 기여 commit을 수정하지 않았고,
문서 ID나 관측 좌표에 따른 예외·렌더러별 분기·절대 px 핀을 추가하지 않았다.

## 문서 후행과 병합 조건

이 trailing commit은 이 검토 기록과 [오늘할일](../../orders/20261101.md)만 추가한다.
코드 head의 완료된 CI를 근거로 review-only 재사용 조건을 확인하며, 최신 trailing head의 preflight·required aggregate·정책·mergeability 성공 뒤 승인 리뷰와 정상 병합을 수행한다.
병합 후 실제 merge SHA·devel 동기화·연결 이슈 상태·후속 댓글과 정리 결과를 다시 확인한다.
최신 head 결과는 원 PR의 승인 리뷰와 메인터너 후속 댓글에 확정하고 contributor fork branch는 보존한다.
