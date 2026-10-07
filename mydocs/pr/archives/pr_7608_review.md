# PR #7608 검토 — 표 속성 편집 시 HWP5 공통 속성 보존

## 최종 판정

**승인 — 작업지시자의 명시적 병합 지시에 따라 현재 제출된 FLAGS 비트 보존 범위만 수용한다.** 검토 head는 `1452bf8845a299873a6b30fff7fe6f37003b2741`이다. 추가 저장 줄 정보 보정과 #7620·#7621 renderer 결함은 후속 작업으로 남긴다. 이는 시각 gate 통과 판정이 아니며 아래 미검증 결과와 기여자의 미달 보고를 유지한다. 일반 시각 수용 조건을 충족했다고 주장하지 않고, 범위를 한정한 작업지시자의 병합 예외 결정을 기록한다.

수정은 넘긴 키와 무관한 공통 속성을 덮어쓰는 원인을 겨냥하고 있으며, 동일 code head의 신규 3건과 기존 5건의 초회 로컬 저장 회귀 검사는 통과했다. 이번 재검토에서 새 제품 결함을 실행으로 검출하지 않았다. 기여자는 초기 `조판 영향 비해당` 판단을 철회하고 PR 본문을 수정했다. 다만 새 저장 줄 정보 보정은 미push이며 제출 head의 직접 시각 증적은 여전히 미검증이다.

이전 보류 해제 조건은 동일 재현 HWP와 독립 한컴 Print PDF의 보존, 정확한 제출 head에서 Native/fresh WASM Visual Sweep 및 대표 PNG 직접 판독, 전체 범위의 쪽수·배치·내용 보존 확인이었다. 아래 최신 결정에서 원 FLAGS 수정만 먼저 병합하고 추가 편집·renderer 출력 검증은 후속으로 분리했다. 일반 gate는 최저 90% 이상이다. 기여자 제출용 전체 TSV 첨부는 최신 devel 가이드에 따라 **Native만 필수**이며, 내부 Native/fresh WASM 실행·결과 기록과 대표 이미지 요구는 유지한다. 이후 최신 head CI·merge simulation과 작업지시자 승인도 재확인한다.

## 접수와 라우팅

