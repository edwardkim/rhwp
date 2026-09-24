---
kind: review
status: active
---

# PR #7260 3차 검토 — 기존 변경 요청 해결

최종 판정: **승인**. 검토 head `4c9efab2c1065757c5b25ebf96e1d0a76a102aeb`가 유지됐고 최신 CI가 성공했다. 이전 두 변경 요청은 해결됐으며 추가 코드 수정 요청은 없다. 사용자 승인으로 GitHub Approve 리뷰를 게시했다. 이번 후속 commit은 검토 기록만 반영하며, 최신 문서 head의 CI·승인 상태를 확인한 뒤 병합은 작업지시자가 직접 수행한다.

## 대상과 검토 범위

- PR: [#7260](https://github.com/edwardkim/rhwp/pull/7260), 기여자 @lpaiu-cs, 이슈 #6639.
- 기여자 회신: [재리뷰 대응](https://github.com/edwardkim/rhwp/pull/7260#issuecomment-5810804854).
- 검토 head: `4c9efab2c1065757c5b25ebf96e1d0a76a102aeb`; 제품·테스트 commit: `cf3e9fc1da3ba9cc949eb94ae6ddfb61f9baba5a`.
- 이전 검토 head: `96092453b70ed762363e13374ba674863f0d4dd4`; 두 새 commit만 증분 검토했다. 새 diff는 제품 3파일·회귀 1파일·README·PNG 4개다.
- base: `505661360e9a2d596f55300d0cb0c5222f0e14b4`. `git merge-tree --write-tree upstream/devel HEAD` 성공, 충돌 없음. `git diff --check` 통과.
- 처리 경로: collaborator 외부 contributor 재검토. 주 작업공간의 사용자 PDF 변경을 보존하기 위해 별도 임시 worktree를 사용했다. contributor source·baseline을 수정하지 않았다.
- [이전 변경 요청 기록](https://github.com/lpaiu-cs/rhwp/blob/96092453b70ed762363e13374ba674863f0d4dd4/mydocs/pr/archives/pr_7260_review.md)은 당시 판정의 이력이다.

## 기존 요청의 판정

| 항목 | 판정 | 확인 근거 |
| --- | --- | --- |
| 반복 편집의 RowBreak 원점 소실 | 충족 | 최초 서식 flush에서 `Paragraph.cell_vpos_reset`에 경계 여부를 저장한다. 이후 수치 역행 유무가 달라져도 `cell_vpos_resets`가 저장한 경계를 우선하며 텍스트 편집 ladder도 조각 시작점부터 계산한다. 합성 batch/eager·반복·snapshot·경계 문단 텍스트 왕복 검사 PASS. |
| 정상 저장본에서 반례 유효성 | 충족 | 기존 `samples/task2430/1382000_domestic_violence_survey.hwp`의 RowBreak 셀 `(0,93,0,0)` 문단 76/77은 `64680/64462`다. 140% 편집 뒤 역행이 사라지는 조건과 두 번의 복원 후 문단 77의 `64462` 보존을 정식 테스트로 재실행했다. |
| 원본의 편집·복원과 기존 시각 차이 구분 | 충족 | contributor의 PR 도입 전 비교 자료와 코드 경로를 대조했다. reviewer의 현재 Native 초기·140%·모양 복원 SVG SHA가 기여자 기록과 모두 일치한다. 높이도 `1009.1200 → 1024.5737 → 1025.3067px`로 같고 전체 snapshot은 초기 SVG로 복귀한다. |
| 성능 지적의 유지 해결 | 충족 | 새 경계 캡처는 기존 flush의 순회 안에서 수행한다. setter마다 셀 전체를 추가 순회하지 않는다. #4118 batch 동등성 PASS. 속도 개선 배수는 재계측하지 않았다. |

수정 전 제품 `e5135e3f`에서 신규 두 검사가 FAIL했다는 자료는 기여자 README의 실행 증거를 검토했다. 이번 reviewer 실행은 최신 head의 PASS이며 수정 전 제품을 다시 빌드한 실행과 구분한다.

원본의 줄 구성·표 높이 차이는 [입력 README](../../../samples/issue6639/README.md)의 도입 전 비교와 일치한다. `formatting.rs`의 raw stream 무효화 → `Document::layout_profile`의 `session_edited` → `HeightMeasurer`의 미편집 TAC 저장 높이 축소 해제 경로를 확인했다. 뒤의 두 파일은 PR 도입 전과 바뀌지 않았다. 모양 ID 복원은 편집 상태 전체를 되돌리는 API가 아니며, PR 본문도 복원 주장을 모양 ID·문단 vpos로 한정했다. 이 근거로 이전 보류를 해제할 수 있으며 한컴 시각 일치를 승인한 것은 아니다.

## 조판 원칙 준수 검토

| 검토 항목 | 판정과 근거 |
| --- | --- |
| 독립 입력·기대값 | 충족. 실제 한컴 저장 HWP의 원래 조각 원점을 기준으로 반복 편집의 보존을 검사한다. 합성 계약과 실제 문서를 구분했다. |
| 생산·소비 경로 | 충족. 서식 setter의 dirty → batch 종료 flush → 경계 캡처 → 공통 ladder → 기존 dirty/rebuild/pagination을 대조했다. 텍스트 편집에서도 같은 저장 경계와 조각 slice를 사용한다. |
| snapshot과 경계 수명 | 충족(검사 범위). snapshot은 document clone으로 표식을 보존한다. 새 분할 문단은 continuation, 셀 폭 재래핑은 기존 경계를 false로 해제한다. 분할·폭 변경 분기는 코드 확인이며 이번 focused 검사의 새 별도 실행 사례로 세지 않는다. |
| 문서별 보정·수치 임계값 | 충족. 새 문서 ID 분기·좌표 clamp·baseline 완화 없음. |
| pagination 컷·예약 높이·종료 변경 | 비해당. 해당 알고리즘을 변경하지 않으며 경계 문단 좌표 갱신만 수정했다. 실제 RowBreak 문서의 한컴 140% 편집 화면·전체 페이지 분할 정합은 별도 미검증이다. |
| 보이는 결과 | 충족(변경 범위). 원본 1쪽의 Native/fresh WASM compare·standalone overlay·review를 직접 판독하고, 140% 및 모양 복원 결과도 기준 PDF와 비교했다. 표 외곽과 추가 줄바꿈 차이는 남는 기존 제한으로 기록한다. |
| 주장하지 않은 범위 | 미검증. Studio UI undo stack 전체 동작, 실제 RowBreak 문서의 한컴 편집 출력 일치, 저장·재열기 후 편집 세션 경계 메타데이터 보존. |

## reviewer가 직접 실행한 검증

실행 checkout은 검토 head 그대로이며 진단 example 등록에 사용한 임시 Cargo 변경은 빌드 뒤 원복했다. macOS arm64, Rust 1.93.1, 96dpi, canonical Chrome webfont 비교다.

| 검사 | 결과 |
| --- | --- |
| #6639 `issue_6639_cell_para_format_vpos` | 10/10 PASS, 새 실제·합성 경계 검사 포함 |
| #4118 `issue_4118_cell_format_batch_deferral` | 1/1 PASS |
| manifest 정책 검사 | base `50566136` 고정, PASS |
| fresh WASM | 현재 checkout, `--dev --no-opt`, 성공; 배포 최적화 빌드의 대체 검사는 아님 |
| 브라우저 실제 편집 | 초기·140%·모양 ID 복원·snapshot 복원 SVG 모두 Native와 byte 동일 |
| 원본 140% 마지막 문단 vpos | `[32760,34020]` |
| 전체 snapshot 복원 | 최초 SVG와 byte 동일 |
| fidelity | `--text-only --export-all-svg --layout-ledger` 완료 |
| Visual Sweep | Native/fresh WASM 각각 원본 1쪽 완료, compare·overlay·review 직접 판독 |
| 입력 commit 일치 | 원본 issue6639 HWP, 실제 task2430 HWP 및 PDF 3개가 검토 head Git blob과 byte 동일 |

SVG SHA-256:

- 초기·전체 snapshot: `87b0960611b6983a044acaf564559e7804e1f191f8f64baeeb5c4b3751c70801`
- 140%: `fa0ff3a16e57fbc06ed5c64528b93597cba9e65e5f869601bcbec5447c56758a`
- 모양 ID 복원: `bc7e098eb0928f00c3bedf61c999918c66b98f29e408ab086231eb669564a69b`
- fresh WASM: `74e201fab05949eb0f3b47ef52a191aa060875e5d0bf9837bcdd818edd0b60c4`
- WASM JS: `a75560f1a0619ccc206444ec9da320b057375f11773611e90b41ce3d9c218175`

재실행 명령은 README의 기존 절차를 따른다. reviewer의 focused 명령은 `node scripts/run-rust-test.mjs <위 test 이름> -- --cargo-profile release-test --target-dir <고정 target/pr-review>`다. WASM은 `CARGO_TARGET_DIR=<같은 target> scripts/wasm-pack-locked.sh --target web --out-dir <임시 wasm-pkg> --dev --no-opt`로 빌드했다. 두 sweep은 기존 검토 기록의 명령에 최신 checkout·바이너리·WASM 경로를 전달했다.

로컬 임시 로그·SVG·비교 이미지는 `/private/tmp/pr7260-round3-evidence/`에 있다. 원본 1쪽은 `sweep-native/original160/`과 `sweep-wasm/original160/`, 실제 편집 비교는 `edited-after/`, 복원 비교는 `edited-restore/`다. 140%와 복원은 canonical 비교 helper로 비교했고 편집 상태 render-tree heuristic은 재실행하지 않았다. 자동 flagged 0이나 픽셀 점수를 한컴 일치 통과 근거로 사용하지 않는다.

영구 대표 이미지는 이번 contributor commit에 있는 [140% 비교](../assets/pr_7260_rereview_fixed140.png), [모양 복원 비교](../assets/pr_7260_rereview_restore160.png), [도입 전 140% 비교](../assets/pr_7260_rereview_base140.png), [최신 원본 WASM 비교](../assets/pr_7260_rereview_original160.png)를 재사용한다. 환경 차이 때문에 Windows의 raster 점수와 reviewer macOS 수치를 직접 비교하지 않는다.

## CI와 다음 원격 조치

2026-09-24 재확인: head는 `4c9efab2c1065757c5b25ebf96e1d0a76a102aeb` 그대로이며 Ready, MERGEABLE이다. CI 확인 시 GitHub reviewDecision은 CHANGES_REQUESTED였으며, 이후 아래 Approve 리뷰를 게시했다.

| 정확한 head의 CI 근거 | 결과 |
| --- | --- |
| [CI / Build & Test](https://github.com/edwardkim/rhwp/actions/runs/35976587145) | completed / success. 회귀 shard A–D, lint(fmt·clippy·WASM check), Native Skia, Frontend package gates 성공 |
| [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/35976587059) | Canvas visual diff 성공 |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/35976587205) | Rust·Python·JavaScript/TypeScript 분석 성공 |
| [Proptest](https://github.com/edwardkim/rhwp/actions/runs/35976587218) | prop roundtrip 성공 |
| [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/35976587241) | 성공 |
| [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/35978296717) | 성공 |

WASM Build job과 일부 비적용 job의 SKIPPED는 실행 성공으로 세지 않는다. fresh WASM 빌드와 브라우저 시각 검증은 위 reviewer 로컬 실행으로 확인했다. 실패·대기·진행 중인 check는 없다.

제품·test·fixture·workflow·baseline·asset 보정을 reviewer가 추가하지 않았고 current-base merge도 clean하므로, local_validation 4.3.0에 따라 이 정확한 head의 GitHub 전체 CI를 재사용한다. 전체 release-test·Native Skia 광범위 회귀 및 lint를 로컬에서 중복 실행하지 않았다. 기여자의 전체 회귀 수치를 reviewer 실행 결과로 바꾸어 쓰지 않는다.

이전 요청의 해결과 최신 CI 성공을 근거로 사용자 승인 후 [Approve 리뷰](https://github.com/edwardkim/rhwp/pull/7260#pullrequestreview-5302281216)를 게시했다. API 재조회로 APPROVED 상태, 검토 SHA, 한글 본문의 초안 일치를 확인했다. 사용자가 검토 문서의 source branch push와 최종 병합 전 확인까지 승인했다. 이 문서만 single-parent trailing commit으로 반영하며, 새 head의 review-only fast-pass·required aggregate·mergeability·승인 상태를 확인한다. 병합은 작업지시자가 직접 수행하므로 reviewer는 merge·issue close를 실행하지 않는다.
