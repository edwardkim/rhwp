---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-14
---

# PR #7107 인라인 표 바깥여백·문단 여백 검토

**최종 판정: 승인.** 최신 devel 충돌을 해소한 현재 원 PR head의 가로 정렬 수정에 대해
머지를 막을 결함을 발견하지 못했다. GitHub에 제출하는 approve와 실제 merge는 별도 단계다.
페이지 전체의 한컴 출력 일치를 승인한 것은 아니다.

## 검토 대상과 출처

| 항목 | 확인값 |
| --- | --- |
| PR / 작성자 | [#7107](https://github.com/edwardkim/rhwp/pull/7107) / LJYeon12 |
| 제목 | fix: 인라인 표 정렬 시 바깥여백 및 문단 여백 계산 보정 |
| reviewer | jangster77, 정식 검토 중 요청 등록을 확인 |
| contributor 원 head | `7a805793476ad8361d4eee87119ff1dcec43b329` |
| 검토 code head / 충돌 해결 commit | `6e5dc6f52b3b5d5c14333a734d95e628cbd77f7c` |
| 최신 base | `d2fc85a9df92bd6a7e2c367bd703d8fad3ef489b` (`upstream/devel`) |
| fork / source branch | `LJYeon12/rhwp` / `fix/inline-table-outer-margins-ci` |
| 로컬 가시성 branch | `review/ljyeon12-pr7107-20260914` |
| 원 코드 diff | 5개 파일, 텍스트 +188/-12, 신규 합성 HWPX 1개 |
| 조회 시점 상태 | OPEN, MERGEABLE/CLEAN, required checks 성공; merge 전 재조회 필요 |

LJYeon12의 조회 가능한 PR은 #7107·#7109이며 이전 merged PR은 없다. 첫 기여자 절차를 적용한다.
일반 fork API의 `push:false`와 `maintainerCanModify:true`는 별개다. 앞선 사용자 승인에 따라
현재 source branch로 충돌 해결 commit을 실제 push한 사실을 확인했다.

관련 [#3396](https://github.com/edwardkim/rhwp/issues/3396)와
[#3410](https://github.com/edwardkim/rhwp/pull/3410)은 TAC 표의 외곽 여백을 포함한 advance 계약의
선행 근거다. [#6601](https://github.com/edwardkim/rhwp/issues/6601)과
[#6604](https://github.com/edwardkim/rhwp/pull/6604)는 선언 폭을 공유하는 별도 표 전용 경로와
관련된다. 이번 수정으로 이들의 모든 잔여 현상이 해결됐다고 주장하거나 추가 close 대상으로 삼지 않는다.

## 코드 판독과 독립적인 기대값

1. `paragraph_layout.rs:4606`의 `tac_offsets_for_line_width`가 선택한 동일 줄의 TAC에만 표의
   좌우 바깥여백을 더한다. 해당 함수는 명시적 개행·다음 줄 시작·말미 TAC 중복 여부를 판독한다.
   문단 전체의 표를 한 줄로 합산하지 않는다. 그림·도형의 기존 여백 계산은 변경하지 않는다.
2. 이 폭을 `estimate_line_run_widths`와 최종 정렬 폭이 소비한다. paint의
   `x += tac_w + tac_table_om.0 + tac_table_om.1`과 점유 폭이 일치한다.
   공통 `flow_width_hu`를 변경하지 않아 여백을 이미 계산하는 표 전용 문단 경로의 중복 합산을 피한다.
3. `paragraph_layout.rs:4958`의 새 판정은 셀 안, 저장 시작 0, 왼쪽 문단 여백 0,
   양수 오른쪽 여백, 저장 폭이 `셀 가용 폭 - 문단 오른쪽 여백`과 1 HU 이내로 일치하는 경우다.
   이 경우 저장 폭을 다시 열 폭으로 사용하지 않아 오른쪽 여백을 두 번 빼지 않는다.
   더 좁은 저장 구간이나 Square-wrap 근거는 기존 경로를 유지한다. 특정 문서명·ID를 검사하지 않는다.
4. 충돌 해결은 최신 devel의 `physical_frame_rows || (...)`와 위 예외를 함께 보존했다.
   저장 물리 행 경로의 우선순위와 여백 0 처리도 유지했다. contributor test 기대값을 추가 수정하지 않았다.

기존 `issue_1285`의 기대값 변경은 실제 저장 PDF의 괘선으로 독립 확인했다.
`pdf/21_언어_기출_편집가능본-2022.pdf`는 Hwp 2022 **12.0.0.4426**,
Hancom PDF **1.3.0.550** 메타데이터를 갖는다. 1쪽 수험번호 표의 수평 괘선 끝점은
`599.343994pt × 96/72 = 799.125326px`다. 세로 괘선 x는 `599.104980pt`로
수평선 끝점과 다르므로 두 좌표를 혼동하지 않았다. 테스트의 1px 허용치를 확대하지 않았다.
합성 fixture의 x=144..432 계약은 문서 치수로 정한 것이며 한컴 출력 실측으로 취급하지 않았다.

## CI와 로컬 검증의 구분

사용자가 **CI에서 회귀테스트가 끝났으므로 별도 회귀테스트는 불필요**하다고 지시했다.
이번 정식 검토에서 Cargo/Native Skia/OVR 회귀테스트를 재실행하지 않았다.
검토 source·test·baseline을 변경하지 않았으며 다음 exact-head CI를 재사용했다.

- [Full CI 34834644226, attempt 2](https://github.com/edwardkim/rhwp/actions/runs/34834644226):
  `headSha=6e5dc6f52b3b5d5c14333a734d95e628cbd77f7c`, conclusion success.
  Archive A/B/C/D는 각각 3,873 / 2,420 / 1,207 / 2,164건으로 **9,664 PASS, 51 skipped**다.
- 변경 직접 검사 5건(신규 3건 + issue_1285 2건)도 Archive D의 PASS 로그를 확인했다.
  [CI 요약](../assets/pr7107_ci_summary.txt), [직접 검사 로그](../assets/pr7107_ci_focused.txt).
- Rust lint, Native Skia, frontend package, Build & Test 성공.
  Native Skia는 workflow에 지정된 필터·suite 실행이며 전체 Skia lib 무필터 실행을 뜻하지 않는다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34834644228),
  [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34834643884),
  [Adapter](https://github.com/edwardkim/rhwp/actions/runs/34834644519),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/34834644231), CI Impact Policy 성공.
- 앞선 충돌 해결 단계에서는 같은 코드의 fmt, native/WASM/workspace Clippy, workspace build,
  manifest, unit-tier와 focused 13건을 통과했다. 이는 이번 회귀 재실행이 아닌
  [이전 단계 기록](../assets/pr7107_conflict_validation.json)이다.

실제 브라우저 시각 확인용 새 WASM만 빌드했다. Docker daemon 연결이 불가능해 macOS host의
`--no-opt`를 사용했다. Rust release 최적화 빌드는 성공했지만 Docker/wasm-opt 표준 빌드 및
성능 검증 성공으로 확대하지 않는다. [빌드 로그](../assets/pr7107_wasm_build.txt).

```sh
CARGO_TARGET_DIR=target/pr7107-conflict-20260914 scripts/wasm-pack-locked.sh   --target web --out-dir /private/tmp/rhwp-pr7107-conflict-20260914/review/pkg --no-opt
venv/bin/python scripts/visual_sweep.py   --file-target answer-sheet samples/21_언어_기출_편집가능본.hwp pdf/21_언어_기출_편집가능본-2022.pdf   --rhwp-bin target/pr7107-conflict-20260914/debug/rhwp   --wasm-pkg /private/tmp/rhwp-pr7107-conflict-20260914/review/pkg   --pages 1 --dpi 96 --out /private/tmp/rhwp-pr7107-conflict-20260914/review/wasm
```

## Visual Sweep 판독

macOS / Chrome 152.0.7977.83 / 96 DPI / Studio 공통 webfont를 사용했다.
새 WASM의 `renderPageSvg`와 같은 문서의 `getPageRenderTree`로 출력·기하를 각각 얻었다.

| 범위 | 실제 관찰 | 판정 |
| --- | --- | --- |
| 답안지 1쪽 수험번호 표 우단 | 이전 Native 약 803.3px → candidate Native/WASM 약 799.8px; PDF 수평선 끝 799.125px | 가로 위치 오차 약 4.2px → 0.7px, 기존 1px 계약 충족 |
| 답안지 1쪽 변화 위치 | render tree의 차이 51개 값은 성명/수험번호 표와 host 줄의 x·w에 한정; y·높이 변화 없음 | 이번 가로 수정과 일치 |
| 답안지 2~15쪽 | before/candidate Native render tree JSON 동일 | 추가 변화 관찰 없음; PDF 전 페이지 시각 일치를 뜻하지 않음 |
| 답안지 1쪽 Native·WASM | 표 좌표 동일; 후보 페이지 0/1, pixel_match 88.12173%, visual_accuracy_proxy 12.44823% | 전체 글꼴·세로 배치 잔차가 크다. 후보 0을 전체 일치로 판정하지 않음 |
| 충돌 해결 단계 NO_LS 대조 | synth 3쪽 SVG가 이전 devel renderer와 byte-identical; 1쪽 후보 1/1, pixel_match 82.89834%, proxy 11.48821% | 기존 차이를 보존한 증거이며 한컴 일치/새로운 개선 증거 아님 |

before renderer는 `2f59c89373f497068f9a0bcb2c22730ec7dc7e51`이다. `src/**`는 최신 base
`d2fc85a9`와 동일하지만 Cargo 의존성 버전은 다르므로 exact-base binary라고 표기하지 않는다.
원 기여자가 제시한 `f537df5`도 직전 부모 음성 대조라고 확대하지 않는다.
Native 전후 tree 전체 차이는 [51개 값과 페이지별 결과](../assets/pr7107_tree_delta.json)에 남겼다.

[이전 Native/PDF](../assets/pr7107_inline_table_before_review_p001.png) ·
[현재 Native/PDF](../assets/pr7107_inline_table_native_review_p001.png) ·
[현재 WASM/PDF](../assets/pr7107_inline_table_wasm_review_p001.png) ·
[현재 WASM/PDF/OVL](../assets/pr7107_inline_table_wasm_review_p001_ovl.png).

검토자가 비교/OVL 이미지를 직접 확인했다. 수평 경계 개선과 별도로 성명/수험번호의 기존 세로 오프셋,
제목·본문 폰트와 본문 흐름 차이는 남는다. 전후 tree에서 해당 y/높이 변화가 없어 이 PR의 신규 결함으로
분류하지 않았으며, 이 PR의 가로 정렬 수용을 위해 baseline을 완화하지 않았다.

## 공통 조판 원칙 준수 검토

| 항목 | 판정 | 근거·제한 |
| --- | --- | --- |
| 독립 근거와 일반 규칙 | 충족 | PDF 괘선·문단 여백 계약으로 설명하며 문서 ID 분기·좌표 clamp 없음 |
| 측정/배치 공통 결과 | 충족 | 같은 줄 TAC 귀속을 재사용하고 정렬 폭에 paint와 같은 표 외곽 여백 반영 |
| 줄 소속·높이 | 충족 / 높이는 비해당 | 줄 소속 helper를 보존; 줄 높이 산식 변경 없음 |
| 경계·대조 사례 | 충족 | CI에서 3방향 정렬, 0/비대칭/음수 여백, NO_LS 별도 경로, 문단 여백 4/12px 검사 |
| 기대값 변경 | 충족 | 기존 PDF 독립 좌표 확인, 1px 허용치 유지; 합성 계약과 한컴 근거 분리 |
| 실제 시각 근거 | 충족 | 발동 1쪽의 전후/PDF/새 WASM OVL 확인; 전 문서 fidelity는 미검증 |
| 성능·표준 Docker WASM | 미검증 | 성능 개선 주장 없음, host fallback만 확인 |

문단 왼쪽 여백이 있는 다른 저장 LineSeg 계약과 임의의 모든 wrap 조합을 새 분기가 해결한다는 주장은
하지 않는다. 이번 PR은 중첩 표 전체 줄 나눔이나 모든 문서의 fidelity를 바꾸는 수정이 아니다.
새 차단 결함은 없으며 merge 전에는 실제 최종 head CI, base·mergeability 및 작업지시자 승인을 확인한다.

## 검증 입력 커밋 확인

**판정: 충족.** 실제 사용한 아래 파일의 로컬 바이트가 모두 code head `6e5dc6f52`의 Git blob과
일치했다. 새 이름으로 복사한 HWP/HWPX/PDF는 없다. 답안지는 기존 samples/PDF,
NO_LS 대조는 기존 issue_6970 fixture/한컴 2020 PDF, 숫자 표는 contributor 합성 fixture다.
PDF 생성 OS/폰트 전체 정보는 저장 메타데이터만으로 확인되지 않는다.

| 경로 | SHA-256 |
| --- | --- |
| `samples/21_언어_기출_편집가능본.hwp` | `905454045ca2e236839a7cab59750678116d08af3db31dbf846819af355b8d15` |
| `pdf/21_언어_기출_편집가능본-2022.pdf` | `f2d858d7974393661d91a658e6b384b951114ef52783379f426a963effd97b72` |
| `tests/fixtures/inline_table_outer_margins/two_digits.hwpx` | `3b1629e42dbd4f2ac78340b362a2a8e91d0c1c5f3a6149092ad526589e915dd0` |
| `tests/fixtures/issue_6970/synth_no_ls_square_wrap.hwp` | `31a5b76148718d92e3fd150f4d68b84b5005b02d16863c4735a6178eb02f2799` |
| `pdf/pr-planet6897-20260914/synth-no-lineseg-hwp-2020.pdf` | `70b63c11514d3d927418c1f116a0cc421614e7db5e9de312bd0d9f964b34e9a3` |

입력 Git blob, PDF 메타데이터·괘선 좌표, exporter/WASM 해시, run manifest,
PNG 해시는 [기계 판독 증적](../assets/pr7107_review_evidence.json)에 보존했다.

## Merge 후 contributor PR comment 계획

- [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md)을 직접 연결하고,
  위 답안지 1쪽 후보 0/1, pixel_match 88.12173%, proxy 12.44823% 및 가로 위치 개선/기존 잔차를 함께 알린다.
- 대표 PNG는 `mydocs/pr/assets/pr7107_inline_table_wasm_review_p001_ovl.png`다.
  devel 반영 후 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7107_inline_table_wasm_review_p001_ovl.png`
  형식의 확정 SHA 링크를 사용한다. 현재 계획 단계의 placeholder URL은 게시하지 않는다.
- 첫 기여 감사, contributor의 정렬 수정과 maintainer의 최신 devel 충돌 해결을 구분한다.
- merge/게시 승인 뒤 UTF-8 본문 파일과 `gh pr comment --body-file`을 사용하고 API로 게시 내용을 재조회한다.
  이 계획을 미리 게시한 것으로 기록하지 않는다.
- 현재 review/오늘할일/증적의 원격 반영과 최종 head CI 확인, 실제 merge 및 후속 처리는 남아 있다.
  상세 단계는 [실행 기록](pr_7107_review_impl.md)을 따른다.
