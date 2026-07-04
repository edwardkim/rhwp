# PR #1530 사전 처리 판단 보고서 — HWPX OLE 개체 데이터 참조 보존

- PR: https://github.com/edwardkim/rhwp/pull/1530
- 제목: `Task #1529: HWPX 직렬화 — OLE 개체(<hp:ole>) 데이터 참조 보존`
- 작성자: `planet6897` (Jaeuk Ryu)
- 관련 이슈: #1529
- 검토일: 2026-06-26
- 검증 head: `d52a165c4e3d0c899b88de47ac38efecb7119a99`
- 검증 worktree: `/private/tmp/rhwp-pr1530-head`
- 처리 경로: collaborator-mediated 외부 PR 처리 경로
- 문서 경로: `mydocs/pr/archives/pr_1530_review.md`, `mydocs/pr/archives/pr_1530_report.md`

## 1. 사전 판단

**수용 가능. Blocking finding 없음.**

PR #1530은 HWPX roundtrip에서 `<hp:ole>`의 `binaryItemIDRef`가 누락되어
`samples/hwpx/143E433F503322BD33.hwpx`의 차트 OLE가 `RawSvg`에서 placeholder로 강등되던
문제를 좁은 범위에서 정정한다.

검토 결과, 변경은 `ShapeObject::Ole` serializer dispatch와 `write_ole()` 추가, visual baseline 승격에
한정된다. 로컬 테스트, 대상 샘플 `render-diff`, roundtrip 패키지 직접 검사, rhwp-studio 로드 확인 모두
통과했다.

작업지시자 시각 검증에서도 PR head roundtrip 파일은 차트가 정상 표시되고, PR 반영 전
`upstream/devel` roundtrip 파일은 OLE 데이터 참조 누락으로 placeholder 강등이 재현됨을 확인했다.

## 2. PR 상태

| 항목 | 값 |
|---|---|
| state | open |
| draft | false |
| mergeable | `MERGEABLE` |
| head SHA | `d52a165c4e3d0c899b88de47ac38efecb7119a99` |
| 변경량 | 3 files, +85 / -10 |
| GitHub review threads | 없음 |
| `closingIssuesReferences` | 비어 있음. PR body는 `closes #1529`를 언급하므로 merge 후 #1529 상태 확인 필요 |

GitHub Actions:

| 체크 | 결과 |
|---|---|
| Build & Test | success |
| Analyze (rust) | success |
| Analyze (javascript-typescript) | success |
| Analyze (python) | success |
| CodeQL | success |
| WASM Build | skipped |

## 3. 변경 검토

| 파일 | 변경 | 판단 |
|---|---|---|
| `src/serializer/hwpx/section.rs` | `ShapeObject::Ole`를 공용 shape XML 경로 대신 `write_ole()`로 dispatch | 타당 |
| `src/serializer/hwpx/shape.rs` | `<hp:ole>` 전용 writer 추가. 공통 attr, `binaryItemIDRef`, shape attr block, extent, lineShape, sz/pos/outMargin/caption 방출 | 타당 |
| `tests/visual_roundtrip_baseline.rs` | `143E433F503322BD33.hwpx`를 `VISUAL_XFAIL`에서 제거 | head 검증에서 PASS하므로 타당 |

핵심 확인:

- 원본 143E 샘플은 `<hp:ole binaryItemIDRef="ole3">`와 `BinData/ole3.ole`을 가진다.
- PR head roundtrip 결과는 manifest id를 재생성해 `binaryItemIDRef="image3"`로 쓰지만,
  `content.hpf id="image3"`와 `BinData/image3.OLE`가 함께 생성되어 3-way 참조가 맞다.
- SVG 원본/roundtrip 모두 `hwp-ole-chart hwp-ole-chart-rust-svg` 그룹을 포함했고, `OLE 개체`
  placeholder 문자열은 나오지 않았다.

비차단 잔여:

