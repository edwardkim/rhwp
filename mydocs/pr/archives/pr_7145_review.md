# PR #7145 검토 — 문단 기준 자리차지 표의 세로 오프셋

## 접수와 범위

| 항목 | 확인 내용 |
|---|---|
| 원 PR | [#7145](https://github.com/edwardkim/rhwp/pull/7145), `planet6897`, devel 대상 |
| 원 head | `0eefe065f2c3f18bcc00f5f2d5f3f0bb406c6939` |
| 최신 base | `4fddb1bb7244fb478ea18f927aca273f93ac7322` |
| 체리픽 | `9f5a63a6ae296d02a5a94b17fc74c9d009e46c1d` |
| 검증한 누적 code head | `65df76e6195e0b87471d82359eb11c437d0bf1a0` (#7149 포함) |
| branch | `codex/pr7145-pr7149-review-20260915` |
| 규모 | 1 commit, 6 files, +224 −1; 제품 Rust 1파일, 테스트 1파일, 입력·보고서·속성 설정 |
| 접수 시점 참고값 | OPEN, non-draft, MERGEABLE/CLEAN, maintainerCanModify=true; reviewer `jangster77` 지정 |
| 관련 이슈 | [#6929](https://github.com/edwardkim/rhwp/issues/6929) |

기본 경로 `collaborator_external_pr`, 보조 경로 `intake_and_review`, `local_validation`,
`visual_fixture_evidence`, `multi_pr_update_branch`와 Visual Sweep 정본을 적용했다.
devel을 동기화한 뒤 보이는 로컬 branch를 만들고 #7145 → #7149 순서로 `cherry-pick -x`했다.
두 건 모두 충돌이 없었으며 원 저자와 source SHA를 보존했다. 원 PR의 current-base merge simulation도 clean이었다.

## 문제와 공통 조판 원칙

`compute_table_y_position`의 `raw_y.max(y_start)`가 앵커 문단 자신의 진행량을 선행 내용으로
취급해 선언 오프셋을 덮었다. 실제 입력에서 `94.5 + 433HU = 100.27px`인 표가 118.5px에
그려져 제목에 겹쳤다. 단이 비고 앵커가 단 최상단인 TOP/양수 offset 표에 한해 push 기준을
앵커로 바꾼다. 진입 시 단의 상태를 보존해 두 호출 경로에 같은 판단을 전달한다.

| 공통 항목 | 검토·판정 |
|---|---|
| 독립 근거·일반성 | 저장 오프셋과 새 한컴 PDF 괘선 측정이 일치한다. 문서명·페이지·문자열을 조건으로 사용하지 않는다. 단 최상단/빈 단은 선행 내용이 없다는 조건이다. |
| 선행 내용과 자기 예약 구분 | 제목이 먼저 배치된 단과 앞선 형제 표가 있는 단은 기존 push 기준을 유지한다. #1549·#1639·#2439 실제 DocumentCore 경로 반례 9개가 통과했다. |
| 측정·배치 공통성 | 높이 측정·줄 구성은 변경하지 않는다. 표 좌표 계산의 두 호출 지점 모두 동일한 진입 시점 상태를 사용한다. Native/WASM 18쪽 render tree가 일치한다. |
| 분할·이어받기 | 분할 컷·rowspan 소유·예약 높이 계산을 바꾸지 않는다. #2439 continuation 포함 반례와 뒤 페이지 보존을 확인했다. 새로운 분할 알고리즘의 일반성은 주장하지 않는다. |
| 높이·뒤 내용 | 표의 빈 마지막 행을 포함한 168.2px bbox와 제목 268.5px를 유지한다. 1쪽에서 해당 표를 제외한 render tree가 수정 전과 같다. |
| 기준값 | 기존 golden·래칫 행·허용치를 완화하지 않았다. 새 PDF의 쪽수 원장 한 행만 독립 PDF와 변경 전후 실측으로 추가했다. |
| 미검증 경계 | 문서 전체 한컴 동등성·전수 글꼴 일치를 주장하지 않는다. 기존 우측 표제 잘림·글꼴·굵기 차이 및 총 쪽수 차이는 남는다. |

## 시각 검증과 9.6px 설명 정정

macOS에서 [Visual Sweep](../../manual/verification/visual_sweep_guide.md)의 공통 webfont 경로로
Native 수정 전/후와 새 WASM을 같은 PDF에 비교하고 1·2쪽 compare 및 1쪽 overlay를 직접 확인했다.
WASM은 Chrome `152.0.7977.83`, 새 `--target web --no-opt` 빌드를 사용했다. 배포 최적화 빌드를
통과했다고 주장하지 않는다. 바이너리·WASM·PNG 해시는 [검증 JSON](../assets/pr7145_pr7149_validation.json)에 보존했다.

| 1쪽 대상 (96dpi px) | 수정 전 | 수정 후 | 새 한컴 PDF |
|---|---:|---:|---:|
| 상단 전폭 괘선 중심 | 118.5 | 100.3 | 100.211 |
| 하단 전폭 괘선 중심 | 277.2 | 258.9 | 258.757 |
| 제목 문단 상단 | 268.5 | 268.5 | 약 268.2 |

표 하단 괘선이 제목을 가로지르던 현상이 해소됐다. PDF의 vector drawing과 실제 비교 PNG가
같은 결론을 보였다. 표 노드 bbox의 바닥은 268.5px지만 마지막 가시 괘선은 258.9px다.
그 사이 9.6px는 괘선 없는 마지막 빈 행이다. 신고자의 후속 정정대로 **높이 결함이 아니며**,
원 보고서의 “남는 높이 차이” 설명을 메인터너 문서 보정으로 바로잡았다.

전체 Native SVG는 18쪽 중 1쪽만 바뀌었다. 2~18쪽은 수정 전과 byte-identical이며, 1쪽도 대상
표를 제외한 tree가 동일하다. Native/WASM은 전체 18쪽 tree가 동일했다. 기준 PDF는 17쪽이므로
기존 쪽수 차이를 해결 성과로 세지 않는다. 1쪽 우측 “공정한 시장경제…” 표제의 잘림·크기 차이,
기존 글꼴·굵기와 2쪽의 자형 차이도 수정 전후 유지됐으며 이번 위치 수정의 회귀가 아니다.

| Sweep | 쪽 | 자동 flag 유형 | pixel match | 잉크 기반 보조값 |
|---|---|---|---:|---:|
| 수정 전 Native | 1 | 없음 | 85.21480% | 24.37979% |
| 수정 후 Native / WASM | 1 | 없음 | 89.09665% | 38.55483% |
| 수정 후 Native / WASM | 2 | 없음 | 92.32893% | 31.23693% |

수치는 자동 보조 지표이며 기능 정확도나 한컴 동등성 비율이 아니다. 수정 전도 flag가 없었으므로
flag 0을 제목 겹침이 없다는 증거로 사용하지 않았다.

- [수정 전 1쪽](../assets/pr7145_before_p001.png)
- [Native 수정 후 1쪽](../assets/pr7145_native_p001.png)
- [WASM 수정 후 1쪽](../assets/pr7145_wasm_p001.png) · [overlay](../assets/pr7145_wasm_p001_overlay.png)
- [WASM 2쪽 대조](../assets/pr7145_wasm_p002.png)

## 입력 보존과 완료한 검증

입력 [HWP](../../../samples/issue6929/148776468_search_ad_terms_press_release.hwp)의 SHA-256은
`a05d313f34852e07305d8bf16223303d9cc00c3d9bdc0b5258f96dfcb347a43d`다. 원 PR이 추가한 파일을
그대로 재사용하며 다운로드 원본과 바이트 일치를 확인했다. 기존 Git 원본을 다른 이름으로 복제하지 않았다.

새 [기준 PDF](../../../pdf/issue6929/148776468_search_ad_terms_press_release-hwp-2020.pdf)는
17쪽, SHA-256 `61c28f0265bde456dea299a6793e00cd731fabe78cb1e258bb9ca5397d233495`다.
engine 선택·job·start/status/download·client/server 해시 일치는 [변환 기록](../../../pdf/issue6929/README.md)에 남겼다.
변환 당시 출력명에는 `-2020.pdf`를 사용했으며 Git 보존 전 canonical 형식 식별자 `-hwp-2020.pdf`로
정리했다. 바이트는 같고 두 PDF를 중복 추가하지 않았다.

| 검증 | 실제 결과 |
|---|---|
| 새 #6929 focused | 3/3 pass (206 skip) |
| #1549 / #1639 / #2439 반례 | 9/9 pass (601 skip) |
| Native CLI build / fresh WASM | 통과 / 통과 |
| 쪽수 원장 | 신규 행 `17 / 18`; 기존 행 변경 없음. 실제 generator를 해당 원본/PDF로 범위 제한해 생성, canonical 선택·전체 수록 확인을 유지. 16/16 partition pass (179 skip) |
| OVR 기존 5문서 | KTX / exam_math / 언어기출 / aift / biz_plan 모두 개체 회귀 0, exit 0 |
| 전체 포맷 / suite manifest / diff check | 통과 |
| [원 head CI](https://github.com/edwardkim/rhwp/actions/runs/34892522304) | API head SHA 일치, success. Lint, Native Skia, Archive A/B/C/D, frontend package 및 별도 required checks 완료 |

기준 바이너리는 앞서 검증한 `68af356c3` 계열의 동결 CLI다. `git diff 68af356 upstream/devel --
src Cargo.toml Cargo.lock crates build.rs`가 비어 있음을 확인했다. OVR 도구의 자동 head 표시는 실행
checkout SHA이므로 baseline 코드의 출처로 사용하지 않았고, 실제 지정한 baseline CLI의 hash를 보존했다.
메인터너 제품 Rust 보정은 없다. 원 PR CI를 확인해 광범위 전체 Rust·Native Skia 회귀를 로컬에서
중복 실행하지 않았으며, 통합 head가 이미 GitHub 전체 CI를 통과했다고 주장하지 않는다.
새 PDF·쪽수 원장 및 #7149 표기 보정이 포함되므로 제출 시 최종 통합 head의 게이트를 별도로 확인한다.

핵심 재현 명령:

```sh
node scripts/run-rust-test.mjs issue_6929_float_table_para_offset -- --cargo-profile release-test --locked --target-dir target/pr7145-pr7149-review-20260915
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr7145-pr7149-review-20260915 --test regression_suite_021 --test regression_suite_006 --test regression_suite_014 -E 'test(/(^|::)issue_(1549|1639|2439)::/)'
node scripts/run-rust-test.mjs oracle_page_count_baseline -- --cargo-profile release-test --locked --target-dir target/pr7145-pr7149-review-20260915
venv/bin/python scripts/visual_sweep.py --file-target issue6929 samples/issue6929/148776468_search_ad_terms_press_release.hwp pdf/issue6929/148776468_search_ad_terms_press_release-hwp-2020.pdf --rhwp-bin <candidate-cli> --wasm-pkg <fresh-web-pkg> --pages 1-2 --dpi 96 --out <output>
```

## 통합 PR #7162의 제출·CI 결과

[통합 PR #7162](https://github.com/edwardkim/rhwp/pull/7162)의 code candidate는
`d4e08890695e8ff2c874024514ef761317db35a8`이다. 제출 전 전체 포맷, native·WASM32·workspace
all-target Clippy, workspace build와 manifest 검사를 모두 통과했다.
[Full CI](https://github.com/edwardkim/rhwp/actions/runs/34953765482)의 Build & Test, Archive A/B/C/D,
lint, Native Skia, frontend package 및 Render Diff·Adapter·Proptest가 완료됐다.
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34953765647)은 attempt 2에서 성공했다.
최초 Rust 분석은 945개 파일 추출 오류 0, SARIF 생성 뒤 업로드 단계에서 실패했다. 명시적인 업로드
오류가 로그에 없어 근본 원인을 단정하지 않았고 실패 job만 재실행했다. 소스 수정은 없었다.
정확한 check/job URL과 결과는 [CI 증적](../assets/pr7145_pr7149_ci.json)에 보존했다.

2026-09-15 확인 시점에 원 candidate의 모든 check가 완료됐고 MERGEABLE/CLEAN이었다.
오늘할일·개별 review·CI 증적만 같은 PR의 single-parent trailing commit으로 추가한다.
사용자가 CI 완료 후 merge·후속 처리까지 승인했다. 최종 문서 head의 CI 재사용 및 최신 상태를
다시 확인한 뒤 merge하고, merge SHA·duration refresh·이슈/원 PR 종료 결과는 GitHub 후속 comment에
기록한다. 병합 뒤 검증 CI를 실행하지 않는다.

## 최종 판정

**승인** — 기준 PDF·시각 증적·측정 정정을 포함한 로컬 통합본 기준이다. #6929의 표 시작 위치와
제목 침범 문제는 해소됐고 제품 코드의 추가 메인터너 보정이 필요한 지적은 없다.
통합 PR 생성과 code candidate CI는 위와 같이 완료했다. 최종 문서 head의 CI·merge 및 원 PR/이슈 종료는
후속 단계이며 실제 결과를 미리 완료로 쓰지 않는다. [처리 계획](pr_7145_review_impl.md)에 순서를 남겼다.

## Merge 후 contributor PR comment 계획

[Visual Sweep comment 절차](../../manual/verification/visual_sweep_guide.md#github-merge-comment)에 따라
원 head, 체리픽 SHA, 통합 PR/merge SHA, 위 괘선·제목 수치 및 1·2쪽 지표와 잔여 차이를 함께 게시한다.
대표 이미지는 `mydocs/pr/assets/pr7145_wasm_p001.png`이며
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7145_wasm_p001.png`
형식으로 고정한다. 실제 merge 및 asset 반영 후 승인된 후속 단계에서 UTF-8 파일을 `--body-file`로
게시하고 API로 한글·링크를 재확인한 뒤 source PR과 #6929의 종료 상태를 확인한다. 현재는 계획이다.
