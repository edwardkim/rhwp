---
issue: 630
milestone: m100
branch: local/task630
stage: 6 — 잔여 1.05 mm 마진 정정 (Issue #635 흡수)
created: 2026-05-06
status: 완료 — 시각 판정 대기
---

# Task #630 Stage 6 완료 보고서 — inline_tabs RIGHT + leader 본문 우측 끝 클램프

## 1. 배경

Stage 5 후 작업지시자 시각 판정: "여전히 왼쪽으로 밀려 있음". PDF (한컴 2022 출력) 와 정밀 비교 시 잔여 1.05 mm 마진 발견 (Stage 5 보고서 §10).

원래 별도 후속 task (Issue #635) 로 등록했으나, 본 task 의 시각 판정 ★ 통과를 위해 **본 task 안에서 흡수**하기로 결정.

## 2. 본질 분석 (디버그 print 검증)

`compute_char_positions` 가 SVG 렌더 시점에 **inline_tabs 분기** (line 358-) 사용. has_custom_tabs 분기 (Stage 6 v1 정정 영역) 는 paragraph_layout.rs 의 tab_leaders 추출 시점에만 사용, SVG 위치 결정에는 영향 없음.

디버그 print 결과:
```
[TAB inline] x_before=234.00 tab_width_px=230.44 tab_type=0x0203 ext=[17283, 0, 515, 0, 0, 0, 9]
                                                  text_chars=['개','발',' ','대','상',' ','기','술','·','제','품','의',' ','개','요']
[TAB inline] x_after=464.44
```

- `tab_type = 0x0203 = 515` = `(2<<8) | 3` = RIGHT + fill_type=3 (점)
- `ext[0]` 가 라인마다 다름 (한컴이 right-tab 시점에 `우측 끝 - 한컴_seg_w - 한컴_x_at_tab` 으로 저장) — 모든 라인에서 `x_at_tab + tab_width_px ≈ 464.44` 정합

LEFT fallback (`x = tab_target.max(x)`) 결과 = `x_at_tab + ext[0]` ≈ 464.4 (run 내 상대) → 600.7 px (절대). 본문 우측 끝 (642.5 run-relative, 718.09 절대) 까지 미달 — 약 1.16 mm 잔여 마진.

원인: 한컴이 ext[0] 저장 시 사용한 한컴_seg_w 와 rhwp 측정 our_seg_w 의 미세 차이 (~4 px). 정정 1 (`·` 측정 통일) 이후에도 폰트 메트릭 미세 차이로 잔존.

## 3. 정정 (Stage 6 v2)

`src/renderer/layout/text_measurement.rs` inline_tabs 분기 (3 곳: estimate_text_width, EmbeddedTextMeasurer::compute_char_positions, WasmTextMeasurer::compute_char_positions):

```rust
let high_byte = (tab_type_raw >> 8) & 0xFF;
let fill_low = tab_type_raw & 0xFF;
match (high_byte, tab_type_raw) {
    (_, 1) => { /* 기존 raw 1 분기 — 호환 유지 */ }
    (_, 2) => { /* 기존 raw 2 분기 — 호환 유지 */ }
    (2, _) if fill_low != 0 => {
        // RIGHT + leader: ')' 끝이 본문 우측 끝까지 정렬
        // x = body_right - our_seg_w. 한컴 ext[0] 무시
        // (한컴_seg_w 와 our_seg_w 미세 차이로 본문 우측 끝 미달).
        let seg_w = measure_segment_from(...);
        x = (body_right - seg_w).max(x);
    }
    _ => { x = tab_target.max(x); /* 기본 LEFT fallback */ }
}
```

**케이스별 명시 가드** (`feedback_hancom_compat_specific_over_general` 정합):
- `high_byte == 2 && fill_low != 0` 만 신규 분기 (RIGHT + leader)
- 다른 raw 값 (LEFT raw 0/1, CENTER raw 2, 기존 호환 분기) 영향 없음

## 4. 결과 (정량)

### 4-1. aift p4 권위 케이스

| 메트릭 | Stage 5 | Stage 6 v2 | 변화 |
|--------|---------|-----------|------|
| paren_x 분포 | 600.25 ~ 601.39 (spread 1.13) | **모두 605.45 (spread 0.00)** | 완전 정합 |
| ')' 끝 평균 | 713.66 px | **718.12 px** = 본문 우측 끝 | +4.46 px 우측 |
| SVG 우측 마진 | 1.16 mm | **−0.01 mm** | -1.17 mm |
| PDF 우측 마진 | 0.11 mm | (정합) | |

23 라인 모두 정확히 paren_x = 605.45, ')' 끝 = 718.12 = 본문 우측 끝. **완벽한 right-tab 정렬**.

### 4-2. 광범위 회귀 검증

```
164 fixture / 1614 페이지 / 페이지 수 회귀 0
cargo test --lib --release: 1135 passed / 0 failed
test issue_630: 1 passed
```

### 4-3. svg_snapshot 갱신

| Snapshot | Stage 5 | Stage 6 v2 |
|----------|---------|-----------|
| issue_147 (aift p3) | 갱신 (Stage 5) | **재갱신** (Stage 6) |
| issue_267 (KTX TOC) | byte-identical | **갱신** (leader 길이 보정 영향) |
| 기타 4 snapshot | passing | **passing** |

**KTX 변화 분석**: 페이지 번호 위치 (last_x ≈ 690.8) 동일. leader 도트 라인 (`<line x2="...">`) 의 끝점이 686.09 → 670.68 로 약 15 px 단축. paragraph_layout.rs:1421-1450 의 leader 길이 보정 로직이 새 paren_x 위치 (605.45) 에 맞춰 leader 끝을 짧게 조정한 효과. 페이지 번호 자체는 영향 없음.

⚠ KTX 시각 판정 필요 — leader 도트 단축이 시각적 회귀인지 향상인지 작업지시자 확인.

## 5. 리스크 분석

### 5-1. 본 정정의 영향 범위

- **inline_tabs 의 RIGHT + leader (fill ≠ 0)** 케이스만 영향
- 한컴 HWP 인라인 탭 + 점선/실선 fill 사용한 모든 right-tab (목차, 표 우측 정렬, dot leader 등)
- aift / KTX 외 다른 fixture 의 RIGHT + leader 케이스가 본 정정으로 본문 우측 끝까지 정렬 → PDF 정합 향상 또는 회귀

### 5-2. 회귀 가능성

- LEFT 탭 (raw 0/1) / CENTER (raw 2) / RIGHT + fill=0 / DECIMAL 영역 영향 없음 (케이스별 명시 가드)
- 광범위 sweep 페이지 수 회귀 0
- cargo test 회귀 0
- KTX leader 도트 단축은 시각 판정으로 결정

## 6. 산출물

- `src/renderer/layout/text_measurement.rs` (inline_tabs 분기 RIGHT + leader 처리 추가, 3 곳)
- `tests/golden_svg/issue-147/aift-page3.svg` 재갱신
- `tests/golden_svg/issue-267/ktx-toc-page.svg` 갱신
- `output/svg/task630_stage6_after/{aift_004.svg, KTX_002.svg}` (시각 판정 자료)

## 7. 시각 판정 요청

작업지시자 시각 판정:
1. **aift p4 (`output/svg/task630_stage6_after/aift_004.svg`)**: `(페이지 표기)` 가 본문 우측 끝까지 정렬 — PDF 정합. ★ 통과 여부.
2. **KTX p1 (`output/svg/task630_stage6_after/KTX_002.svg`)**: 페이지 번호 위치 동일, leader 도트 끝점 15 px 단축 — 시각적 회귀 여부 확인.

## 8. 후속 처리

- **Issue #635 close** — 본 task 안에서 정정 흡수
- 시각 판정 ★ 통과 시 PR #636 force-push 또는 추가 commit 으로 update