- 기존 HWPX parser/IR 범위상 `<hp:ole>` 원본 XML의 `id`, `numberingType="PICTURE"`,
  manifest id 문자열 `ole3` 자체는 그대로 보존되지 않고 재생성된다. 이번 PR의 목표인 OLE 데이터 참조
  보존과 시각 회귀 해소에는 영향이 없으며, 별도 무손실 XML 보존 범위로 분리하는 것이 맞다.

## 4. 로컬 검증

| 명령 | 결과 |
|---|---|
| `git diff --check upstream/devel...HEAD` | 통과 |
| `cargo fmt --check` | 통과 |
| `cargo test --lib serializer::hwpx::shape` | 통과, 14 passed |
| `cargo test --test hwpx_roundtrip_baseline` | 통과, 4 passed |
| `cargo test --test visual_roundtrip_baseline` | 통과, 3 passed |
| `cargo test --test issue_1156_chart_column_flow` | 통과, 2 passed |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `rhwp render-diff samples/hwpx/143E433F503322BD33.hwpx --via hwpx --max-disp 0.5` | PASS, page 1→1, max_disp 0.00px, struct mismatch 0 |
| `rhwp hwpx-roundtrip samples/hwpx/143E433F503322BD33.hwpx -o output/poc/pr1530-roundtrip` | PASS, diff=0, r2=0 |

PR 반영 전 `upstream/devel` 비교:

| 항목 | 결과 |
|---|---|
| roundtrip 생성 | `hwpx-roundtrip` 자체는 PASS, diff=0, r2=0 |
| `<hp:ole>` XML | `binaryItemIDRef` 없음 |
| `render-diff --via hwpx` | `STRUCT_MISMATCH`, `Placeholder: 0→1`, `RawSvg: 1→0` |

따라서 기존 `hwpx-roundtrip` PASS만으로는 OLE 시각 회귀를 검출할 수 없고,
이번 PR의 `visual_roundtrip_baseline` 승격과 `render-diff` 구조 검증이 필요한 회귀 게이트다.

Roundtrip 직접 검사:

```text
<hp:ole ... binaryItemIDRef="image3" ...>
<hc:extent x="7200" y="7200"/>
```

```text
content.hpf: <opf:item id="image3" href="BinData/image3.OLE" media-type="application/octet-stream" isEmbeded="1"/>
ZIP:         BinData/image3.OLE
```

## 5. 시각 검증 산출물

생성 위치:

- `/private/tmp/rhwp-pr1530-head/output/poc/pr1530-roundtrip/143E433F503322BD33.rt.hwpx`
- `/private/tmp/rhwp-pr1530-head/output/poc/pr1530-ole-visual/original/143E433F503322BD33.svg`
- `/private/tmp/rhwp-pr1530-head/output/poc/pr1530-ole-visual/roundtrip/143E433F503322BD33.rt.svg`
- `/private/tmp/rhwp-pr1530-head/output/poc/pr1530-ole-visual/debug-original/143E433F503322BD33.svg`
- `/private/tmp/rhwp-pr1530-head/output/poc/pr1530-ole-visual/debug-roundtrip/143E433F503322BD33.rt.svg`

확인 결과:

- 원본/roundtrip SVG 모두 1페이지 우측 상단 차트가 Rust SVG OLE chart로 렌더된다.
- `render-diff`는 구조 불일치 0, 최대 변위 0.00px다.
- roundtrip SVG 크기는 원본 대비 36 bytes 차이가 있으나, OLE chart 그룹과 좌표는 동일하다.

## 6. rhwp-studio 직접 검증 준비

dev server:

- URL: http://127.0.0.1:7700/
- worktree: `/private/tmp/rhwp-pr1530-head`
- 실행 명령: `cd /private/tmp/rhwp-pr1530-head/rhwp-studio && npm run dev -- --host 127.0.0.1 --port 7700`

파일 선택용 경로:

- 원본: `/private/tmp/rhwp-pr1530-head/samples/hwpx/143E433F503322BD33.hwpx`
- roundtrip: `/private/tmp/rhwp-pr1530-head/output/poc/pr1530-roundtrip/143E433F503322BD33.rt.hwpx`

