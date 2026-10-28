---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7200_review.md
last_verified: 2026-09-17
---

# PR #7200 검토

## 최종 판정

**승인 — 메인터너 보정으로 머지 보류 사유 해소.**
정상 저장본에서도 빈 선행 줄의 공간을 지우던 실제 배치 fallback을 보정했다.
원본 width 합성 파일은 정상 한컴 저장본으로 갱신하고, 부실 저장 복구 검사는 메모리에서 캐시를
손상시키는 별도 경계로 유지한다. 잘못된 한컴 겹침 PDF를 복구 렌더링의 정답으로 요구하지 않는다.
최종 Native/fresh WASM 20쪽을 재캡처했으며 backend별 PNG가 모두 동일하다.
원 PR에 보정 code head `10107f443`을 직접 push했고 Full CI·CodeQL·Render Diff·Adapter·Proptest가 통과했다.
이 문서는 merge 전 검토 기록이다. review·오늘할일 trailing head의 검사와 실제 merge SHA·duration
결과는 PR의 최종 검사 및 merge 후 댓글로 확인한다.

진행 기록: [1회차](../../working/task_m100_7200_stage1.md),
[2회차](../../working/task_m100_7200_stage2.md), [3회차](../../working/task_m100_7200_stage3.md).
PDF 버전 제한 제거는 `fbc14758f`에 별도 커밋했다. PDF 1.4 및 `Hwp 2020 0.0.0.0`은
보류 사유가 아니다. 이번 empty 변환의 실제 한컴은 `11.0.0.9136`이며 앞선 width의
`12.0.0.4605`와 구분한다. 선택 MCP engine은 둘 다 `2020`이다.

사용자가 비공개 실제 7쪽 양식 대신 **함께 커밋된 HWP 파일로 대체**하도록 지정했다.
비공개 원본 부재 자체를 보류 사유로 삼지 않는다. 그 원본 7쪽 개선은 reviewer 미검증이다.

## 원 PR 직접 push와 검증 계보 (2026-09-17)

- 사용자 지시로 별도 통합 PR을 만들지 않고 **PR #7200**의
  `LJYeon12/rhwp:feat/disability-page7-local`에 직접 반영했다.
- 원 source `2543382e757967183be1938893ac2c5be2c0c16e`는 그대로 조상으로 유지했다.
  미공개 메인터너 커밋만 같은 로컬 branch에서 그 위로 replay했다. 원 기여자 commit의
  rebase/amend/reset이나 force-push는 하지 않았다.
- fork API의 `permissions.push=false`, `maintainerCanModify=true`를 확인했고, LFS 대상
  0개/미전송 LFS object 0개를 판독한 뒤 `GIT_LFS_SKIP_PUSH=1` dry-run과 실제 push가 성공했다.
  remote ref와 PR head가 모두 `10107f443b3a701663891e91678509c28b206ecc`임을 재확인했다.
- 실제 `refs/pull/7200/merge`는 `7d946a5e499f2f6e85edd1479055b51720ba5bdf`이며 부모는
  최신 devel `222c8c4819de414898bf5b15608b0d0d394a3748`과 보정 head `10107f443`이다.
  tree `cc9b35a428c6a68fcbcc0fda8a92e7281be53a14`는 전체 로컬 검증을 마친 `6dd78f9e5`의
  tree와 **완전히 동일**하다. PR API의 오래된 base SHA 표기만으로 테스트 대상을 추정하지 않았다.