| 항목 | 확인값 |
| --- | --- |
| PR / 관련 이슈 | [#7608](https://github.com/edwardkim/rhwp/pull/7608) / [#7606](https://github.com/edwardkim/rhwp/issues/7606), `closes #7606` |
| 작성자 | sacru2red, rhwp 첫 PR, contributor fork |
| 원 head / source repository | `1452bf8845a299873a6b30fff7fe6f37003b2741` / `sacru2red/rhwp` |
| base | devel, 제출 base `d9749c5a15b95206455953494f6fc871c115cef1` |
| 검토 base | 초회 `a048da32d2b76867283aa6dbdf2578816a0e6238`; 2026-10-07 재검토 `6ed5c0909723289749b2b757ac4f02a604608a59` |
| 규모 | 1 commit, source 1개·신규 test 1개, +159/-7 |
| reviewer | edwardkim 지정 후 API 재조회 확인 |
| 상태 | 작성 시점 open/non-draft, mergeable=true, clean; merge 전 재조회 필요 |

- base route: `maintainer_general.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `first_time_contributor.md`, `visual_fixture_evidence.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서, `review_template.md`; Git·검증 환경 문서도 확인했다.
- 검토 worktree: `/tmp/rhwp-pr7608-review-20261006`, local branch `review/pr7608-20261006`. 기본 checkout은 devel을 유지했다.

## 원인과 구현 검토

`create_table_ex_native`의 TAC 생성은 `raw_ctrl_data` FLAGS에 `0x002A0301`을 넣지만 `Table.attr`에는 TABLE 레코드 값 `0x04000006`을 둔다(`object_ops/table.rs:990–1040`). 생성 표의 seal은 None이다. 기존 setter는 `Table.attr` 전체를 FLAGS에 써 너비·높이 기준 및 요청하지 않은 위치 속성도 바꿨다.

원 PR의 `touched_attr`는 8개 setter 분기의 수정 비트 범위만 모으고 `(raw_flags & !touched_attr) | (table.attr & touched_attr)`를 기록한다(`table_ops.rs:2827–2995`). FLAGS 범위를 읽을 수 없을 때 raw를 확장하지 않는 기존 계약도 유지한다. 표 생성 초기값을 바꾸거나 layout 값을 clamp하는 보정은 없다. 표시 키만 바꾸는 방식은 문제 범위에 맞는다.

검토한 실제 소비 경로:

1. 생성/속성 편집: `object_ops/table.rs:990–1040` → `table_ops.rs:2983–2995`의 raw FLAGS.
2. 저장: `serializer/control.rs:597–623`는 raw가 있고 seal이 허용하면 raw를 방출한다. 생성 표의 seal=None이 이 경로에 해당한다.
3. 재파싱: `parser/control.rs:149–159`는 FLAGS를 `table.common`으로 파싱하고 `table.attr = table.common.attr`로 동기화한다.
4. 흐름/측정: `renderer/typeset/controls/tac_flow.rs:131–138`의 일반 HWP TAC 분기는 `table.attr & 1`을 읽는다. `renderer/height_cursor.rs:710–716` 등의 흐름 분기는 `common.treat_as_char`, `text_wrap`, `vert_rel_to`를 소비한다.
5. 실제 배치: `renderer/layout/table_layout.rs:5258–5275`는 TAC와 floating 원점 선택을 구별하고 `common.horz_rel_to`를 읽는다. 세로 기준도 `:5501` 이후 실제 원점을 바꾼다.

따라서 편집 직후 IR 변경 방식이 같다는 사실만으로 저장·재열기 후 조판 영향을 비해당으로 판단할 수 없다. TAC 표에 `{}`를 넘긴 경로는 재파싱 후 TAC/배치 비트가 기존과 달라지고, native 생성 표의 가로 기준도 보존 대상이다. 너비·높이 기준의 한컴 동작은 이슈 보고를 참고했으며 reviewer가 실제 한컴 출력으로 재확인하지 않았다.

기존 생성부의 Paper/Page 주석·IR 불일치와 TAC `Table.attr` 불일치는 원 PR이 의도적으로 바꾸지 않은 범위다. 기존 코드의 상태를 이번 수정으로 새로 발생한 결함이라고 분류하지 않는다.

## 실행한 검증

| 항목 | 결과 |
| --- | --- |
| head 동일성 / CI 재사용 | source SHA와 CI run `37438370689`의 head가 정확히 같음; Build & Test·lint·4 archive worker 성공 |
| 별도 CI | 같은 head의 CodeQL analyze 3개·adapter inter-diff·proptest 성공; CI Impact Policy success |
| current-base merge simulation | `git merge-tree --write-tree upstream/devel upstream/pr7608-head` exit 0; tree `30c7e70dab3a7db586d1ed54067a8aef9e0c4316` |
| diff / fmt | `git diff --check upstream/devel...upstream/pr7608-head`, `cargo fmt --all -- --check` 통과 |
| 파생 suite / policy | `node scripts/rust-test-suite-manifest.mjs --prepare`; `--check --base-ref a048da32d2b76867283aa6dbdf2578816a0e6238` 통과, 48/48 targets |
| 신규 회귀 | `node scripts/run-rust-test.mjs issue_7606_tac_table_size_criterion -- --cargo-profile release-test --target-dir target/pr-review`: **3 PASS** |
| 기존 정상 대조군 | 같은 runner의 `issue_3552_table_common_attr_save`: **5 PASS**; HWP5/HWPX 속성 보존·다른 공통 속성 보존·raw 유지·무편집 byte 안정성 |
| 수정 전 FAIL | PR 본문에 3건 실패와 관측 attr/기준값이 기재되어 있음. reviewer는 수정 전 검사를 재실행하지 않았으며 기여자 보고로 구분함 |
| 광범위 로컬 회귀·Clippy | 새 code/test/fixture 보정이 없고 current-base merge가 clean하여 exact-head GitHub Full CI를 재사용; 로컬 전체 nextest와 Clippy 묶음을 반복 실행하지 않음 |
| Native Skia / fresh WASM / Visual Sweep | 원 CI Native Skia는 skipped. 이번 로컬 실행도 미실행. 해당 출력에 대한 통과 근거가 아님 |

로그와 API snapshot은 ignored `output/pr-review/pr7608-20261006/`에 보존했다. Cargo는 기본 저장소 `target/pr-review`를 symlink로 공유했고 별도 target을 만들거나 캐시를 삭제하지 않았다. host nextest 0.9.137은 권장 0.9.140보다 낮으며 설정의 JUnit report-skipped 키 경고가 있었다. 선택한 총 8건은 실제 실행되어 통과했고 skip 수는 필터로 제외된 suite의 다른 검사다.

## 검증 입력 커밋 확인

- 신규 3건은 코드에서 문서를 생성·직렬화·재파싱하므로 별도 fixture 파일은 사용하지 않았다.
- 기존 5건의 `samples/2010-01-06.hwp`는 검토 source SHA의 Git blob과 실행 파일이 byte-identical이었다. SHA-256: `d2562d9219fc1d491dd6b9f6d787314153246efb79e18c42c63830ac22194958`.
- 위 로컬 구조 보존 검사의 입력 확인은 **충족**이다. 실제 #7606 재현 문서와 독립 한컴 Print PDF는 PR/이슈에 포함되지 않아 시각 검증 입력은 **미검증**이다.

## 조판 원칙 준수 검토

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | 전체 FLAGS 덮어쓰기 제거, 호출자가 넘긴 비트 범위만 갱신; 샘플 ID·수치 예외 없음 |
| 측정·배치 일관성 | 미검증 | 저장·재파싱 후 소비 경로는 추적했으나 실제 영향 출력 비교는 없음 |
| 분할·이어받기 계약 | 비해당 | 직접적인 컷·예약 높이·분할 알고리즘 수정 없음; 재열기 후 실제 쪽 구성의 보존은 별도 미검증 |
| 줄 소속과 점유 높이 | 미검증 | 재파싱 TAC 비트가 흐름 분기로 전달되므로 새 저장본의 실제 줄/쪽 소속 증적 필요 |
| 사례와 독립성 | 미검증 | 내부 생성·roundtrip 계약과 기존 정상 입력은 통과; 한컴 기준 출력 없음 |
| 기준값 변경 | 비해당 | golden·baseline·허용치 변경 없음 |
| 주장과 검증 범위 | 미검증 | 초기 `조판 비해당` 설명은 기여자가 철회·수정함. 미push 편집 보정과 시각 증적은 제출 head에 없어 직접 확인하지 못함. 새 실행 검출 회귀로 보고하지 않음 |

초회 blocker 중 시각 영향 범위 분류는 기여자가 정정했다. 현재 blocker는 **제출 head의 필수 시각 증거 부족**이다. 저장 계약 검사가 통과한 사실을 실제 한컴 표 모양·위치 일치로 승격하지 않는다.

## 다음 절차와 원격 조치

기여자가 자기 branch에서 동일 재현 HWP와 한컴 Print PDF를 보존하고 영향 출력 및 전체 Native TSV를 첨부하도록 요청할 한국어 댓글 초안을 `output/pr-review/pr7608-20261006/contributor-comment-ko.md`에 준비했다. 대표 Native/fresh WASM review·standalone overlay는 정확한 제출 repository/head SHA의 raw URL을 사용한다. PDF 출력에는 메인터너 전용 MCP 접근을 요구하지 않는다.

작업지시자가 보완 요청 댓글 게시를 승인한 뒤, `gh pr comment 7608 --repo edwardkim/rhwp --body-file output/pr-review/pr7608-20261006/contributor-comment-ko.md`로 [한국어 보완 요청](https://github.com/edwardkim/rhwp/pull/7608#issuecomment-6016559860)을 게시했다. 게시 시각은 `2026-10-06T12:46:09Z`이며, API 재조회로 초안과 본문의 일치 및 한글·LF 줄바꿈·BOM/치환 문자 부재를 확인했다. 댓글 게시 전 원격 head는 원 검토 SHA와 같았다. reviewer 지정과 이 댓글만 원격에 반영했고 GitHub review/push/merge/close는 수행하지 않았다. 시각 판독이 없으므로 merge 후 시각 통과 댓글 계획은 작성하지 않았다. 기여자의 증적 보완 후 해당 head를 재검토한다.

## 2026-10-07 재검토 — 기여자 응답과 의존 작업

### 최신 상태와 실제 확인

- 라우팅은 maintainer 일반 + intake/local/visual/first-time/old-base로 유지했다. reviewer `edwardkim`은 이미 지정되어 있어 중복 assign하지 않았다.
- 원격 head와 fork branch는 계속 `1452bf8845a299873a6b30fff7fe6f37003b2741`, 1 commit / 2 files / +159 −7이다. 작성 시점 OPEN / non-draft / MERGEABLE / CLEAN이다. PR 본문은 수정됐지만 code head 변경은 없다.
- [Full CI 37438370689](https://github.com/edwardkim/rhwp/actions/runs/37438370689) attempt 2는 같은 head에서 success다. Build & Test·4 archive worker·lint·CodeQL analyze·adapter·proptest·CI Impact Policy success를 API로 다시 확인했다. Native Skia/WASM/Frontend gate는 skipped여서 출력 통과 증거로 쓰지 않았다.
- 최신 upstream/devel `6ed5c0909723289749b2b757ac4f02a604608a59`를 fetch했다. `git merge-tree --write-tree upstream/devel upstream/pr7608-head` exit 0, tree `2f83f62792e2afa75997c4ca3ca8e5a284110291`. `git diff --check upstream/devel...upstream/pr7608-head`도 PASS다.
- 기존 worktree와 review-only commit을 유지했다. `node scripts/rust-test-suite-manifest.mjs --prepare`, `--check --base-ref 6ed5c0909723289749b2b757ac4f02a604608a59`를 재실행해 PASS(48/48 targets)를 확인했다. 파생 산출물은 stage하지 않았다.
- 동일 code head·보정 없음이므로 초회 focused 3+5 PASS와 exact-head Full CI를 재사용했다. 이번에 Cargo·fresh WASM·Visual Sweep을 새로 실행한 결과로 쓰지 않는다. 원 제출 code/test를 다시 읽고 FLAGS mask 범위와 저장·재파싱 검사 범위를 확인했다.
- API snapshot과 이번 정책 로그는 ignored `output/pr-review/pr7608-20261007/`에 보존했다. source/test/fixture/baseline 보정 및 원격 comment/review/push/merge/close는 이번 재검토에서 수행하지 않았다.

### 기여자 보고와 검증 경계

[첫 응답](https://github.com/edwardkim/rhwp/pull/7608#issuecomment-6018285321)에서 초기 조판 비해당 판단을 철회했고 본문도 수정했다. 같은 생성 경로 전후 6건의 한컴독스 인쇄 PDF 대조를 보고했다. 보고값은 TAC `{}` 56.63%→88.84%, TAC 위치 변경 44.58%→49.26%, native `{}` 100%→100%다. 이는 **기여자 보고이며 reviewer의 직접 측정값이 아니다**.

[후속 응답](https://github.com/edwardkim/rhwp/pull/7608#issuecomment-6027688539)에서는 생성 TAC 문단 reflow/vpos, treatAsChar 실제 전환 때 재조판, 표만 든 floating 문단의 앵커 줄 보정을 미push 작업으로 보고했다. TAC `{}`는 진단 100%, native 대조군 100% 유지라고 했지만 TAC→floating 검사는 FAIL이며 전체 10505건 중 10504 PASS / 1 FAIL이라고 명시했다. 공개 head에는 이 보정이 없어서 호출 경로·테스트·독립 기준과의 직접 대조를 완료했다고 판정하지 않는다.

의존 작업은 다음처럼 구분한다.

| 작업 | 최신 공개 상태 | #7608과의 관계 |
| --- | --- | --- |
| [#7620](https://github.com/edwardkim/rhwp/issues/7620) | OPEN; issue에 RowBreak 표 28.21%, None 대조군 100% 보고 | 문단 기준 TopAndBottom 표의 바깥 여백 누락. 기여자 보고로 남기며 최신 devel에서 reviewer가 재현한 결과로 쓰지 않음 |
| [#7621](https://github.com/edwardkim/rhwp/issues/7621) | OPEN; 기여자의 [#7630](https://github.com/edwardkim/rhwp/pull/7630)이 이미 OPEN | 표만 든 문단 뒤 글자 문단의 한 줄 밀림. PR head `f3b26a6cf303589737ddc233bac915802421517b`를 조회했지만 이 PR의 정식 코드·시각 review 또는 수용을 수행한 것은 아님 |
| #7608 | 원 제출 head 그대로 | FLAGS 보존은 제출됨. LINE_SEG 보정·동일 편집 저장본의 PDF/대표 증적은 아직 제출되지 않음 |

#7630에는 자체 #7621 HWP/PDF·대표 PNG가 포함됐지만, 이 자료를 #7608의 서로 다른 편집 시나리오 기준 출력으로 대신하지 않는다. #7608에는 HWP/PDF/대표 PNG 신규 파일이 없으며 아직 직접 Native/fresh WASM 전체 TSV·직접 PNG 판독을 수행할 재현 자료가 보존되지 않았다. 따라서 미검증 상태를 유지한다.

### 제안하는 처리 범위와 다음 게이트

기여자에게 안내할 초안을 `output/pr-review/pr7608-20261007/contributor-reply-ko.md`에 준비했다. 이후 아래 작업지시자 결정에 맞춰 범위 수용과 담당 권고를 명확히 반영했다.

1. #7608은 FLAGS 보존과 그 생성/배치 전환에 필요한 저장 줄 정보 보정까지로 한정한다. renderer 결함은 #7620과 #7621/#7630에서 별도 처리한다. 기여자의 미push 구현을 아직 승인했다고 쓰지 않는다.
2. 한컴독스라는 제품명이나 cairo Producer만으로 기준 PDF를 거부하지 않는다. 동일 저장본·서버 인쇄용 PDF의 출처, 페이지 크기/배율, 원문 대응과 누락 없음을 확인한다. 데스크톱 한/글 2024 일치를 주장하는 범위는 별도 미검증으로 둔다. 서버 인쇄용 원본 PDF가 있으면 보존하고 브라우저 재인쇄본과 출처를 구분한다.
3. renderer 선행 변경이 반영된 최신 devel에서 기여자가 재검토·재조판하고 TAC `{}` / TAC→floating / native `{}` 세 시나리오의 새 저장본을 다시 출력한다. unchanged treatAsChar 호출의 저장 줄 보존과 정상 대조군도 포함한다. 사용한 원본·PDF를 제출 commit에 보존한다.
4. 새 code head에서 관련 전체 Native/fresh WASM 최저 90% 이상, 전체 쪽수·뒤 문단·표 여백·누락/중복, 대표 review/overlay 직접 판독을 확인한다. Native 전체 TSV와 exact-head Markdown image를 PR 본문에 갱신한 뒤 fresh head CI와 current-base merge simulation을 재확인한다.

기여자에게 전달할 처리 범위·순서의 승인 요청과 PR 자체의 merge 승인은 구분한다. 현재 원 head는 계속 **머지 보류**이며 이번에 새 GitHub approve나 merge를 요청할 상태가 아니다.

## 작업지시자 결정 — 편집 경로 수용과 renderer 담당 권고

2026-10-07 작업지시자는 “이 PR 은 수용하고, 두 렌더러 결함도 기여자가 맡는 것을 권고드립니다.”라고 지시했다. 직전 범위·담당·처리 순서 논의에 따라 다음 방향을 기여자에게 전달한다.

- #7608은 편집 경로(FLAGS 비트 보존 + 저장 줄 정보 보정)까지 포함하는 제안을 수용한다.
- #7620과 #7621도 동일 기여자가 별도 PR로 맡아 진행하도록 권고한다. #7621은 이미 제출한 #7630으로 이어간다.
- 기여자 제안대로 renderer 두 수정이 devel에 반영된 뒤 #7608을 rebase하고 세 시나리오의 새 head Native/fresh WASM 검증과 PR 본문·증적 갱신을 진행한다.
- 처리 방향 수용과 아직 제출되지 않은 구현·시각 검증 완료 판정은 구분한다. 이번 조치는 승인된 범위·담당 권고 댓글 게시이며 GitHub APPROVE 또는 merge 실행은 아니다.

댓글 게시 직전 PR은 OPEN / non-draft이고 원 head `1452bf8845a299873a6b30fff7fe6f37003b2741`이 유지됨을 확인했다. 게시 뒤 API 재조회로 본문·한국어·LF·BOM/치환 문자 부재를 확인하고 permalink를 여기에 기록한다.

[범위·담당 답변](https://github.com/edwardkim/rhwp/pull/7608#issuecomment-6037128007)을 `2026-10-07T11:39:40Z` 에 게시했다. API 재조회로 작성자와 본문의 완전 일치, 한국어·실제 LF, BOM/치환 문자 부재를 확인했다.

## 최신 결정 — 원 FLAGS 수정의 범위 한정 병합

작업지시자가 원래 FLAGS 보존 구현의 완료 여부를 확인한 뒤 “이 PR 은 머지로 처리를 진행하세요”라고 명시했다. 앞선 선행 renderer 수정 후 이 PR을 병합한다는 안내보다 이 최신 지시를 우선한다.

- 병합 대상은 원 code head `1452bf8845a299873a6b30fff7fe6f37003b2741`의 FLAGS mask 수정과 기존 신규 회귀 3건이다. 코드·테스트·baseline 보정을 추가하지 않았다.
- 신규 3건과 기존 정상 대조군 5건의 로컬 구조 검증 및 동일 exact-head Full CI 성공을 수용 근거로 사용한다. 직접 시각 검증은 미실행이고 기여자의 88.84%/49.26% 보고를 통과로 바꾸지 않았다. 새 합성 배치 회귀·golden·허용치 조정도 없다.
- 최신 devel `6ed5c0909723289749b2b757ac4f02a604608a59`를 다시 fetch하고 source head 동일성, CI `37438370689` attempt 2 success, PR MERGEABLE/CLEAN 및 current-base merge tree `2f83f62792e2afa75997c4ca3ca8e5a284110291` exit 0을 확인했다.
- 추가 LINE_SEG/문단 reflow/vpos 보정은 현재 head에 없으며 별도 후속 PR로 이어간다. #7620과 #7621도 기여자가 맡도록 한 권고를 유지하고 #7621은 #7630으로 진행한다. renderer의 해결이나 미push 구현의 검증 완료를 주장하지 않는다.
- 병합 후 댓글에서 이번 병합 범위와 별도 후속 제출로 바뀐 처리 순서를 명시한다. #7606 종료는 원래 크기 기준 FLAGS 덮어쓰기 결함의 해결 범위이며 후속 renderer 이슈의 종료와 구분한다.
- 운영 기록은 archive review와 오늘할일만 maintainer 직접 반영한다. source fork는 보존하고 후속 기록·댓글·duration 결과 확인 뒤 이번 검토 작업공간만 정리한다.

### Merge 후 contributor PR comment 계획

공개 저장 FLAGS 수정의 범위 한정 병합과 첫 기여 감사를 한국어 존댓말로 기록한다. exact code head와 실제 CI/로컬 저장 계약 검증, 직접 Native/fresh WASM 시각 검증 미실행, 기여자의 미달 보고와 남은 renderer/저장 줄 보정을 구분한다. 이전 안내의 renderer 반영 후 #7608 rebase 계획은 현재 원 PR의 병합으로 대체됐으므로, 추가 편집 보정은 최신 devel 기반 별도 PR에서 세 시나리오를 검증하도록 안내한다. 이번 수용은 시각 증적을 통과 근거로 사용하지 않아 없는 PNG·수치를 만들지 않는다. 실제 merge SHA와 후속 이슈 링크를 --body-file로 게시하고 API 재조회로 본문을 확인한다.

## 병합 후 기록

- [원 PR #7608](https://github.com/edwardkim/rhwp/pull/7608)을 2026-10-07 20:49:55 KST에 [merge commit `d0ad557e65428afb9e2b83bbde1a4a9bbc54ac71`](https://github.com/edwardkim/rhwp/commit/d0ad557e65428afb9e2b83bbde1a4a9bbc54ac71)로 devel에 병합했다. `--match-head-commit`으로 원 code head를 고정했다.
- 원 head의 [APPROVED review](https://github.com/edwardkim/rhwp/pull/7608#pullrequestreview-5441820555)에 범위 한정 수용, 실제 검증과 시각 미검증·후속 작업을 기록했다. 승인 본문을 시각 검증 통과로 해석하지 않는다.
- 후속 문서 처리: maintainer 직접 반영. 이 archive와 `mydocs/orders/20261007.md`만 운영 기록 commit으로 devel에 보존한다. root devel은 merge commit으로 fast-forward했다.
- [duration metadata run](https://github.com/edwardkim/rhwp/actions/runs/37616760831) success: `ready=true`, `successful-pr-worker-measurements`, source CI `37438370689` attempt 2의 B/C/D 실측을 수집해 46 target을 갱신하고 metrics branch `36dfe03b` push를 확인했다. [자동 이슈 종료 run](https://github.com/edwardkim/rhwp/actions/runs/37616760869)도 success이며 #7606은 20:50:08 KST에 CLOSED가 됐다. 병합 후 CI·CodeQL·Adapter·Proptest·Oracle을 시작하거나 재실행하지 않았다.
- 기록 반영 뒤 #7606 종료 상태와 관련 issue/PR 댓글을 확인·게시한다. 댓글은 원 FLAGS 수정 병합과 저장 줄 정보/renderer 후속 처리로 변경된 순서를 명확히 안내한다. renderer 이슈 #7620·#7621은 닫지 않는다.
- 후속 게이트가 완료되면 `/tmp/rhwp-pr7608-review-20261006`, `review/pr7608-20261006`, 임시 fetch ref `upstream/pr7608-head`만 정리한다. 해당 worktree의 review-only 기록은 이 archive로 보존한다. contributor `sacru2red/rhwp:fix/issue-7606-tac-table-size-criterion` 및 #7630 fork branch, 다른 검토 작업공간, 공유 target/pr-review는 보존한다.

현재 source head의 병합·자동 종료와 실측 갱신을 확인했고, user 승인에 따른 원 FLAGS 수정 수용 기록은 이 archive에 남겼다. 초기 보류·범위 확대 논의는 당시 검토 이력이며 최종 처리는 위 최신 결정과 병합 후 기록을 따른다.
