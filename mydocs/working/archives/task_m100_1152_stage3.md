# Task #1152 Stage 3 — 회귀 + 인접 케이스 + clippy

- 이슈: [#1152](https://github.com/edwardkim/rhwp/issues/1152)
- 브랜치: `local/task1152`
- 작성일: 2026-05-28

## 1. 인접 케이스 페이지 수 비교 (패치 적용 전후)

`samples/` 전수 스캔에서 후보로 식별된 인접 케이스 + 본 이슈 대상.

| sample | 패치 전 | 패치 후 | Δ |
|--------|---------|---------|---|
| `2022년 국립국어원 업무계획.hwp` (본 이슈) | 35 | **35** | 0 |
| `kps-ai.hwp` | 80 | 80 | 0 |
| `2025년 기부·답례품 실적 지자체 보고서_양식.hwpx` | 30 | 30 | 0 |
| (비공개 sample A) | 185 | 185 | 0 |

→ **페이지 수 변동 0**. 페이지 32 → 33 의 아이템 재배치만 발생, 신규 페이지 생성/삭제 없음.

## 2. 전체 테스트 회귀 (`cargo test --release --no-fail-fast`)

`/tmp/full_test.log` 결과:

| 항목 | 값 |
|------|---|
| 통과 binary | 약 40+ (전체 1308 unit + 70+ integration) |
| 실패 binary | 2 (모두 사전 존재 — stash 검증으로 확인) |
| 신규 실패 | **0** |

### 사전 존재 실패 (본 패치 무관)

1. `tests/issue_598_footnote_marker_nav.rs`
   - `issue_598_body_footnote_marker_has_hit_and_cursor_unit` — 좌표 hit-test 실패
   - `issue_598_second_body_footnote_marker_has_same_cursor_unit` — 좌표 hit-test 실패
2. `tests/svg_snapshot.rs`
   - `issue_267_ktx_toc_page` — golden SVG mismatch
   - `issue_617_exam_kor_page5` — golden SVG mismatch
   - `issue_677_bokhakwonseo_page1` — golden SVG mismatch

stash 으로 패치 제거 후 동일 실패 5건 확인 → 본 패치와 무관 (별도 이슈로 분리 권장).

## 3. 신규 회귀 테스트

`tests/issue_1152_intra_para_vpos_reset.rs`:

```
running 1 test
test issue_1152_별첨_box_starts_page_33_not_32 ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured
```

## 4. clippy

| 명령 | 결과 |
|------|------|
| `cargo clippy --release --lib --no-deps -- -D warnings` | ✅ 통과 |
| `cargo clippy --release --test issue_1152_intra_para_vpos_reset --no-deps -- -D warnings` | ✅ 통과 |

## 5. fmt

`cargo fmt -- --check` 는 전역 스캔이라 사전 드리프트(다른 파일 다수 + typeset.rs 의 2059, 2687, 2847, 2868) 가 포함되지만, **본 패치가 추가한 라인 (typeset.rs:2244-2263) 은 드리프트 0**.

CLAUDE.md 규칙 (기능 브랜치: 무관한 rustfmt diff 금지) 준수: 사전 드리프트는 본 브랜치 범위 외.

## 6. 결론 / 다음 단계

- 패치 적용 후 페이지 수 변동 0, 신규 회귀 0.
- 사전 실패 5건은 본 패치와 무관 — 별도 이슈로 처리 권장.
- 신규 회귀 테스트 통과 + lib/test clippy 통과 + 본 패치 라인 fmt clean.

→ Stage 4 (시각 검증 with `--debug-overlay`) 로 진행.
