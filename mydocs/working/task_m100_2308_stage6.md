# M100 #2308 Stage 6 — 통합·시각 검증·문서화

## 기준

- 브랜치: `issue-2308-render-normalized-derived-state`
- 비교 기준: `upstream/devel@cbddc1cd87084b60685da9a2b4369a4511d86173`
- 코드 기준: Stage 5 `6438a4cfb`
- 완료일: 2026-07-23
- 상태: 전체 로컬 회귀 검증·OVR 완료, 한컴 oracle gap #3128 분리, draft PR #3130 게시

## 최종 코드 동일성

Stage 2~5를 재구성한 뒤 `src/`와 `tests/`를 기존 구현 완료본
`1f2054faafdf0d82f6fa7634f01f4d2537f42036`과 비교했다. 파일 차이는 0건이다. 이력 재구성은
검증된 최종 구현을 바꾸지 않고 Stage 경계와 문서 추적성을 복원했다.

## OVR

```text
python3 tools/object_visual_regression.py \
  samples/76076_regulatory_analysis.hwp \
  samples/issue2004_cell_image_stack.hwp \
  -o /private/tmp/issue2308-hw-ovr \
  --diff-against upstream/devel
```

| 샘플 | 페이지 | 개체 | 회귀 |
| --- | ---: | ---: | ---: |
| `76076_regulatory_analysis.hwp` | 82→82 | 9→9 | 0 |
| `issue2004_cell_image_stack.hwp` | 8→8 | 0→0 | 0 |

- 비교: 현재 `6438a4cfb` vs `upstream/devel@cbddc1cd8`
- 허용 오차: ±2px
- 합계: 회귀 0건
- 결과: `/private/tmp/issue2308-hw-ovr/ovr_diff.md`

OVR은 한컴 없이 실행하는 geometry 보조 근거다. 한컴 before/after/OVL 사람 판정은 전체 검증
결과와 함께 아래에 기록한다.

## 전체 로컬 검증

### Rust

