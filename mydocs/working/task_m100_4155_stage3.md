# Stage 3 — task_m100_4155 술어 통일과 시각 증적 (L2)

- **이슈**: [#4155](https://github.com/edwardkim/rhwp/issues/4155)
- **계획서**: [`mydocs/plans/task_m100_4155.md`](../plans/task_m100_4155.md)
- **선행**: [stage1](task_m100_4155_stage1.md) · [stage2](task_m100_4155_stage2.md)
- **브랜치**: `task_m100_4155_hwp3_char_shade`
- **커밋**: `5c95b4619`
- **작업 시각**: 2026-08-09 KST

## 1. 변경 내용

`model::color` 에 `opaque_rgb`·`char_shade` 를 신설하고, `hidden_text.rs` 의 private 술어를
정본으로 승격했다(근거 주석 포함 이관). 소비처 8곳이 이를 호출한다.

| 위치 | 종전 판정식 | 동작 |
| --- | --- | --- |
| `renderer/svg.rs`·`web_canvas.rs`·`html.rs`·`skia/text_replay.rs`·`canvaskit_policy.rs` | `rgb != 0xFFFFFF && rgb != 0` | **무변경** (§3 로 확인) |
| `paint/text_v2.rs` | `shade_color != 0x00FFFFFF` (마스크 없음) | **변경** — sentinel·0 이 `textVisualEffect` 로 새던 것이 멎는다 |
| `paint/paint_op.rs` | `(shade & 0xFFFFFF) == 0xFFFFFF` | **변경** — 0 을 음영으로 보던 것이 멎어 HWP3 텍스트가 fill-only glyph replay 에 편입된다 |
| `parser/doc_info.rs` | 읽기 실패 시 `0xFFFFFF`(흰색) | 저장값이 sentinel 로 정렬 |

라이터 3종은 대상이 아니다 — "그려지는가"가 아니라 "값을 보존하는가"를 묻는 다른 질문이고,
흰색 음영(`0x00FFFFFF`)은 한컴산 코퍼스에 49건 실재해 저장에선 보존돼야 한다.

## 2. 시각 증적 — L1 (결함 수정)

수정 커밋 직전 소스(`71f607188~1`)로 빌드한 바이너리와 현재 바이너리의 `export-svg` 전 페이지
대조다.

| 표본 | 달라진 페이지 | 신규 음영 `<rect>` | HWP3 원본 |
| --- | ---: | --- | --- |
| `hwp3-sample16.hwp` | 4 | `#d8d8d8` × 8 | 팔레트 0 × 15% ×8 |
| `hwp3-sample5.hwp` | 1 | `#efefef` × 4 | 팔레트 0 × 6% ×4 |
| `hwp3-sample11.hwp` | 0 | — | §2.1 |
| `SO-SUEOP.hwp` | 0 | — | 전건 비율 0 |

건수가 이슈 본문의 HWP3 원본 실측("sample16 0×15% ×8", "sample5 0×6% ×4")과 **정확히**
일치한다. 수정 전 세 표본의 SVG 에는 `#d8d8d8`·`#efefef`·`#999999` 가 **0건**이었다 — 이 회색이
표 셀 채우기에서 온 것이 아니라 글자 음영으로 새로 그려진 것임을 확정한다.

`SO-SUEOP` 이 0 페이지인 것도 정합이다. 종전에도 검정 음영은 렌더러가 건너뛰었고
(`shade_rgb != 0` 조건) 지금은 sentinel 이라 역시 건너뛴다. **자체 렌더가 이 결함을 볼 수 없다는
사실 자체가 여기서 재확인된다** — 한컴만 보는 결함이었다.

### 2.1 `hwp3-sample11` — 저장 바이트에는 있고 렌더에는 없다

계약 테스트 ③ 은 통과한다(저장 HWP5 의 CHAR_SHAPE 에 `0x00999999`·`0x00d8d8d8` 존재).
그런데 SVG 에는 해당 색이 0건이다. 즉 그 글자 모양이 정의는 되어 있으나 렌더된 본문 런에서
참조되지 않는다. **원인은 이번에 확인하지 않았다** — 이슈의 한컴 저장본 실측도 sample11 에서
`0x00999999 × 1, 0x00d8d8d8 × 1` 로 극소수라 같은 양상으로 보인다. 저장 계약은 통과하므로
이 PR 의 판정에는 영향이 없다.

## 3. 시각 증적 — L2 (술어 통일)

L2 전후 `export-svg` 전 페이지 **바이트 동일**:

| 표본 | 결과 |
| --- | --- |
| `hwp3-sample11.hwp` | 전 페이지 동일 |
| `hwp3-sample5.hwp` | 전 페이지 동일 |
| `hwp3-sample16.hwp` | 전 페이지 동일 |
| `SO-SUEOP.hwp` | 전 페이지 동일 |

렌더 5곳은 위임만이므로 예상대로다. **다만 SVG 로는 `paint` 축 변화를 볼 수 없다** —
`is_fill_only_glyph_replay`·`textVisualEffect` 는 Skia/canvaskit 경로가 소비한다. 그쪽은 아래
Native Skia 게이트로 본다.

## 4. 검증 결과

| 게이트 | 결과 |
| --- | --- |
| `cargo test --profile release-test --lib` | **3,379 passed / 0 failed** |
| `cargo test --profile release-test --tests` | **5,510 passed / 0 failed** |
| `cargo fmt --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 (exit 0, 경고 0) |
| Native Skia 3종 | §5 |
| `wasm-pack build` | §5 |

## 5. 렌더러 행 추가 게이트

`mydocs/manual/pr_review/local_validation.md` §4.3 renderer 행 · §191 의 Skia 공식 3종.

| 명령 | 결과 |
| --- | --- |
| `cargo test --profile release-test --features native-skia skia --lib` | **58 passed / 0 failed** |
| `--features native-skia --test issue_2225_missing_picture_placeholder` | **2 passed / 0 failed** |
| `--features native-skia --test render_p37_direct_pdf_export` | **4 passed / 0 failed** |
| `wasm-pack build --target web --out-dir pkg` | **통과** — `pkg/` 생성 (`rhwp_bg.wasm` 외 6파일) |

`wasm-pack` 이 이 작업 환경에 설치돼 있지 않았다(`~/.cargo/bin` 에 rustup shim 만 있고 cargo
install 산물이 0개, Homebrew 에도 없음 — PATH 문제가 아니라 미설치였다). host fallback 으로
넘기지 않고 `cargo install wasm-pack`(0.15.0)으로 설치해 **표준 명령을 그대로 실행**했다.
`wasm-bindgen-cli` 0.2.125 는 wasm-pack 이 자동 설치했고 `wasm-opt` 최적화까지 완주했다.

`pkg/` 는 `.gitignore` 에 있어(`/pkg/`) 커밋 대상이 아니다 — 빌드 통과만 게이트다.

`dev_environment_guide.md` 는 `wasm-pack build` 를 요구하면서 설치 방법을 적어두지 않았다.
같은 환경을 새로 꾸리는 사람이 막히는 지점이므로 가이드에 한 줄 추가할 만하다(후속).

`paint` 축의 동작 변경 2건(`is_fill_only_glyph_replay`·`textVisualEffect`)이 실제로 소비되는
경로가 이 3종이다 — SVG 무회귀만으로는 덮이지 않는 범위를 여기서 닫는다.

## 6. 한컴 판정용 산출물

```bash
target/release-test/rhwp convert samples/SO-SUEOP.hwp /tmp/so-sueop-4155.hwp
target/release-test/rhwp convert samples/hwp3-sample11.hwp /tmp/sample11-4155.hwp
```

판정 기준: ① `SO-SUEOP` 본문의 검정 막대 소멸 ② 음영 표본의 회색 톤이 한컴 저장본과 일치.
최종 시각 판정 권위는 작업지시자다.