- [Full CI 35122939525](https://github.com/edwardkim/rhwp/actions/runs/35122939525),
  [CodeQL 35122939551](https://github.com/edwardkim/rhwp/actions/runs/35122939551),
  [Render Diff 35122939151](https://github.com/edwardkim/rhwp/actions/runs/35122939151),
  [Adapter 35122939491](https://github.com/edwardkim/rhwp/actions/runs/35122939491),
  [Proptest 35122939988](https://github.com/edwardkim/rhwp/actions/runs/35122939988): 모두 SUCCESS.
  CI preflight는 `fast_pass=false`, `no-trailing-review-only-commits`로 Full 검증을 실행했다.
  CI의 WASM Build skip은 fresh WASM 성공으로 세지 않으며 로컬 실제 빌드/실행 증거를 사용한다.

### 로컬 기록 SHA와 공개 원 PR SHA 대응

아래 및 fixture README에 남은 SHA는 로컬 통합 검토 당시의 값이다. 원 PR에서 과거 입력·실패 PDF·PNG를
재현할 때는 오른쪽 공개 commit의 **같은 경로**를 사용한다. 각 공개 commit과 위 최신 devel의
자동 merge tree가 해당 로컬 commit tree와 동일함을 5건 모두 확인했다.
예를 들어 원래 손상 width/empty 입력과 PDF는 `f76afb6d4`의 같은 파일 경로에서 복원할 수 있다.

| 로컬 통합 검토 SHA | 공개 원 PR SHA | 내용 |
| --- | --- | --- |
| `281795d20` | `4f302475ca879e99a711d93dd1d9d329854f8b53` | 최초 검토·빈 줄 실패 증거 |
| `4ce8d253c` | `8de8745a0a5c4f8dd2608faec30374dc60234f0c` | 1회차 보정 |
| `fbc14758f` | `7c395188169587207a78c96976326f87522f7c69` | PDF 버전 제한 제거 지침 |
| `78fd17cf8` | `f76afb6d459bf613eafc94034b201fc340512a14` | 2회차 보정·과거 합성 입력 보존 |
| `6dd78f9e5` | `10107f443b3a701663891e91678509c28b206ecc` | 최종 앵커 보정·입력 정상화 |

## Merge 후 contributor PR comment 계획

최종 trailing head의 검사 성공과 merge를 확인한 뒤 실제 merge SHA·최종 CI URL·duration 갱신 결과를
한국어로 설명하고 감사한다. [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을
연결하고 다음 자료를 **merge SHA에 고정한 raw.githubusercontent.com URL로 이미지 삽입**한다.

- 대표 review: `maintainer_width_top_review_001.png`, `maintainer_empty_top_review_001.png`.
- 실제 standalone overlay 6개: `maintainer_width_{top,center,bottom}_overlay_001.png`,
  `maintainer_empty_{top,center,bottom}_overlay_001.png`.
- 모두 `mydocs/pr/assets/pr7200_review/`에 있다. 비교 범위 20쪽·자동 후보 0/20·Native/WASM 동일
  20/20 및 아래 실제 pixel/ink 지표 6개를 함께 기록한다. 자동 보조값은 사람의 정확도 판정이
  아님을 설명하고, 정상 입력의 5줄·빈 줄·표/End/테두리 판정과 남은 래스터 차이를 구분한다.
- 정상 저장 앵커의 22.69px 오류 보정과 손상 캐시/정상 저장본의 별도 검증을 설명한다.
  비공개 원본 7쪽은 reviewer 미검증이며 기존 #2470 2쪽 사진/행 높이와 #7150은 이번 PR의
  해결·종료 근거가 아님을 명시한다. 관련 closing issue가 없으므로 새 issue를 임의로 닫지 않는다.
- UTF-8 본문 파일과 `--body-file`로 게시하고, 이미지 URL·한국어·실제 head/merge SHA를 API로
  다시 확인한다. 병합 후 검증 CI는 실행하지 않고 duration 갱신 결과만 확인한다.

## 3회차 — 최종 입력 보정과 정상 저장 앵커 보존

### 원인과 실제 수정

- 1·2회차는 저장 vpos가 reset된 경로를 공통 sequential 계획으로 보정했지만, 정상 저장 경로의
  `has_preceding_text=false` fallback은 이미 계산한 문단 원점을 다시 `inner_area.y`로 덮어썼다.
  빈 문단의 1688HU 공간이 지워져 Top의 중첩 표가 **22.69px 위**에 놓였다.
- `src/renderer/layout/table_layout.rs`에서 이 fallback이 `para_y_before_compose`를 소비하도록
  고쳤다. 새 clamp·허용 오차·문서 ID 예외는 없다. 앞 글자가 있는 flow와 reset 공통 계획은 유지한다.
- width 원본은 200자/1개 줄 캐시인 반면 재조판 테스트는 5줄을 요구했다. 정상 재생성본을
  **기존 samples 경로에 반영**하고 별도 fixture 폴더의 같은 HWP 3개는 제거했다.
  독립 PDF는 이미 있는 `width-*-recomposed-2020.pdf`를 재사용했다. 잘못된 원본과 실패
  `pr7200-width-*-2020.pdf`·PNG는 `78fd17cf8`에 보존하며 현재 입력의 정답으로 쓰지 않는다.
- `damaged_width_nodes`는 현재 정상 5줄을 확인한 뒤 메모리에서 1개 줄/0 앵커로 손상시켜
  기존 복구·정렬 계약을 계속 검사한다. 원문을 정상화하면서 손상 캐시 검사를 없애지 않았다.
- empty 입력도 저장 줄만 제거해 한컴에서 HWP→HWPX로 정상 저장하고 **최종 HWPX에서 PDF**를
  출력했다. 원문·PDF는 기존 경로를 갱신했다. 기존 16px 합성 경계는 `cell-*-collapsed.hwpx`를
  메모리에서 빈 문단으로 만드는 별도 검사로 유지한다. 생성 job과 해시는
  [empty 기록](../../../tests/fixtures/pr7200_empty_leading_paragraph/README.md),
  [width 기록](../../../tests/fixtures/pr7200_hancom_recomposed/README.md)에 있다.

### 수정 전후 검사와 직접 판독

`saved_empty_leading_paragraph_preserves_pdf_table_and_border_positions`는 동일한 최종 HWPX와
PDF로 수정 전 FAIL / 보정 후 PASS다. 표 상단·End 위치·22.55px 문단 테두리를 직접 검사한다.
수정 전 Top y=127.20px, 독립 PDF y=149.891px였고, 보정 후 세 정렬 모두 0.6px 위치 기준을
통과했다. 관련 focused 7개 PASS에는 정상 width·손상 캐시·빈 줄·overlay·글자 테두리도 포함한다.

| empty 정렬 | 보정 후 표 y / PDF y (px) | 보정 후 End y / PDF y (px) |
| --- | ---: | ---: |
| Top | 149.7 / 149.891 | 169.7 / 169.904 |
| Center | 191.8 / 191.961 | 211.8 / 211.975 |
| Bottom | 233.9 / 234.033 | 253.9 / 254.047 |

rhwp 표시는 render-tree의 소수 첫째 자리 직렬화 값이다. 정식 검사에서는 반올림 전 값을 사용한다.

수정 전 정상 empty Top: [compare](../assets/pr7200_review/before_saved_empty_top_compare_001.png) ·
[overlay](../assets/pr7200_review/before_saved_empty_top_overlay_001.png) ·
[review](../assets/pr7200_review/before_saved_empty_top_review_001.png).

| 최종 입력 1쪽 | pixel / ink match (%) | 최종 fresh WASM 증적 |
| --- | --- | --- |
| width-top | 97.83054 / 8.27243 | [compare](../assets/pr7200_review/maintainer_width_top_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_width_top_overlay_001.png) · [review](../assets/pr7200_review/maintainer_width_top_review_001.png) |
| width-center | 97.84435 / 10.37476 | [compare](../assets/pr7200_review/maintainer_width_center_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_width_center_overlay_001.png) · [review](../assets/pr7200_review/maintainer_width_center_review_001.png) |
| width-bottom | 97.78025 / 9.25153 | [compare](../assets/pr7200_review/maintainer_width_bottom_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_width_bottom_overlay_001.png) · [review](../assets/pr7200_review/maintainer_width_bottom_review_001.png) |
| empty-top | 97.73281 / 7.95094 | [compare](../assets/pr7200_review/maintainer_empty_top_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_empty_top_overlay_001.png) · [review](../assets/pr7200_review/maintainer_empty_top_review_001.png) |
| empty-center | 97.69089 / 9.52964 | [compare](../assets/pr7200_review/maintainer_empty_center_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_empty_center_overlay_001.png) · [review](../assets/pr7200_review/maintainer_empty_center_review_001.png) |
| empty-bottom | 97.65313 / 6.25195 | [compare](../assets/pr7200_review/maintainer_empty_bottom_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_empty_bottom_overlay_001.png) · [review](../assets/pr7200_review/maintainer_empty_bottom_review_001.png) |

직접 판독으로 width의 5줄·뒤 표/End·Top/Center/Bottom, empty의 빈 줄 점유·표 원점·후행
테두리가 최종 원문에 대응하는 PDF와 맞는지 확인했다. 빈 문단의 약 6px 테두리 차이도 정상
저장 메트릭과 실제 원점을 함께 반영한 최종 입력에서 해소됐다. 폰트 글리프 상자와 가는 선의
래스터 차이는 남으며 **픽셀 전체 일치라고 판정하지 않는다**. PDF 벡터는 0.24pt(96DPI에서
0.32px), rhwp도 약 0.3px이다. 낮은 ink 점수를 감추거나 선을 임의로 두껍게 만들지 않았다.

같은 실행에서 여백 경계 4쪽, overlay 3쪽, 날짜 표 3쪽, 실문서 4쪽도 다시 캡처했다.
전체 20쪽 Native/fresh WASM PNG가 동일하며, empty를 제외한 17쪽은 2회차 결과와 바이트가 같다.
#2470 2쪽의 기존 사진/행 높이 차이 등은 새 개선으로 보고하지 않는다. 이 PR의 수정으로
발생한 차이가 아니며 #7150 해결·종료 근거로도 사용하지 않는다.

### 최종 검증 계보

`78fd17cf8` 위 작업 트리의 아래 source 해시에서 빌드했다. 검증 후 source 변경은 없다.

| 최종 게이트 | 결과 |
| --- | --- |
| focused | 7 PASS, 정상 empty 경계는 수정 전 FAIL / 보정 후 PASS |
| release-test 전체 nextest | **9946 PASS / 0 FAIL / 51 skipped**, exit 0, 실행 302.072초 |
| fmt / Native·WASM·workspace-all-targets Clippy | 모두 PASS |
| workspace build / 고정 base suite 정책 | PASS, 48/48 targets |
| Native Skia lib | **4112 PASS / 13 ignored**, exit 0 |
| Native Skia 그림 / direct PDF export | **2 PASS / 4 PASS**, 각각 exit 0 |
| Native / fresh WASM Visual Sweep | 20쪽 각각 재캡처, 페이지 PNG 20/20 바이트 동일 |
| 최종 원문 보안 검사 | 36개 × hidden-text/injection(include-fields)/unicode = **108 clean** |
| 문서 검사 | 변경 문서 5개 상대 링크, review 메타데이터 PASS |

전체 Cargo 게이트는 `stage3/gates.sh`로 전용 target에서 순차 실행했다. 선택된 새 samples 28개는
전체 회귀의 `RHWP_SECURITY_SWEEP_SAMPLES_JSON`에 전달했다. fixture까지 포함한 36개는 최종
CLI로 추가 직접 검사했다. source-side unit test를 바꾸지 않아 unit-tier 증분 검사는 비해당이다.
원 head CI를 이 보정 source의 결과로 재사용하지 않았다. 이후 직접 push한 code head의 Full CI 성공은 위 원격 검증 절에 기록했다.

| 대상 | SHA256 |
| --- | --- |
| `src/renderer/layout/table_layout.rs` | `222c0af37fe5874719307f4ad70265ab8d09e93e82b11b7046b40c7083feba03` |
| `tests/cases/stored_nested_content_flow.rs` | `fd5b3ea5441fac52322205f77dec9c8027949faaceeca1d8a0f18e630e3f43e6` |
| Native CLI | `4aedd219dde5fb388fed736e296719b44930185b90b0e486869d6cbc63abbcbb` |
| fresh `rhwp.js` | `707049ae519de26779c842eec40f94e016b09bda2688fa721a37d2ae670c2f26` |
| fresh `rhwp_bg.wasm` | `2ddb8a82a3819ed37f97a685381a4bbdccf82b97c7db3493942ef661533f2dfe` |

raw 로그/JSON/TSV는 `/private/tmp/rhwp-pr7200-review-20260916/stage3`에만 두고 커밋하지 않는다.
`sweep.py native|wasm`은 `visual_sweep.py --file-target ... --pages 1`과 실문서 `--pages 1,2`를
실행한다. 96DPI/threshold32를 유지하고 compare·standalone overlay·review를 모두 생성했다.

## 1·2회차 기록의 해석

아래는 보정 전 입력과 이전 source에 대한 당시 검토다. 최종 판정은 위 3회차를 따른다.
3회차에서 갱신한 width/empty 입력과 PNG를 과거 입력의 통과 증거로 읽지 않는다.
과거 입력·이미지의 정확한 쌍은 `78fd17cf8`의 이 문서와 같은 commit의 파일로 재현할 수 있다.

## 원 PR 검토 대상과 계보 (보정 전)

| 항목 | 확인값 |
| --- | --- |
| 원 PR | [#7200](https://github.com/edwardkim/rhwp/pull/7200), 저장 위치가 초기화된 중첩 표 정렬과 표 뒤 문단 흐름 수정 |
| 작성자 / reviewer | LJYeon12 / jangster77 사전 할당 완료 |
| 기여 이력 | #7107 병합 이력이 있어 첫 기여자 처리 대상 아님 |
| 원 head | `2543382e757967183be1938893ac2c5be2c0c16e` |
| 검토 base | `222c8c4819de414898bf5b15608b0d0d394a3748` |
| 로컬 branch | `codex/pr7200-review-20260916` |
| 체리픽 직후 code head | `25e0d8c747ff6f38117c5ef1899b4caad50f3a52` |
| 체리픽 | `a2c1f814e` → `4e8c087a0`, `2543382e7` → `25e0d8c74`, 모두 `-x`, 충돌 없음 |
| 범위 | 엔진 4파일, focused 1파일/3함수, 합성 입력 28개와 README; baseline 변경 없음 |
| 원격 재확인 | 원 head 불변, OPEN/non-draft, CLEAN; maintainer_can_modify=true |

route: collaborator_external_pr + intake_and_review + local_validation + visual_fixture_evidence.
fork 직접 push 가능 여부는 이번 검토에서 dry-run하지 않았다. maintainer 수정 허용과 실제 push 성공은 구분한다.

## 원 PR 검토 당시 P2 — 실제 배치가 사용하지 않는 빈 문단 높이를 정렬 측정에 더함

변경 위치: `src/renderer/layout/table_layout.rs:8408–8419`.
새 `calc_nested_controls_bottom_height`는 저장 vpos 사다리가 무너진 셀에서 일반 문단 높이를
`flow_y`에 무조건 더한다. 그러나 같은 파일의 실제 중첩 표 배치 `:6787–6793`은
`has_preceding_text == false`이면 누적 `para_y` 대신 `inner_area.y`를 사용한다.
따라서 첫 문단이 빈 문단이면 측정에만 그 줄의 높이 16px가 포함된다.
이 과대 측정이 Center 여유를 8px, Bottom 여유를 16px 줄인다.

[재현 입력 3개와 생성 내역](../../../tests/fixtures/pr7200_empty_leading_paragraph/README.md)은
원 PR `cell-*-collapsed.hwpx`의 첫 `Start` 텍스트만 비웠다. 저장 줄과 표 구조는 유지했다.
Top의 실제 중첩 표 하단이 179.3px, 부모 셀 하단이 286.0px이므로 여유는 106.7px이다.
정렬 이외에는 동일한 입력이므로 Center는 53.35px, Bottom은 106.7px 이동해야 한다.
이는 원 PR이 사용한 Top 점유 영역 기반 기하 계약이며, 합성 파일의 한컴 외형 기대값이 아니다.

| 실행 | Top 표 y | Center 표 y / 이동량 | Bottom 표 y / 이동량 | 판정 |
| --- | ---: | ---: | ---: | --- |
| base Native | 126.0 | 179.4 / 53.4 | 232.7 / 106.7 | PASS, exit 0 |
| PR Native | 126.0 | 171.4 / 45.4 | 216.7 / 90.7 | FAIL, exit 1 |
| PR fresh WASM | 126.0 | 171.4 / 45.4 | 216.7 / 90.7 | FAIL, exit 1 |

[재현 검사 스크립트](../assets/pr7200_review/check_empty_leading_paragraph.py)는 성공 시 0,
이 회귀 시 1을 반환한다. 허용 오차 0.6px는 기존 PR assertion과 같다. 예외 허용치를 늘리지 않았다.
보류 해제에는 측정과 배치가 같은 실제 콘텐츠 배치 결과를 사용하도록 보정하고 이 경계의
수정 전 FAIL / 보정 후 PASS 및 기존 원 PR 경계의 보존을 확인해야 한다.
`has_preceding_text` 제한을 새 측정 휴리스틱으로 단순 복제하는 것으로 공통 결과를 대체하지 않는다.

## 원 PR 검토 당시 검증 실행

| 항목 | 결과 |
| --- | --- |
| 원 head 전체 CI | [35088688931](https://github.com/edwardkim/rhwp/actions/runs/35088688931), 정확한 head `2543382e7`, Full 성공; Archive A/B/C/D, Lint, Native Skia 성공 |
| 동반 CI | [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/35088688936), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/35088688794), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/35088688970), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/35088689038) 성공 |
| 로컬 Native | `cargo build --locked --bin rhwp`, exit 0 |
| 원 PR focused | `cargo test --locked --test regression_suite_007 stored_nested_content_flow -- --nocapture`, 3 PASS, 0 FAIL, 194 filtered, exit 0 |
| fresh WASM | `scripts/wasm-pack-locked.sh --target web --out-dir /private/tmp/rhwp-pr7200-review-20260916/wasm-pkg`, release + wasm-opt, exit 0 |
| WASM 실제 실행 | `export-wasm-for-sweep.mjs`, Chrome/153.0.8010.47, 3개 대체 HWP와 3개 빈 문단 경계 |
| Visual Sweep | 같은 PDF에 base Native / PR Native / PR WASM 각각 3문서×1쪽, compare·standalone overlay·review 생성, 누락 0, 직접 판독 |
| 경계 검사 | 위 표: base PASS / PR Native·WASM FAIL |

환경: macOS, `DEVELOPER_DIR=/Library/Developer/CommandLineTools`, 전용
`CARGO_TARGET_DIR=/Users/tsjang/rhwp/target/pr7200-review-20260916`.
Rust 빌드는 순차 실행했다. 검토 중 엔진 코드를 변경하지 않았고 전체 회귀·Clippy·Native Skia는
성공한 원 head CI를 재사용했다. 통합 head 전체 CI를 새로 실행했다고 표현하지 않는다.
upstream WASM job은 skip이므로 이를 성공한 fresh WASM 빌드라고 세지 않았으며 위 로컬 빌드를 사용했다.

base 비교 CLI는 앞선 작업에서 보존한 `/private/tmp/rhwp-6611-20260916/rhwp-base`다.
그 빌드 source `78340390d`와 검토 base `222c8c4819`의 `src`, `crates`, `Cargo.toml`,
`Cargo.lock` 차이가 없음을 확인했다. Sweep manifest의 작업공간 HEAD를 이 base 바이너리의
빌드 SHA로 혼동하지 않는다.

| 산출물 | SHA256 |
| --- | --- |
| base CLI | `70b624cdeedf30857434bbdb4e13dbb82883e55a795b6a2b6c20ba7b4cf6a46a` |
| PR CLI | `b9a3959536be04e578ecc5c126636613c3b4836c26e6d45d25d01a9eb1ba3f10` |
| fresh rhwp.js | `707049ae519de26779c842eec40f94e016b09bda2688fa721a37d2ae670c2f26` |
| fresh rhwp_bg.wasm | `103d2a2b09bad9580c984018cb97f08def722a869a89c094acef4fafb46e081c` |

## 대체 HWP의 한컴 PDF와 Visual Sweep

기존 Git 입력 `samples/stored-nested-content-flow/width-{top,center,bottom}.hwp`를 재사용했다.
`rhwp info --json`의 저장 메타데이터는 `hancom-office-2010 / 8.0.0.466`이나 빈 문서에서 합성한
입력이므로 실제 해당 제품으로 저장했다는 증거로 삼지 않는다. 정책상 구버전 호환 engine `2020`을
선택했고 실제 응답은 **Hancom 12.0.0.4605**, `input_preprocess=none`, 폰트 등록 실패 0이었다.
비동기 start → status(succeeded 확인) → download를 사용했다. PDF는 모두 1쪽이다.

| 입력 | PDF | 변환 job |
| --- | --- | --- |
| width-top.hwp | [Top](../../../pdf/pr7200/pr7200-width-top-2020.pdf) | `18f5c1c6-fcec-4431-8dd6-50ecc410c759` |
| width-center.hwp | [Center](../../../pdf/pr7200/pr7200-width-center-2020.pdf) | `9c0ecf97-f437-4042-aa73-7547949cbe93` |
| width-bottom.hwp | [Bottom](../../../pdf/pr7200/pr7200-width-bottom-2020.pdf) | `2f7facd2-3111-4c9d-92bd-52988fbcade0` |

96 DPI, Chrome webfont raster, threshold 32. PR Native와 WASM의 각 페이지 PNG는 SHA256이 같다.
따라서 같은 페이지 이미지를 중복 보관하지 않고 fresh WASM 대표 증적과 base 증적을 보존했다.
원 HWP를 복사/이름 변경해 추가하지 않았다. 새 PDF 3개와 새 경계 입력 3개만 추가한다.

| 입력 1쪽 | base pixel / ink match (%) | PR Native·WASM pixel / ink match (%) | PR 증적 | base 증적 |
| --- | --- | --- | --- | --- |
| Top | 94.83490 / 6.62398 | 94.83490 / 6.62398 | [compare](../assets/pr7200_review/wasm_width_top_compare_001.png) · [overlay](../assets/pr7200_review/wasm_width_top_overlay_001.png) · [review](../assets/pr7200_review/wasm_width_top_review_001.png) | [compare](../assets/pr7200_review/base_width_top_compare_001.png) · [overlay](../assets/pr7200_review/base_width_top_overlay_001.png) · [review](../assets/pr7200_review/base_width_top_review_001.png) |
| Center | 94.54531 / 6.01274 | 94.65260 / 6.17747 | [compare](../assets/pr7200_review/wasm_width_center_compare_001.png) · [overlay](../assets/pr7200_review/wasm_width_center_overlay_001.png) · [review](../assets/pr7200_review/wasm_width_center_review_001.png) | [compare](../assets/pr7200_review/base_width_center_compare_001.png) · [overlay](../assets/pr7200_review/base_width_center_overlay_001.png) · [review](../assets/pr7200_review/base_width_center_review_001.png) |
| Bottom | 94.78802 / 6.54651 | 94.49635 / 5.47878 | [compare](../assets/pr7200_review/wasm_width_bottom_compare_001.png) · [overlay](../assets/pr7200_review/wasm_width_bottom_overlay_001.png) · [review](../assets/pr7200_review/wasm_width_bottom_review_001.png) | [compare](../assets/pr7200_review/base_width_bottom_compare_001.png) · [overlay](../assets/pr7200_review/base_width_bottom_overlay_001.png) · [review](../assets/pr7200_review/base_width_bottom_review_001.png) |

**직접 판정: PDF 시각 일치 미충족.** 한컴 PDF 자체에서도 `Wide text`가 한 줄에 중복 인쇄된 것처럼
겹친다. rhwp는 이를 여러 줄로 재조판하며 표의 가로 위치·테두리·문단 위치도 다르다.
이 차이는 base에도 존재하고, PR은 Center/Bottom의 콘텐츠 이동량을 바꾼다. Top은 변하지 않았다.
숫자가 조금 좋아지거나 자동 후보가 0이라는 이유로 한컴 재현 성공이라고 판정하지 않는다.
합성 HWP의 비정상 저장 줄을 한컴의 정상 문서 조판 기준으로 일반화할 수 없다.
이 자료의 한계는 앞서 실행으로 확인한 새 빈 문단 정렬 회귀와 분리한다.

## 원 PR 검토 당시 조판 원칙 및 범위 심사

| 주장 / 소비 경로 | 판정과 근거 |
| --- | --- |
| 초기화된 저장 위치에서 앞 문단 높이 계상 | 기존 3함수의 공개 경계 PASS. 그러나 빈 문단에서 실제 배치와 불일치: 위 P2 **미충족** |
| 배경·어울림 표 흐름 공유 | `nested_table_is_overlay`, `nested_table_flow_advance`를 측정과 실제 중첩 표 배치가 공유. 공개 overlay/square/flow 경계 PASS |
| 글 폭에 따른 앞 문단 재조판 | 기존 composed lines 사용, width 기하 계약 PASS. 한컴 시각 일치는 위와 같이 미충족 |
| HWP5 본문 표 소속 줄 선택 | `layout.rs`의 `projected_seg`가 HWP5 경로로 확대됨. 가시 선행 객체·표 높이 가드 유지. 음수/0 줄간격 및 순수 HWPX 통제 focused PASS |
| 일반 셀·부분 셀·continuation | `table_layout.rs`, `table_cell_content.rs`, `table_partial.rs`의 caller가 구성 문단을 전달. 구성 길이가 다른 continuation은 새 sequential 분기를 타지 않음. 실제 다쪽 분할·이어받기 경계는 **미검증**, 1쪽 합성으로 대체하지 않음 |
| baseline·문서별 예외 | 변경 없음. 특정 문서 ID 분기 없음. 다만 helper 공유만으로 모든 측정/배치 일치가 입증되지는 않음 |
| 실제 비공개 7쪽 / 저장 후 재열기 | 작성자 주장으로만 남김. reviewer는 사용자 지정 대체 HWP 3개로 검증 |

추가 탐색에서 같은 호스트의 표 2개는 이번 변경으로 정렬이 개선됐다. 뒤 텍스트가 있는 별도 합성
변형의 기존 넘침은 base에서도 재현돼 새 회귀로 세지 않았다. 최초 검토 당시 보류 사유는 위 P2 1건이었다. 사용자 요청 이후 시각 차이도 보류 범위에 포함했다.

## 재현 명령과 후속 범위

```bash
python3 mydocs/pr/assets/pr7200_review/check_empty_leading_paragraph.py \
  --rhwp-bin target/pr7200-review-20260916/debug/rhwp \
  --trees /private/tmp/pr7200-check

venv/bin/python scripts/visual_sweep.py \
  --file-target width-top samples/stored-nested-content-flow/width-top.hwp pdf/pr7200/pr7200-width-top-2020.pdf \
  --file-target width-center samples/stored-nested-content-flow/width-center.hwp pdf/pr7200/pr7200-width-center-2020.pdf \
  --file-target width-bottom samples/stored-nested-content-flow/width-bottom.hwp pdf/pr7200/pr7200-width-bottom-2020.pdf \
  --rhwp-bin target/pr7200-review-20260916/debug/rhwp \
  --wasm-pkg /private/tmp/rhwp-pr7200-review-20260916/wasm-pkg \
  --pages 1 --out /private/tmp/pr7200-sweep
```

Native 비교는 `--wasm-pkg`를 생략하고 base는 위 base CLI를 지정했다.
전체 실행 원시 자료는 `/private/tmp/rhwp-pr7200-review-20260916`에 있으며 raw log/JSON/TSV/SVG는
커밋하지 않는다. 현재 검토 branch와 전용 target은 후속 보정을 위해 유지한다.
이하 메인터너 보정 결과를 추가했다. 최종 head CI·원격 조치·병합은 수행하지 않았다.


## 메인터너 보정 — 위치·문단 테두리·객체 글자 테두리

사용자 요청으로 실행 중이던 전체 nextest와 소유 하위 프로세스를 중단했다(exit **143**).
정렬 불변식만 통과한 상태에서 큰 PDF 차이를 남기고 전체 테스트를 진행한 것은 잘못된 순서였다.
1회차에서는 이후 영향 페이지의 직접 비교를 우선했고 전체 회귀를 재실행하지 않았다.
2회차에서는 시각 보정 뒤 전체 검증을 완료했으며, 아래 최종 검증 기록을 따른다.

### 1회차의 위치·문단 테두리 보정

- 빈 줄의 점유 공간과 가시 텍스트 유무가 다르다. 측정/배치가 `SequentialNestedCellLayout`의
  원점·하단을 함께 소비하게 하여 빈 선행 줄 16px를 실제 중첩 표 배치에도 보존했다.
- 좁은 비-TAC 표를 무조건 가운데 놓던 경로가 글 앞으로/뒤로 표의 명시한 가로 앵커도 무시했다.
  Para/Column 기준 overlay의 Left/Center/Right와 offset을 반영했다. 빈 문단 입력의 x는
  **128 → 48px**, 한컴 PDF는 **48.32px**다. 기존 Square 경계 #5787은 통과했다.
- 본문 테두리 큐에 셀 문단이 섞이는 것을 막기 위해 셀의 문단 테두리를 통째로 생략하고 있었다.
  셀별 큐를 분리하고 중첩 셀/부모/본문의 범위를 복원한다. 테두리를 가진 빈 block 표 호스트도
  배치하고, 별도 PageItem인 TAC 표도 본문 문단 테두리 점유 범위에 넣었다.
- `border_connect`를 실제 해소 스타일과 병합 판정에 전달하여 연결이 꺼진 문단을 임의로 합치지 않는다.
  셀 마지막 문단은 정렬 측정에서 제외하는 후행 줄간격까지 테두리로 감싸되, 그 paint 범위를
  흐름 전진에 이중 가산하지 않는다.
- 정상 한컴 저장 줄의 `line_height=1000 / text_height=1056 / spacing=632`에서 56HU를 놓쳤다.
  일반 구성 줄에 실제 텍스트 점유 높이도 포함하여 측정과 paint가 함께 사용한다.

### 원본 합성과 정상 한컴 대조군을 구분

원본 `width-*.hwp`의 200자 문단은 저장 줄이 1개여서 한컴 PDF에서도 겹친다. 원본과 기존 PDF를
바꾸거나 숨기지 않았다. [한컴 재조판 대조군과 생성 절차](../../../tests/fixtures/pr7200_hancom_recomposed/README.md)는
저장 줄 캐시만 제거한 뒤 한컴으로 다시 저장한 **새로운 입력**이다. 텍스트·표·셀/문단 테두리를
보존했고 Top/Center/Bottom 각각의 최종 HWP와 PDF를 함께 보존한다. 단순 이름 변경 복사본이 아니다.
원본의 비정상 PDF를 모방하려고 renderer의 정상 줄 나눔을 일부러 겹치게 하지 않았다.
**대조군의 개선은 원본 HWP/PDF의 일치 증거가 아니다.**

| 정상 대조군 | 원 PR End x/y | 보정 End x/y | 한컴 PDF End x/y |
| --- | ---: | ---: | ---: |
| Top | 128 / 256.0 | 48 / 259.7 | 48.774 / 259.734 |
| Center | 128 / 373.1 | 48 / 376.8 | 48.774 / 376.566 |
| Bottom | 128 / 490.1 | 48 / 493.9 | 48.774 / 493.558 |

기존 구현의 5줄 y는 Top에서 127.2 / 149.0 / 170.7 / 192.5 / 214.2px였고,
보정 후 127.2 / 149.7 / 172.2 / 194.7 / 217.2px다. PDF 상단은
127.718 / 150.094 / 172.629 / 195.164 / 217.540px다.
정식 검사에 3개 정렬의 5줄과 End 위치·별도 문단 테두리를 포함했고 모두 통과했다.
같은 독립 PDF 위치 계약을 보정 전/후 WASM tree에 대조하면 기준 밖 글자 위치는 14곳 → 0곳이다.
[보정 전 Top](../assets/pr7200_review/before_valid_top_review_001.png),
[Center](../assets/pr7200_review/before_valid_center_review_001.png),
[Bottom](../assets/pr7200_review/before_valid_bottom_review_001.png)과 아래 최종 증적을 직접 대조했다.

### 2회차: 단독 TAC 객체 글자 테두리

별도 PageItem인 표는 텍스트 run의 글자 테두리 paint를 거치지 않아 추가 외곽선이 누락됐다.
표 호스트 control의 char shape를 찾아 물리적인 표 상자에 글자 테두리를 그린다. 배치·행 높이·
다음 문단 전진에는 이 장식 상자의 높이를 더하지 않는다. 투명 1×1 wrapper도 자기 테두리가
있으면 물리 상자를 보존한다. 텍스트/객체가 섞인 run 및 쪽 분할 조각은 이 신규 paint 범위가 아니다.

독립 한컴 PDF의 한쪽 700/800HU, 양쪽 1000/2000HU 여백에서 각각 최소 2.5mm(708HU)의
장식 여백을 확인했다. 고정 14pt 하단 덧셈이 아니라 실제 여백과 최소값을 적용한다.
[입력·PDF·생성 내역](../../../tests/fixtures/pr7200_hancom_recomposed/README.md)에 통제 방법과
job을 기록했다. 새 정식 테두리 검사는 1회차 코드에서 FAIL(기존 5개 PASS), 보정 후 6개 모두 PASS다.
글자 테두리만 끈 대조군에서는 추가 외곽선을 그리지 않는 것도 확인했다.

### 전체 회귀와 추가 PDF에서 확인한 보정

1차 전체 nextest는 9,945개 중 **9,941 PASS / 4 FAIL / 51 skipped**로 종료됐다.
이 결과를 성공으로 세지 않았으며 아래 원인을 수정했다.

- 줄 높이의 `max(line_height, text_height)` 보정 뒤 저장 TAC 줄 소속 검사는 옛 원값과 비교하여
  정상 한컴 저장 정보를 거부했다. `stored_line_box_height`를 구성·수용 판정이 공유하게 했다.
  기존 숫자 표의 같은 줄, 명시적 개행, 너비에 따른 줄바꿈, 뒤 텍스트 검사가 다시 통과한다.
- 원 PR overlay 합성 입력의 두 표는 모두 LEFT/offset=0이었다. 올바른 가로 앵커에서 `End`와
  `Second`가 겹쳤으므로 두 번째 표의 가로 위치를 첫 표 너비 12000HU만큼 명시했다.
  이 속성 하나 외에 XML과 다른 ZIP entry는 동일하다. 글자 겹침은 세 정렬 각각 1→0이다.
  baseline·허용치는 변경하지 않았다.
- 이 입력의 새 한컴 PDF는 두 표가 같은 y라는 이전 합성 가정도 반박했다. `Second`는 `End`보다
  **15.9968px** 아래다. 배경 객체는 전진하지 않지만 빈 호스트 줄은 1000+200HU = **16px**를
  점유한다. sequential 계획에서 배경 표의 호스트 줄을 보존하고, 동일 결과를 정렬 측정과 실제
  원점 배치가 소비한다. 새 16px assertion은 수정 전 FAIL, 수정 후 기존 5개와 함께 6 PASS다.

최종 집중 검사: `stored_nested_content_flow` 6개와 `stored_inline_table_suffix` 2개, **8/8 PASS**.
실제 저장 숫자 표의 PDF 3개와 변경한 overlay 입력/PDF 3개를 추가로 직접 비교했다.
기존 HWP/PDF는 원래 경로를 재사용했고 overlay HWPX는 새 사본 없이 기존 입력 자체를 수정했다.

### 2회차 Native/fresh WASM 직접 판독

96 DPI, threshold 32를 유지했다. 19개 단쪽 경계와 실문서 4쪽의 Native/WASM은 **23/23
바이트 동일**하다. 정상 저장본 7쪽에서 큰 위치·줄바꿈·외곽선 누락이 해소됐으며, 글꼴/가는 선의
래스터 차이는 남는다. 자동 ink 지표를 시각 통과율로 사용하지 않는다. 원본 width 3쪽은 여전히
한컴의 겹친 글자와 다르고, empty 3쪽은 합성 저장 메트릭과 한컴 재계산의 약 1.4px 글자/약 6px
후행 테두리 차이가 남는다. 작은 불변식이나 전체 회귀 통과로 이 차이까지 해소됐다고 보고하지 않는다.

| 입력 1쪽 | pixel / ink match (%) | 증적 |
| --- | --- | --- |
| empty-bottom | 97.16953 / 6.54342 | [compare](../assets/pr7200_review/maintainer_empty_bottom_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_empty_bottom_overlay_001.png) · [review](../assets/pr7200_review/maintainer_empty_bottom_review_001.png) |
| empty-center | 97.10391 / 6.43614 | [compare](../assets/pr7200_review/maintainer_empty_center_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_empty_center_overlay_001.png) · [review](../assets/pr7200_review/maintainer_empty_center_review_001.png) |
| empty-top | 97.01667 / 6.22135 | [compare](../assets/pr7200_review/maintainer_empty_top_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_empty_top_overlay_001.png) · [review](../assets/pr7200_review/maintainer_empty_top_review_001.png) |
| valid-bottom | 97.78025 / 9.25153 | [compare](../assets/pr7200_review/maintainer_valid_bottom_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_valid_bottom_overlay_001.png) · [review](../assets/pr7200_review/maintainer_valid_bottom_review_001.png) |
| valid-center | 97.84435 / 10.37476 | [compare](../assets/pr7200_review/maintainer_valid_center_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_valid_center_overlay_001.png) · [review](../assets/pr7200_review/maintainer_valid_center_review_001.png) |
| valid-top | 97.83054 / 8.27243 | [compare](../assets/pr7200_review/maintainer_valid_top_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_valid_top_overlay_001.png) · [review](../assets/pr7200_review/maintainer_valid_top_review_001.png) |
| width-bottom | 94.07292 / 4.91707 | [compare](../assets/pr7200_review/maintainer_width_bottom_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_width_bottom_overlay_001.png) · [review](../assets/pr7200_review/maintainer_width_bottom_review_001.png) |
| width-center | 94.15729 / 5.43707 | [compare](../assets/pr7200_review/maintainer_width_center_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_width_center_overlay_001.png) · [review](../assets/pr7200_review/maintainer_width_center_review_001.png) |
| width-top | 94.34141 / 5.82907 | [compare](../assets/pr7200_review/maintainer_width_top_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_width_top_overlay_001.png) · [review](../assets/pr7200_review/maintainer_width_top_review_001.png) |
| margin-both-1000 | 94.99896 / 12.27445 | [compare](../assets/pr7200_review/maintainer_margin_both_1000_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_margin_both_1000_overlay_001.png) · [review](../assets/pr7200_review/maintainer_margin_both_1000_review_001.png) |
| margin-both-2000 | 94.60625 / 7.83197 | [compare](../assets/pr7200_review/maintainer_margin_both_2000_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_margin_both_2000_overlay_001.png) · [review](../assets/pr7200_review/maintainer_margin_both_2000_review_001.png) |
| margin-min-700 | 95.00703 / 9.90555 | [compare](../assets/pr7200_review/maintainer_margin_min_700_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_margin_min_700_overlay_001.png) · [review](../assets/pr7200_review/maintainer_margin_min_700_review_001.png) |
| margin-min-800 | 95.10469 / 10.08323 | [compare](../assets/pr7200_review/maintainer_margin_min_800_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_margin_min_800_overlay_001.png) · [review](../assets/pr7200_review/maintainer_margin_min_800_review_001.png) |
| digits-break | 96.58507 / 0.83186 | [compare](../assets/pr7200_review/maintainer_digits_break_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_digits_break_overlay_001.png) · [review](../assets/pr7200_review/maintainer_digits_break_review_001.png) |
| digits-table-wrap | 97.39019 / 1.06943 | [compare](../assets/pr7200_review/maintainer_digits_table_wrap_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_digits_table_wrap_overlay_001.png) · [review](../assets/pr7200_review/maintainer_digits_table_wrap_review_001.png) |
| digits-wrap | 97.32899 / 2.36395 | [compare](../assets/pr7200_review/maintainer_digits_wrap_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_digits_wrap_overlay_001.png) · [review](../assets/pr7200_review/maintainer_digits_wrap_review_001.png) |
| overlay-bottom | 96.77422 / 6.63300 | [compare](../assets/pr7200_review/maintainer_overlay_bottom_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_overlay_bottom_overlay_001.png) · [review](../assets/pr7200_review/maintainer_overlay_bottom_review_001.png) |
| overlay-center | 96.78906 / 6.49882 | [compare](../assets/pr7200_review/maintainer_overlay_center_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_overlay_center_overlay_001.png) · [review](../assets/pr7200_review/maintainer_overlay_center_review_001.png) |
| overlay-top | 96.72448 / 6.26723 | [compare](../assets/pr7200_review/maintainer_overlay_top_compare_001.png) · [overlay](../assets/pr7200_review/maintainer_overlay_top_overlay_001.png) · [review](../assets/pr7200_review/maintainer_overlay_top_review_001.png) |

실문서 #2470 1·2쪽과 #6787 1·2쪽도 재캡처·직접 판독했다. 1회차/base와 각 페이지가
바이트 동일하다. #2470 2쪽 사진/행 높이 차이와 #6787 기존 글꼴·자간 차이를 해결한 것은 아니다.

- #2470: [1쪽 overlay](../assets/pr7200_review/maintainer_issue2470_overlay_001.png), [2쪽 overlay](../assets/pr7200_review/maintainer_issue2470_overlay_002.png)
- #6787: [1쪽 overlay](../assets/pr7200_review/maintainer_issue6787_overlay_001.png), [2쪽 overlay](../assets/pr7200_review/maintainer_issue6787_overlay_002.png)

1회차 전체 결과와 이전 지표는 commit `4ce8d253cc0ea1fa7429957ce31184ee3d8e9bc0`의 이 문서에
보존했다. 현재 `maintainer_*` 경계 PNG는 2회차 출력이며, 원 PR/base/보정 전 PNG는 유지했다.

### 최종 검증 기록

모든 아래 검사는 위 최종 production source와 최종 입력에서 성공했다. 과거 사용자 중단
nextest(exit 143)와 1차 전체의 4 FAIL은 이 결과에 합산하지 않는다.

| 검사 | 명령 / 범위 | 결과 |
| --- | --- | --- |
| 집중 회귀 | case wrapper `stored_nested_content_flow`, `stored_inline_table_suffix` | 6 + 2 PASS |
| 전체 회귀 | `cargo nextest run --locked --cargo-profile release-test --tests --test-threads 8 --no-fail-fast` | **9945 PASS, 0 FAIL, 51 skipped**, exit 0; 실행 317.275초 |
| 포맷 | `cargo fmt --all -- --check` | PASS |
| Native Clippy | `cargo clippy --locked -- -D warnings` | PASS |
| WASM Clippy | `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown -- -D warnings` | PASS |
| workspace | `cargo build --locked --workspace` | PASS |
| 전체 target Clippy | `cargo clippy --locked --workspace --all-targets -- -D warnings` | PASS |
| suite 정책 | `rust-test-suite-manifest.mjs --check --base-ref 222c8c4819de414898bf5b15608b0d0d394a3748` | PASS, 48/48 integration targets |
| Native Skia lib | `cargo test --locked --profile release-test --features native-skia --lib` | rhwp 3930 + 공용 crate 182 PASS, 13 ignored |
| Native Skia 그림 | case wrapper `issue_2225_missing_picture_placeholder`, release-test/native-skia | 2 PASS |
| Native Skia PDF | case wrapper `render_p37_direct_pdf_export`, release-test/native-skia | 4 PASS |
| 새 문서 보안 | 전체 회귀에는 sample 28개를 환경변수로 전달. 추가로 최종 CLI의 `inspect hidden-text`, `inspect injection --include-fields`, `inspect unicode`를 sample 28 + fixture 11개에서 실행 | **39 × 3 = 117 clean**, 읽기/파싱 실패 0 |
| 시각 검증 | 같은 입력·PDF의 Native/fresh WASM compare·standalone overlay·review | 23쪽 직접 판독, backend PNG **23/23 동일** |
| 문서 | 변경 guide 메타데이터, 상대 링크, `git diff --check` | PASS |

Rust/Cargo 작업은 전용 target에서 순차 실행했다. baseline·golden·래칫 허용치와 원본 width
입력/PDF는 변경하지 않았다. 원격 head CI는 보정 내용을 아직 검사한 것이 아니며 push·merge는 미수행이다.

### 2회차 당시 해결 범위와 남은 시각 판정 (3회차로 갱신)

- 빈 선행 줄의 측정/배치 불일치, 80px 가로 앵커 이동, 누락 문단·객체 글자 테두리, 정상 저장 줄의
  높이·소속 불일치 및 배경 표 호스트 줄 생략은 보정하고 실행 증거를 남겼다.
- 원본 `width-*.hwp`의 저장 줄은 200자에 1개다. 한컴 PDF는 글자를 겹치고 원 PR의 정식 계약은
  너비에 맞춘 여러 줄 재조판을 요구한다. 이 입력에서 두 결과를 동시에 만족한다고 주장할 수 없다.
  원본과 실패 PDF를 보존하고 정상 저장 대조군의 결과를 분리했다. 단순히 합성/기존 차이라고
  분류하여 보류를 해제하지 않았으며, 원본 PDF 일치 주장은 여전히 미충족이다.
- 정상 저장 대조군 7쪽의 큰 위치·누락선은 해소됐다. 폰트/가는 선 래스터, empty 합성의 약 1.4px
  글자 위치 및 약 6px 후행 테두리 차이는 남는다. 실문서 4쪽의 기존 차이는 이번 변경 전후 동일하다.
- 혼합 텍스트/객체 run의 연결 글자 테두리, 쪽 분할 객체 테두리 및 비공개 7쪽 원본은 검증 범위에
  포함하지 않는다. 작은 assertion·전체 회귀 통과를 그 범위의 PDF 외형 일치로 확대하지 않는다.

### 최종 빌드 계보

기준 source는 `fbc14758f` 위 작업 트리의 메인터너 보정이다. 원 PR head의 CI를 보정 head의
검증 결과로 재사용하지 않는다. Native CLI는 후속 workspace build와 구분해 캡처 시점의
바이너리를 임시 경로에 보존했다. 원문·PDF 해시는 위 fixture README에 기록했다.

| 캡처 산출물 | SHA256 |
| --- | --- |
| Native CLI | `5c6e138fa32b32983d5fe0550733fe25dbb1640f90cab645030cf18e6336c1f3` |
| fresh rhwp.js | `707049ae519de26779c842eec40f94e016b09bda2688fa721a37d2ae670c2f26` |
| fresh rhwp_bg.wasm | `8db65b047f0c878da2800bdf3868843e5590575cda6966a050265cbf8aad8b72` |

| 보정 production source | SHA256 |
| --- | --- |
| `src/renderer/composer.rs` | `77335344cb0cbdf79e665aabce6fed2c1fa98a0b8c7946fc8cffa628b8a46757` |
| `src/renderer/layout.rs` | `cb1911fd337c0e6cf1f01cda3355fb56dd09dab954b7577512ba23453a918d1d` |
| `src/renderer/layout/paragraph_layout.rs` | `47a8232c376afc88989020571f9315bb3ab903926ec0542047da42f070a6eb3b` |
| `src/renderer/layout/table_layout.rs` | `07fce5df5883b3ee8a7e3bc8dd8fb4543b2b4e00b6127d71eeb8c976d6d0b877` |
| `src/renderer/layout/table_partial.rs` | `0bb5e322b66fc7b19476ad0632de31a239739d266804c5ff9b82564585fb2e3c` |

최종 실행 경로는 `/private/tmp/rhwp-pr7200-review-20260916/stage2-final-*`이며, raw log·JSON은
커밋하지 않는다. `visual_sweep.py --rhwp-bin target/pr7200-review-20260916/debug/rhwp`에
위 표의 동일 입력/PDF, `--pages 1`(실문서는 `--pages 1,2`)을 지정했다. WASM 실행에는
`--wasm-pkg /private/tmp/rhwp-pr7200-review-20260916/stage2-final-wasm-pkg`를 추가했다.
수정 전 증거와 원본 실패 자료를 남기고, 최종 대표 PNG만 기존 경로에 갱신했다.
