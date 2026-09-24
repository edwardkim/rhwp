# PR #7316 리뷰 — 저장 되감김의 쪽 경계 위치 일치 판정

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR | [#7316](https://github.com/edwardkim/rhwp/pull/7316) |
| 작성자 | `planet6897` 외부 contributor |
| 관련 이슈 | [#6761](https://github.com/edwardkim/rhwp/issues/6761), [#6782](https://github.com/edwardkim/rhwp/issues/6782) |
| 원 contributor head | `edd54b6aae5deec172981858f5ed3fc281929b30` |
| 통합 적용 commit | `878ba0d8035b6466c53ba3ef6a0598904d7b7060` (`git cherry-pick -x`) |
| 메인터너 provenance 보정 | `4c663976b` |
| Visual Sweep 증적 | `4e054c2ae` |
| 통합 base | `be33c935939c84d4fca9692ddd64a5d951876ded` (`upstream/devel`) |

라우팅: `collaborator_external_pr` + `intake_and_review` + `local_validation` +
`visual_fixture_evidence`. 원 contributor commit은 rewrite하지 않았으며 최신 `upstream/devel` 위의
통합 검토 브랜치에 `-x`로 적용했다. `4c663976b`은 제품 코드를 바꾸지 않고 #6761 test의 기준 PDF
provenance만 실제 추적 파일로 바로잡은 메인터너 보정이다.

## 변경과 메인터너 보정

`typeset.rs`는 저장 위치로 되감긴 문단의 쪽 경계를 채움률만으로 거부하지 않고, 실제 저장 위치가
현재 쪽 시작 위치와 일치하는 경우를 경계로 인정한다. #6761 회귀는 이 재현을 고정하고, #6782의
cell-float 사례는 그림 위치·표 경계 계약을 함께 검사한다.

원 contributor test 주석은 기준을 MCP engine 2024와 `13.0.0.3901` 저장본으로 적었으나 실제 파일과
맞지 않았다. 메인터너 보정은 다음의 실제 provenance로 교체했다.

| 항목 | 실제 값 |
| --- | --- |
| 입력 HWP | `samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp` |
| 입력 SHA-256 | `398d03a5d5e4d6e857086be532d6d9ed0cec9c8ad06f95c17bbb7f83056ae860` |
| HWP 저장 정보 | `hancom-office-2010` / `8.5.8.1677` |
| 기준 변환 프로필 | `engine 2020` |
| 기준 PDF | `pdf/1480000-201900042-chemical-product-labeling-study-2020.pdf` |
| 기준 PDF SHA-256 | `f8e5c0408e221080ede9a9a67b153d02d792d22961c738e46749641f32a32e79` |
| PDF Creator·쪽수 | Hwp 2022 0.0.0.0, 103쪽 |

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | 통과 |
| #6761 경계 회귀 2개 | 통과 |
| #6782 갱신 회귀 2개 | 통과 |
| Native·WASM Clippy | 통과 |
| `rust-test-suite-manifest --check --base-ref upstream/devel` | 통과 |
| `git diff --check` | 통과 |
| Native Visual Sweep, 13·14쪽 | complete, 구조 flag 0개 |
| fresh WASM Visual Sweep, 13·14쪽 | complete, 구조 flag 0개 |

## Visual Sweep 증적

실제 HWP와 한컴 2020 기준 PDF의 13·14쪽을 96 dpi로 대조했다. Native와 fresh WASM 양쪽에서
14쪽 시작 경계가 복원되고, 기존 그림 좌표 계약은 유지됐다. 다음 8장은 review와 standalone overlay를
함께 보관한 최종 증적이다.

| backend | 13쪽 review / overlay | 14쪽 review / overlay |
| --- | --- | --- |
| Native | [review](../assets/pr7316_review/native_review_013.png) / [overlay](../assets/pr7316_review/native_overlay_013.png) | [review](../assets/pr7316_review/native_review_014.png) / [overlay](../assets/pr7316_review/native_overlay_014.png) |
| fresh WASM | [review](../assets/pr7316_review/wasm_review_013.png) / [overlay](../assets/pr7316_review/wasm_overlay_013.png) | [review](../assets/pr7316_review/wasm_review_014.png) / [overlay](../assets/pr7316_review/wasm_overlay_014.png) |

Native pixel similarity는 13쪽 91.41749%, 14쪽 93.95511%였고, WASM도 같은 구조 결과를 보였다.
표·그림·글꼴의 기존 raster 잔차가 남으므로 이 증적을 전체 PDF 시각 일치 주장으로 확대하지 않는다.
수용 근거는 저장 되감김의 14쪽 경계 복원과 그림 좌표·구조 계약의 유지다. 세부 파일 provenance는
[증적 기록](../assets/pr7316_review/provenance.md)에 고정했다.

## 최종 판정

**메인터너 보정 후 수용 가능.** 원 contributor head에는 기준 PDF provenance 오류가 있어 그대로
수용할 수 없지만, 제품 코드와 분리된 `4c663976b` 보정으로 실제 입력·변환 프로필·기준 PDF를
정확히 기록했고, 그 통합 후보에서 focused 회귀와 Native/fresh WASM Visual Sweep을 완료했다.

이 판정은 원 contributor PR을 직접 merge하거나 GitHub approve·push·merge를 실행하는 권한 행사가
아니다. 통합 PR 생성 뒤 이 review·오늘할일 trailing commit을 포함한 최신 head의 required CI,
`MERGEABLE`/`CLEAN`, 작업지시자의 merge 지시를 다시 확인한다. 병합 후 원 PR에는 실제 통합 PR,
merge SHA, CI, 메인터너 provenance 보정 범위와 위 8개 증적 이미지가 보이도록 한국어 comment를
게시한다.
