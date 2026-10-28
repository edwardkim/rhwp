---
kind: report
status: active
last_verified: 2026-10-04
---

# PR #7563 리뷰 — TAC 공통 줄과 Square 문단 흐름 보정

## 최종 판정

**승인.** 공통 TAC 줄 구성·Square 후속 흐름·legacy 커서 소유권의 명시된 부분 보정 범위로 판정합니다. 검토 head `e857df9b292d098bfb36926ab645b6fbe21fc249`의 CI 성공과 1,000줄 초과 변경의 별도 검토 cycle을 완료했습니다. 이 기록을 추가한 후행 head의 최신 CI 및 작업지시자의 최종 merge 승인은 별도 조건입니다. 50% 페이지 수용은 해결 범위 밖이며 미검증·미구현입니다. 사용자90% 예외와 남은 시각 차이는 유지합니다. 이 판정은 GitHub approve 또는 merge 실행이 아닙니다.

## 접수와 범위

- [PR #7563](https://github.com/edwardkim/rhwp/pull/7563), edwardkim, base `devel`; reviewer를 지정하지 않은 self-review 기록입니다.
- 최초 공개 head `659c940bd00b6c684493ba71781dda97fca726c7`, 검증 code head `0373fb43c4189a138482f72ebbb5ecb0a081b4a5`, 고정 검증 base `6b3faf77d8085441f9f26d88d65a49791e910352`입니다.
- 원 기여자 davindev의 [#7482](https://github.com/edwardkim/rhwp/pull/7482) head `kidsnote/rhwp@9bc8478d5ddf43dc87c06278b8c0deb8c7640959`를 조상으로 보존했습니다. fork push403 때문에 원 저장소의 `fix/pr7482-shared-tac-rows`로 공개했습니다.
- 이번 PR은 공통 TAC 줄 구성·Square 후속 제외 영역·legacy 후속 커서 소유권의 부분 보정입니다. 원 PR의 approve·close·merge, #7518, 관련 이슈 전체 종료는 수행하지 않았습니다.

## 검증과 조판 근거

[원 PR의 상세 검토](../pr_7482_review.md)에 생산→측정/예산→실제 배치 호출 경로, 독립 기대 관계, 수정 전 FAIL/후 PASS, 적용/비적용 경계를 연결했습니다. 입력 생성·독립 PDF job/hash는 [fixture README](../../../samples/issue7482/README.md)와 [provenance](../../../samples/issue7482/provenance.json)에 있습니다. 원본/수동 NO_LS 변형을 구분하며, 50% 진단을 정상 저장본의 일반 규칙 증거로 승격하지 않았습니다.

- fmt·Native/WASM/workspace all-target Clippy·workspace build·base 고정 manifest/unit-tier PASS입니다.
- focused24/24(정식 TAC/Square/legacy15 + #6970 경계9), 전체 nextest10,265/10,265 PASS(50 skipped,792.835s)입니다.
- Native Skia lib workspace4,109 PASS(rhwp3,927;13 ignored), missing picture2/2, direct PDF4/4 PASS입니다.
- root wrapper fresh WASM SHA-256 `91654b0814b6cadc38225cf9eaab48ef1e3a35d9a7ad2b08cd7a02761ed4d95b`와 pkg/public/실제 browser 응답이 일치했습니다. CDP18문서/45검사 PASS,pageErrors0입니다. 추가 #6970 원본은 기존 fixture bytes를 CDP 응답으로 공급해 실제 loadHwpFile/WASM/canvas를 확인했습니다.
- 후행 기록 commit은 mydocs만 바꾸며, 검증한 Rust source/test/fixture/baseline에 변경을 추가하지 않습니다. local 성공은 최신 remote CI를 대신하지 않습니다.

## 시각 증적과 남은 차이

Native/fresh WASM 각각22문서27쪽 review PNG·대표 standalone overlay를 직접 판독했습니다. 최신 PR 본문에 정확한 head SHA의 대표28PNG를 Markdown 이미지로 표시합니다. [PNG와 입력/PDF hash](../assets/pr7482_shared_rows_20261004/provenance.json), [실행 집계](../assets/pr7482_shared_rows_20261004/validation-summary.json)를 사용합니다.

같은 줄/너비 부족/개행의 표·텍스트 순서와 뒤 문단, Square 옆22문단과 p2 큰 TAC의 단일 소속, #6970 p3 오른쪽 글줄의 독립16px 간격을 확인했습니다. 마지막 경계는 immutable c8에서1run FAIL하고 현 source에서 PASS했습니다. 전역gate·기존baseline/golden·래칫은 완화하지 않았으며, 사용자90% 예외는 #7482를 보정하는 이번 부분 제출에만 적용합니다. distribution p3 85.83661%,masked p2 43.1166%,regulatory HWP/HWPX p38 77.99388/78.00141%,form p2 65.49734%,#6970 p3 74.11148%의 raw gate와 남은 차이를 보존합니다. 완전 시각 일치로 판정하지 않습니다.

## 후속 조건

이 문서의 후행 head CI와 mergeability를 확인하고 작업지시자의 최종 merge 승인을 받아야 합니다. 대형 변경의 별도 검토 결과와 실제 merge/comment 후보는 아래와 같습니다. 원 PR과 관련 이슈의 종료는 부분 해결 범위에 맞춰 별도로 판단합니다.

## CI 성공 후 별도 검토 cycle (2026-10-04)

기본 경로는 collaborator self-merge이며, intake/local validation/visual fixture evidence/review-only fast-pass/대형 PR/post-merge 보조 문서를 적용했습니다. 작성자 self-review이며 다른 사람의 독립 review로 표시하지 않습니다. 기존 source/test 20파일 +1,820/-136을 `upstream/devel` 대비 읽고, 상세 주장·원인별 실패 증거·실제 최종 좌표 assertion을 [원 검토](../pr_7482_review.md#생산과-소비-경로)와 대조했습니다. #7478 serializer 수정은 검증 base와 원 #7482에 이미 포함됐고 이 PR과 변경 파일이 겹치지 않습니다.

### exact head CI와 병합 시뮬레이션

head `e857df9b292d098bfb36926ab645b6fbe21fc249`의 check 35개는 success31/skipped4, 실패·대기0이었습니다. [Full CI](https://github.com/edwardkim/rhwp/actions/runs/37162026923)의 Build & Test, lint, Native Skia, archive A/B/C/D와 [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/37162026886), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/37162026683), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/37162026955), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/37162026863)는 성공했습니다. skipped인 WASM Build를 실행된 검사로 표시하지 않으며 실제 웹 검증은 위 fresh WASM/CDP 증거입니다. `CI Impact Policy` status도 success입니다.

최신 fetch base `6b3faf77d8085441f9f26d88d65a49791e910352`에서 `git merge-tree --write-tree <base> <head>` exit0, tree `999363ced625a5ef1bfb443bfb15086df653d00e`를 확인했습니다. API에서 Open/non-draft/mergeable=true/clean이며 branch protection required `Build & Test` 성공을 확인했습니다. 실제 merge 직전에 다시 조회합니다.

검증 code `0373fb43c` 이후 Rust source/test source는 동일합니다. 후행 기록에 신규 oracle 항목 `tac-after-plain-paragraphs.hwpx 2/2`가 추가된 차이는 별도로 확인했습니다. 전체 PR baseline diff는 신규11입력 등록뿐이며 기존 행·허용치의 변경은 없습니다. 신규2/2는 동일 입력의 독립 PDF 전2쪽과 위 direct review/정식 소속 검사에 연결되고 최신 exact-head Full CI에서도 검증됐습니다. 이 차이를 문서-only로 분류하지 않습니다.

### 조판 주장과 실제 소비 경로 대조

| 항목 | 판정 | 별도 cycle의 실제 확인 |
| --- | --- | --- |
| 구현 근거·일반성 | 충족 | 너비/여백/순서/개행을 반영한 물리 줄 구성. 저장 줄/탭/캡션/중첩/각주/기존 standalone RowBreak owner의 적용 제외를 `supports_table_text_rows`에서 확인; 문서 ID에 따른 새 배치 예외 없음 |
| 측정·배치 일관성 | 충족 | `plan/build_plan`의 rows/boxes/end → `commit_inline_flow`의 current_height와 column metadata → `layout.rs` FullParagraph 조기 소비 → `layout_inline_flow_plan` 최종 좌표. Square는 `use_square_host_plan`의 text-fit/object-fit → `record_square_host_flow`의 원점 변환 → host-plan paint와 `plan.end`; 이후 legacy max 덮어쓰기가 비적용임을 확인 |
| 줄 소속·점유 | 충족 | 같은 줄/너비 부족/개행/텍스트 혼재/외부 여백 및 뒤 빈 문단의 최종 bbox assertion을 대조. 기준선 최대와 descent 최대를 결합하며 같은 줄의 표 높이를 세로 합산하지 않음 |
| 분할·이어받기 | 비해당(신규 컷 변경) | 공통 whole-row 계획이 페이지 예산에 맞을 때만 확정. 큰 혼합 문단·standalone 여러 행 RowBreak 등은 기존 fragment owner로 반환. 이번 승인에 해당 기존 경로의 모든 조합 또는50% 수용 정책을 포함하지 않음; p38/원 #2319의 최종 표·뒤 내용 보존은 기존 정식 증거로 대조 |
| legacy/공통 cursor 경계 | 충족 | `typeset_inline_flow`는 마지막 실제 item의 공통 plan.end가 current_height를 잇는 경우에만 unchanged plain rows를 계속 소유. #6970 새 사례의 c8 FAIL/현재 PASS 및 독립12pt=16px pitch를 대조 |
| 입력·독립 근거·baseline | 충족(제출 범위) | 원본/수동변형 구분과 input/PDF hash, 입력11개의 신규 oracle 등록을 대조. 원본 저장 정보를 수동변형으로 완화하지 않음;50% 적용조건 및 전체 시각 일치는 제외 |
| 주장과 검증 한계 | 충족(부분 범위) | 기존 focused24/full10,265/lint/Skia/CDP45 및 Native/fresh WASM 각27쪽의 exact-code 증거 재사용. 새 코드 변경 없이 전 회귀를 반복하지 않음. 이번 cycle에서 대표5개 review PNG를 다시 직접 열어 같은 줄·개행·Square 옆22문단·다음 쪽 TAC·#6970 글줄 비겹침과 남은 차이를 확인 |

실행 로그/API/merge receipt는 로컬 ignored `output/pr-review/davindev-20261003/merge-readiness-7563/`에 보존합니다. 필수 증거가 없는 범위를 새로 충족 처리하지 않았으며, 이번 별도 cycle에서 새 실행 회귀 또는 제출 범위의 추가 blocker를 확인하지 않았습니다.

## Merge 후 contributor PR comment 계획

작업지시자의 최종 승인과 최신 head CI 성공 뒤, davindev 원 commit/저자를 보존하는 일반 merge commit 방식으로 #7563을 병합합니다. `--admin` 예외를 사용하지 않습니다. merge SHA 확인 → PR에 이미 포함된 기록/asset 확인 → 안전한 local devel fast-forward → 관련 issue 상태 조회 → 승인 범위의 댓글 → 안전 조건을 충족한 PR 전용 branch/worktree 정리 순서입니다. 원 #7482는 별도 처리 대상으로, 승인 없이 close/approve/merge하지 않습니다.50% 미구현을 남기며 관련 이슈 전체 종료를 주장하지 않습니다.

원 기여의 TAC 후속 겹침 진단·Square 제목 복원 성과를 먼저 인정하고, 여러 TAC/개행/혼재·Square 후속 제외 영역·legacy cursor에 추가 보정이 필요했던 이유와 독립 좌표 검사를 설명합니다. #7563과 승인 시 원 #7482에 같은 부분 수용 사실을 알리되 같은 merge/증적의 기존 댓글이 있으면 중복하지 않습니다. API로 UTF-8·실제 줄바꿈·본문/이미지 SHA를 다시 확인하는 `--body-file` 경로를 사용합니다.

실제 확인한 범위는 Native/fresh WASM 각22문서27쪽과 대표28PNG이며, 댓글에 모든 쪽이 완전히 일치한다고 쓰지 않습니다. 같은 줄 mixed TAC Native p1 100.0%, 명시적 개행 mixed TAC WASM p1 100.0%, Square follower Native p1 100.0%/WASM p2 99.43386%, #6970 Native p3 74.11148%를 사용합니다. #6970의 오른쪽 뒤 글줄은 독립16px 간격으로 겹침이 해소됐지만 왼쪽 단의 기존 줄바꿈·장식 차이는 남고 #7482 전용90% 예외임을 명시합니다.

대표 이미지5개의 안정 경로는 `mydocs/pr/assets/pr7482_shared_rows_20261004/` 아래 `tac-mixed-one-row_native_p001_review.png`, `tac-mixed-explicit-break_wasm_p001_review.png`, `square-follower_native_p001_review.png`, `square-follower_wasm_p002_review.png`, `square-columns_native_p003_review.png`입니다. merge 뒤 실제 asset 존재와 해시를 확인한 후 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7482_shared_rows_20261004/<file>.png`를 Markdown 이미지로 표시합니다. [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 함께 연결합니다. 페이지·지표·원 점수와 제한을 위 계획 그대로 유지하며 임시 output에서 추정하지 않습니다.
