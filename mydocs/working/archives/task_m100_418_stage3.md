# Stage 3 완료 보고서 — Task M100 #418

## 작업 내용

회귀 테스트 추가 + 자동 검증 종합 + WASM 빌드.

### 변경 — `tests/issue_418.rs` (신규)

```rust
//! Issue #418: hwpspec.hwp 20 페이지의 빈 문단 + TAC Picture 가
//! paragraph_layout 와 layout_shape_item 양쪽에서 emit 되어 SVG 에 두 번
//! 그려지는 회귀.
//! ...

#[test]
fn hwpspec_page20_no_duplicate_image_emit() {
    let bytes = fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwpspec.hwp"))?;
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)?;
    let svg = doc.render_page_svg_native(19)?;
    let image_count = svg.matches("<image").count();
    assert_eq!(image_count, 3, "기대 3, 실제 {image_count}");
}
```

## 자동 검증 결과

| 항목 | 명령 | 결과 |
|------|------|------|
| 회귀 테스트 | `cargo test --test issue_418` | ✅ **1/1 passed** |
| 전체 lib test | `cargo test --lib` | ✅ **1023 passed**, 0 failed |
| svg_snapshot | `cargo test --test svg_snapshot` | ✅ **6/6 passed** (다른 샘플 무회귀) |
| clippy | `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | `docker compose run --rm wasm` | ✅ 1m 22s, 4,101,019 bytes (Task #416 → +769 bytes, 가드 추가) |

## 시각 검증 결과 (Stage 2 완료 시점)

| 시나리오 | 결과 |
|----------|------|
| `samples/hwpspec.hwp` 20 페이지 image 개수 | ✅ **6 → 3** (이중 출력 해소) |
| 제거된 중복 좌표 | 442.76 / 601.43 / 738.76 (layout.rs 의 약 2.67px 위 중복 emit) |
| 남은 정상 좌표 | 445.43 / 604.09 / 741.43 (paragraph_layout 의 정상 emit) |
| `samples/hwpspec.hwp` 1 페이지 (Task #416 효과) | ✅ 보존 |
| 다른 hwp 샘플 | ✅ 무회귀 (svg_snapshot 6/6) |

## 메모리 / 트러블슈팅 갱신 (Stage 3 부분)

본 task 는 Task #376 의 정정 누락 회귀이므로, **별도 메모리 / 트러블슈팅 갱신 검토**:

- 메모리 후보: "이슈 close 만 하지 말고 정정 commit 이 devel 에 머지됐는지 확인" — 메인테이너 작업 절차 강화
- 트러블슈팅 후보: 본 결함이 같은 영역 (paragraph_layout + layout_shape_item TAC 이중 emit) 에서 재발 가능성 높음 → 회귀 방지 가이드

→ 최종 보고서에서 결정.

## 다음 단계

- 최종 보고서 작성
- 작업지시자 승인 시 머지 + close + 메모리/트러블슈팅 갱신

## 산출물

- 신규 파일: `tests/issue_418.rs`
- WASM 빌드: `pkg/rhwp.js`, `pkg/rhwp_bg.wasm` (4,101,019 bytes)
- 본 보고서: `mydocs/working/task_m100_418_stage3.md`
