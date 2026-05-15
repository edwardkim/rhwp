# Task #902 최종 결과 보고서 — WMF renderer 근본 개선

**이슈**: [edwardkim/rhwp#902](https://github.com/edwardkim/rhwp/issues/902)
**브랜치**: `local/task902` (base: `upstream/devel @ 72a6bbc3`)
**v1 → v2 전환**: "WMF unit scale 정합" → "WMF renderer 근본 개선"
**Timebox**: open-ended (작업지시자 결정)

## 1. 처리 개요

### 1.1 본 task 의 본질

issue #902 의 원래 보고: "WMF 텍스트가 한컴 viewer 와 비교 시 크기/간격 미세 차이". 분석 진행 시 단일 dx 휴리스틱 수정만으로는 정합 미달, **WMF renderer 의 fundamental 갭 다수** 발견 → scope 재정의 (v2).

### 1.2 v1 → v2 전환

| 항목 | v1 | v2 |
|------|----|----|
| Scope | dx 단일 fix | WMF renderer 근본 개선 |
| Stage 수 | 5 | 9 (Stage 3~9 신규) |
| Timebox | ~3일 | open-ended |
| 진행 방식 | 매 stage 승인 | 작업지시자 자동승인 지시 (PR 직전까지) |
| 시각 검증 | 작업지시자 시각 비교 | rsvg-convert 셀프 검증 |

## 2. Stage 별 처리 요약

| Stage | 내용 | 결과 |
|-------|------|------|
| 1 (v1) | 다중 sample WMF binary + PDF 추출 | 완료, commit |
| 2 (v1) | WMF binary 정밀 디코드 + Task #896 분석 오류 정정 | 완료, commit |
| **3** | **DX byte-aware indexing + absolute X** | **본 issue 의 ROOT CAUSE bug fix** |
| 4 | META_SETVIEWPORTEXT/ORG 구현 + MM_ANISOTROPIC ratio | viewport 사용 sample 정합 |
| 5 | EXTTEXTOUT options flags (ETO_OPAQUE/PDY) | flag 사용 sample 정합 |
| 6 | OFFSET WINDOW/VIEWPORT, SCALEVIEWPORTEXT 구현 | offset/scale 사용 sample 정합 |
| 7 | 폰트 metric 정합 (font-family 체인) | garbled fallback 제거 + cross-platform 한국어 fallback |
| 8 | 광범위 회귀 검증 | 1412 passed / 0 failed |
| 9 | 최종 보고서 + PR | 본 보고서 + PR 생성 (작업지시자 승인 대기) |

## 3. ROOT CAUSE — Stage 3 의 핵심 bug

WMF EXTTEXTOUT 의 DX 배열은 **MBCS byte index**. Korean wide char (2 byte) 는 DX 2 entry 차지 (실제 advance + 0).

**기존 버그**:
```rust
for (i, s) in text_content.graphemes(true).enumerate() {
    let dx = *record.dx.get(i - 1).unwrap_or(&0);  // grapheme index — 오류
}
```

grapheme index 로 접근 시 wide char 마다 매 둘째 entry (=0) 접근 → 글자 위치 잘못 누적.

**수정**:
```rust
let width = s.width().max(1);  // unicode_width: Korean=2, ASCII=1
let advance: i32 = (0..width)
    .map(|k| i32::from(*record.dx.get(dx_idx + k).unwrap_or(&0)))
    .sum();
acc_x += advance;
dx_idx += width;
// tspan x = acc_x (absolute, 폰트 metric 독립)
```

검증 (수정 전 후 SVG tspan x 값):
- Before: `<tspan x="291">전</tspan> <tspan x="291">산</tspan>` (산 위치 잘못)
- After: `<tspan x="291">전</tspan> <tspan x="408">산</tspan>` ✓ monotonic 정합

## 4. 변경 파일 요약

| 파일 | 변경 | 효과 |
|------|------|------|
| `src/wmf/converter/svg/mod.rs` | EXTTEXTOUT DX byte-aware + absolute X, ETO_OPAQUE/PDY, viewport/offset/scale records | 본 task 의 핵심 fix |
| `src/wmf/converter/svg/device_context.rs` | Viewport struct + offset/scale mutators | MM_ANISOTROPIC ratio 계산 |
| `src/wmf/converter/svg/util.rs` | font-family chain 정합 + garbled filter | 폰트 fallback robust |

## 5. 회귀 검증

| 검증 | 결과 |
|------|------|
| `cargo test --release --all-targets` | 1412 passed / 0 failed |
| `cargo test --release --test svg_snapshot` (golden SVG) | 8 / 8 passed |
| sample14 SVG export (Task #860 fixture) | 11 페이지 정상 |
| sample16 SVG export (#902 본 대상) | 64 페이지 정상 |
| sample17 SVG export | 12 페이지 정상 |
| sample18 SVG export | 69 페이지 정상 |
| sample19 SVG export | **pre-existing parser 실패** (WMF 무관, follow-up) |
| rsvg-convert PNG 셀프 검증 | sample16/18 정상 |

## 6. 잔존 한계 (follow-up)

| 영역 | 잠재 follow-up |
|------|--------------|
| WMF 굴림체 ↔ open 한국어 폰트 glyph metric 미세 차이 | 폰트 임베딩 (`--embed-fonts=full` 활용) |
| 사용 빈도 낮은 WMF records | Region/Palette/Clip/Pixel ops, set_mapper_flags, set_layout 등 |
| HWP3 sample19 parser 실패 | 별도 issue (parser 영역) |

## 7. 산출물

### 7.1 계획서

- `mydocs/plans/task_m100_902.md` (v1 수행 계획서, 역사 기록)
- `mydocs/plans/task_m100_902_impl.md` (v1 구현 계획서, 역사 기록)
- `mydocs/plans/task_m100_902_v2.md` (v2 수행 계획서)
- `mydocs/plans/task_m100_902_impl_v2.md` (v2 구현 계획서)

### 7.2 Stage 보고서

- `mydocs/working/task_m100_902_stage{1,2,3,4,5,6,7,8}.md`

### 7.3 최종 보고서

- `mydocs/report/task_m100_902_report.md` (본 파일)

### 7.4 소스 변경 (commit)

```
21eaed4a Task #902: 수행 계획서 작성
f0f871a7 Task #902: 구현 계획서 작성 (Stage 1~5)
45e16d90 Task #902 Stage 1: 다중 sample WMF binary + PDF 추출
a978a47b Task #902 Stage 2: WMF binary 정밀 디코드 + Task #896 분석 오류 정정
a0658b31 Task #902 v2 Stage 3: WMF EXTTEXTOUT DX byte-aware indexing fix
dbe8cb80 Task #902 v2: 다중 sample fixture 추가 (sample17/18/19 + PDF)
53a3063f Task #902 v2 Stage 4: META_SETVIEWPORTEXT/ORG 구현 + MM_ANISOTROPIC ratio 정합
a7040891 Task #902 v2 Stage 5: EXTTEXTOUT options flags 처리 (ETO_OPAQUE/PDY)
fa3e59aa Task #902 v2 Stage 6: 미구현 WMF records 완성 (offset/scale)
4c64c670 Task #902 v2 Stage 7: 폰트 metric 정합 (font-family 체인 조정)
f76f1582 Task #902 v2 Stage 8: 광범위 회귀 검증 (다중 sample + golden SVG)
```

## 8. PR 생성 시점

작업지시자 명시 승인 후 `gh pr create` 실행. PR template:
- Base: `upstream/devel`
- Head: `local/task902`
- Title: "Task #902: WMF renderer 근본 개선 (v2)"
- Body: 본 보고서 요약 + closes #902

## 9. orders 갱신

`mydocs/orders/{오늘날짜}.md` 의 Task #902 상태 갱신 (작업 완료 → PR 대기).