| 명령 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo build --release` | PASS |
| `cargo test --release --lib` | 2537 passed, 7 ignored |
| `cargo test --profile release-test --tests` | 모든 test binary PASS |
| `cargo test --profile release-test --features native-skia skia --lib` | 56 passed |
| `cargo test --profile release-test --features native-skia --test issue_2225_missing_picture_placeholder` | 2 passed |
| `cargo test --profile release-test --features native-skia --test render_p37_direct_pdf_export` | 4 passed |
| `cargo clippy -- -D warnings` | PASS |
| `cargo clippy --all-targets -- -D warnings -A clippy::identity-op` | PASS |
| `cargo test --doc` | 1 ignored, 실패 0 |

`cargo clippy --all-targets -- -D warnings`만
`src/parser/hwp3/johab.rs:113`의 기존 `0x8000 | ... | 0`에 대해 Rust 1.93.1
`clippy::identity-op`로 실패했다. 이 파일은 `upstream/devel`과 동일하고 실제 CI 명령인
`cargo clippy -- -D warnings`는 통과했다. 해당 lint 하나를 허용한 all-targets 재실행도 통과해
#2308 변경 회귀가 아닌 upstream 기준선으로 분리했다.

### WASM·Studio

| 명령 | 결과 |
| --- | --- |
| `wasm-pack build --target web --out-dir pkg` | PASS |
| `npm test` | 505 passed |
| `npm run build` | PASS |
| `npm run e2e:renderer-contract` | PASS |
| `npm run e2e:issue-2214` | HWP/HWPX 각 3회와 IME/iOS raw smoke 모두 GREEN |
| `npm run e2e:render-diff` | 3 fixture PASS |
| `npm run e2e -- --mode=headless` | 입력·줄바꿈·분할·페이지 넘김·병합 PASS |

- `pkg/rhwp_bg.wasm` SHA-256:
  `8e1ef352cbe6536dedf51553ea709d25943a560d3217947bc1e651f840a1eae4`
- canvas render diff는 `basic/KTX.hwp` 1쪽에서 `116/889746` 픽셀
  (`0.01304%`) 차이로 허용 범위 안이었고, 나머지 두 fixture는 0픽셀이었다.
- 대표 `npm run e2e:baseline:headless`는 전체 캡처 뒤 `exam-math`의
  `equation:invalidLayout` 때문에 종료 코드 1이었다. 독립 `upstream/devel@cbddc1cd8`
  WASM으로 같은 표본을 재실행해 네 backend/profile 조합이 모두 같은 사유로 실패함을 확인했다.
  두 실행의 `exam-math` PNG 6개 SHA-256도 각각 일치해 #2308 회귀에서 분리했다.

## 한컴 기준 before/after/OVL

저장소의 한컴 기준 PDF를 사용해 `scripts/task1274_visual_sweep.py`를 실행하고 review contact
sheet를 직접 판정했다. 이 판정은 upstream 대비 회귀 여부와 한컴 oracle 정확성을 분리한다.

| 표본 | 범위 | 페이지 | 자동 후보 | upstream 회귀 | 한컴 oracle |
| --- | --- | ---: | ---: | --- | --- |
| `76076_regulatory_analysis.hwp` | #2195 핵심 33~34쪽 | 82=82 | 0/2 | PASS | KNOWN GAP — #3128 |
| `issue2004_cell_image_stack.hwp` | 전체 1~8쪽 | 8=8 | 0/8 | PASS | 글꼴 민감 차이, exact fidelity 미판정 |

- 한컴 OVL 평균 `visual_accuracy_proxy_percent`는 76076 표본 `10.50925`,
  #2004 표본 `19.88209`였다. 로컬 한컴/HY 글꼴 부재에 따른 glyph raster 차이가 커서 이 값은
  구조 정확성 판정이 아닌 보조값으로만 사용했다. 자동 후보 0건도 한컴 정확성 PASS를 뜻하지
  않고, 현재 휴리스틱이 frame overflow 등 정해진 후보를 잡지 않았다는 뜻이다.
- #2004 before/after 8쪽 PNG는 모두 바이트 동일했다.
- 76076의 34쪽은 before/after가 바이트 동일하지만 한컴과는 분명한 차이가 있다. continuation
  셀 텍스트가 fragment 상단이 아니라 크게 아래에 배치되고 wrapping·우측 clip이 다르며, 뒤따르는
  `직접편익` 표도 아래로 밀린다. 기존 oracle fidelity gap으로
  [#3128](https://github.com/edwardkim/rhwp/issues/3128)에 분리했다.
- 76076의 33쪽은 `1507/891662` 픽셀(`0.16901023%`)이 달랐고
  최대 채널 차이는 27, 차이 bbox는 `(269, 777, 711, 792)`였다. 같은 텍스트 한 줄의
  부동소수점 직렬화·서브픽셀 안티앨리어싱 차이이며 OVR geometry와 육안 배치는 동일했다.
- current review:
  `/private/tmp/issue2308-visual/76076/issue2308-76076/review_contact_sheet.png`
  및
  `/private/tmp/issue2308-visual/issue2004/issue2308-issue2004/review_contact_sheet.png`

## 문서화

- 수행·구현 계획의 Stage 1~6 경계를 유지했다.
- Stage별 완료보고서를 `task_m100_2308_stage1.md`부터 `stage6.md`까지 분리했다.
- `mydocs/tech/rendering_engine_design.md`에 source IR, revision cache, sparse overlay,
  invalidation/fallback 계약을 반영했다.
- 최종 보고서에 Stage별 커밋과 focused/OVR 결과를 기록했다.

## 원격 현황

- 브랜치 push: 완료
- draft PR: [#3130](https://github.com/edwardkim/rhwp/pull/3130)
- 한컴 oracle 후속: [#3128](https://github.com/edwardkim/rhwp/issues/3128)
- 남은 작업: CI·review 대응, #2308 구현 결과 코멘트