URL 자동 로드:

- 원본: http://127.0.0.1:7700/?url=%2Fsamples%2Fpr1530%2Foriginal-143E433F503322BD33.hwpx&filename=original-143E433F503322BD33.hwpx
- roundtrip: http://127.0.0.1:7700/?url=%2Fsamples%2Fpr1530%2Froundtrip-143E433F503322BD33.rt.hwpx&filename=roundtrip-143E433F503322BD33.rt.hwpx

브라우저 확인:

- rhwp-studio 첫 화면 로드 성공, 콘솔 error/warn 없음.
- 원본 URL 자동 로드 성공, 1페이지 canvas 렌더, 차트 표시.
- roundtrip URL 자동 로드 성공, 1페이지 canvas 렌더, 차트 표시.
- 두 파일 모두 `파일 로드 실패` 상태가 아니다.
- 작업지시자가 PR head roundtrip 파일과 PR 반영 전 roundtrip 파일을 직접 열어 차이를 확인했다.

주의:

- Docker daemon이 실행 중이 아니어서 표준 `docker-compose --env-file .env.docker run --rm wasm` 빌드는 실패했다.
- rhwp-studio 직접 검증을 위해 임시 worktree에서만 로컬 `wasm-pack build --target web` fallback을 사용했고,
  생성된 `pkg/` 산출물을 `rhwp-studio/public/`에 복사했다. PR 반영 대상이 아니다.

## 7. 권장 처리

권고: **Approve 가능**.

merge 전 확인:

1. review 문서 커밋이 PR head diff에 포함되는지 확인한다.
2. 문서 커밋 push 후 GitHub Actions 최신 상태 또는 review 문서 전용 fast-pass 결과를 확인한다.
3. merge 전 최신 `mergeable` 상태를 다시 확인한다.
4. merge 후 #1529가 자동 close되지 않으면 수동 close comment를 남긴다.

GitHub review 코멘트 초안:

```text
PR #1530 head d52a165c 기준으로 검토했습니다.

이번 변경은 HWPX serializer의 OLE 경로에 좁게 한정되어 있고, 확인한 원인과 직접 맞습니다. 기존 공용 shape writer 경로로 빠지면서 <hp:ole>의 데이터 참조가 빠지던 문제를, OLE 전용 writer에서 resolved binaryItemIDRef와 shape_attr/extent/line/size/position 정보를 함께 방출하도록 정정합니다.

로컬 검증은 모두 통과했습니다.

- cargo fmt --check
- git diff --check
- cargo test --lib serializer::hwpx::shape
- cargo test --test hwpx_roundtrip_baseline
- cargo test --test visual_roundtrip_baseline
- cargo test --test issue_1156_chart_column_flow
- cargo clippy --all-targets -- -D warnings

대상 샘플 samples/hwpx/143E433F503322BD33.hwpx에 대해 render-diff는 page count 1->1, max displacement 0.00px, structure mismatch 0으로 통과했습니다. PR head roundtrip 패키지도 section0.xml의 binaryItemIDRef="image3", content.hpf의 id="image3", ZIP entry BinData/image3.OLE가 3-way로 정합합니다. rhwp-studio에서도 원본과 PR head roundtrip 파일 모두 OLE 차트가 placeholder가 아니라 차트로 표시됨을 확인했습니다.

추가로 PR 반영 전 upstream/devel roundtrip 결과도 같은 샘플로 확인했습니다. 해당 파일은 <hp:ole>에서 binaryItemIDRef가 빠지고, render-diff가 RawSvg -> Placeholder 구조 불일치를 보고합니다. 따라서 이 PR은 의도한 회귀를 실제로 해소합니다.

작업지시자 시각 검증 캡처 첨부 위치:

- PR 반영 전 roundtrip 캡처: [여기에 placeholder로 강등된 화면 캡처 첨부 예정]
- PR 반영 후 roundtrip 캡처: [여기에 OLE 차트가 정상 표시된 화면 캡처 첨부 예정]

Blocking finding 없습니다. Approve합니다.
```
