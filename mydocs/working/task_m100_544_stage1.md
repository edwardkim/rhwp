# Task #544 Stage 1 완료 보고서

**제목**: TDD 통합 테스트 (RED) + fix 위치 정밀 진단 + 광범위 사전 평가
**브랜치**: `local/task544`
**이슈**: https://github.com/edwardkim/rhwp/issues/544

---

## 1. TDD 통합 테스트 추가 (RED 확인)

`integration_tests.rs` 에 `test_544_passage_box_coords_match_pdf_p4` 추가.

페이지 4 col 0 [7~9] passage 박스 좌표 PDF 정합 검증:
- `box_top_y` = 233.8 px (±2)
- `box_left_x` = 117.0 px (±2)
- `box_width` = 425.1 px (±2)

```
test test_544_passage_box_coords_match_pdf_p4 ... FAILED
[7~9] 박스 top y=224.43 가 PDF 기대값 233.80 (±2 px) 와 일치해야 함.
```

`#[ignore]` attribute 적용 (Stage 2 fix 적용 시 GREEN 기대). 기존 1120 단위
테스트 모두 통과.

## 2. fix 위치 정밀 진단

### 2.1 박스 top y 차이 (-9.4 px) 본질

vpos correction 추적 (`RHWP_VPOS_DEBUG=1`):

```
VPOS_CORR: pi=82 prev_pi=81 ...  ← pi=82 만 적용
(pi=81 의 vpos correction 미출력 = skip)
```

**원인**: `layout.rs:1481` 가드:
```rust
if !(seg.vertical_pos == 0 && prev_pi > 0) {
    // vpos correction logic
}
```

페이지 4 의 pi=80 (페이지 시작 paragraph) 은 `seg.vertical_pos = 0`,
`prev_pi = 80 > 0` → 가드 활성 → pi=81 의 vpos correction skip → pi=81 paragraph_layout
호출 시 `y_start = 224.43` (sequential, trailing-ls 716 HU 누락).

만약 vpos correction 이 정상 작동했다면:
```
end_y = col_area.y + (pi=81 IR vpos - base) * scale
     = 209.76 + 1816 * 96/7200 = 209.76 + 24.21 = 233.97 ≈ PDF 233.8 ✓
```

가드 의도: vpos reset (PartialParagraph line_segs 8 이후 vpos=0) 보호.
페이지 시작 paragraph 는 정상 vpos=0 인데 가드가 구분 못 함.

### 2.2 박스 left x / width 차이 (+11.5 / -22.6 px) 본질

`paragraph_layout.rs:2695-2697`:
```rust
(box_x, box_w) = (col_area.x + box_margin_left, col_area.width - box_margin_left - box_margin_right)
```

`box_margin_left = para_style.margin_left = ParaShape margin_left / 2` (2배 저장).

페이지 4 [7~9] 박스 group 의 첫 paragraph = pi=82 (Task #540 Stage 4 fix 후
pi=81 push skip). pi=82 `margin_left = 1704 HU / 2 = 11.36 px`.

→ SVG box_x = col_area.x (117.17) + 11.36 = 128.53 px ≈ 측정값 128.5 ✓.

PDF box_x = 117.0 (margin 미적용).

**결론**: paragraph margin_left/right 가 박스 outline 좌표에 적용되는 것이
PDF 와 다름. PDF 는 paragraph margin 을 텍스트 inset 으로만 사용, 박스
outline 은 col_area 전체.

## 3. 광범위 사전 평가

### 3.1 다른 샘플 PDF vs SVG 박스 비교

페이지 1 비교 결과:

| 샘플 | PDF 박스 | SVG 박스 | 차이 |
|------|----------|----------|------|
| exam_math | x=69.3, w=889.8 | x=71.8, w=889.3 | +2.5/-0.5 (거의 일치) |
| exam_eng | x=117.0, w=420.0 | x=117.2, w=419.5 | +0.2/-0.5 (거의 일치) |
| 21_언어_기출 (p4) | x=117.0, w=425.1 | x=128.5, w=402.5 | **+11.5/-22.6** |
| synam-001 | x=78.7, w=643.5 | x=37.8, w=716.3 | -40.9/+72.8 (페이지 layout 다름) |

### 3.2 paragraph margin_left 분포

| 샘플 | border 가진 첫 paragraph margin_left | 박스 차이 |
|------|------------------------------------|----------|
| exam_math (표 outline 우세) | n/a | n/a |
| exam_eng (ps_id=70) | **0** HU | 박스 거의 일치 (margin=0 → box_x=col_area.x) |
| 21_언어_기출 (pi=82 ps_id=11) | **1704** HU | +11.5 px 시프트 (margin_left/2=11.36 px) |

**결론**: 모든 샘플의 SVG 박스 산식은 동일 (`col_area.x + margin_left`).
margin_left=0 인 샘플은 우연히 PDF 와 일치. 21_언어_기출 만 큰 margin 으로
차이 노출.

→ fix 는 모든 샘플에 적용해야 (A안 광범위). margin_left=0 샘플은 변경
없음 (col_area.x + 0 = col_area.x). margin_left > 0 샘플은 변경 (PDF 일치).

### 3.3 회귀 위험 재평가

| 케이스 | 회귀 위험 | 완화 |
|--------|-----------|------|
| margin_left=0 paragraph (exam_eng 등) | 없음 (변경 없음) | - |
| margin_left>0 paragraph (21_언어_기출) | PDF 일치 (개선) | - |
| 셀 내부 paragraph border | 미확인 | Stage 3 검증 |
| wrap=Square 호스트 (border_box_override) | 미확인 | Stage 3 검증 |
| 텍스트 좌표와의 정합 | 위험 (텍스트 effective_margin_left 별도 적용) | Stage 2 fix 시 텍스트 위치 보존 |

## 4. fix 방향 정리

### 4.1 박스 left/width fix

`paragraph_layout.rs:2695-2697` 변경:
```rust
// 현재
(box_x, box_w) = (col_area.x + box_margin_left, col_area.width - box_margin_left - box_margin_right)

// 정정 (A안: 광범위)
(box_x, box_w) = (col_area.x, col_area.width)
```

`border_box_override` 분기는 wrap=Square 호스트 케이스 — 별도 분석 후 보존
또는 동일 로직 적용.

### 4.2 박스 top y fix

옵션 (a): `layout.rs:1481` 가드 완화 — `seg.vertical_pos == 0 && prev_pi > 0`
조건을 vpos reset case 만 인식하도록 정밀화. 페이지 첫 paragraph 는 vpos
correction 적용.

옵션 (b): `paragraph_layout.rs:786` bg_y_start 산출 시 prev paragraph 의
trailing-ls 만큼 보정. paragraph_layout 함수 인자에 prev_trailing_ls 추가.

옵션 (c): paragraph_layout 호출 전 build_single_column 에서 prev paragraph 의
trailing-ls 를 y_offset 에 미리 더해서 전달.

(a) 가 가장 본질적 (vpos correction 가드 정밀화). (b)/(c) 는 paragraph border
한정 보정. (a) 는 회귀 위험 큼 (vpos correction 자체 변경).

### 4.3 권장 fix 범위

**A안 부분 적용 권장**:
- 박스 left/width: `(col_area.x, col_area.width)` 로 변경 — 광범위
- 박스 top y: 옵션 (b) `bg_y_start` 만 보정 — paragraph border 한정 (회귀 영향 최소)

이유:
- 박스 left/width 는 paragraph_layout 내부 라 영향 범위 명확.
- 박스 top y 의 옵션 (a) (vpos correction 가드 변경) 는 다른 paragraph 위치도
  변경되어 회귀 위험 매우 큼. (b) 는 박스 outline 만 영향, 본문 텍스트 위치
  변경 없음.

## 5. 작업지시자 입력 요청

다음 사항 결정 부탁드립니다:

1. **fix 범위 확정**: A안 부분 적용 (위 4.3 권장) 진행 OK?
2. **셀 내부 paragraph border 케이스**: 셀 내부에도 paragraph border 박스가 있는
   샘플이 있다면 알려주세요. Stage 2 시 케이스 보존 fix 필요.
3. **wrap=Square 호스트 (border_box_override) 케이스**: wrap 박스가 있는 샘플이
   있다면 알려주세요.
4. **한컴 2020 / 한컴독스 검증**: 한컴 2010 PDF 외 환경 비교 결과가 있다면
   알려주세요. 메모리 룰 [feedback_pdf_not_authoritative] 에 따라 다중 환경
   검증 권고.

## 6. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/integration_tests.rs` | TDD 테스트 1건 (RED, +96 LOC) |
| `mydocs/working/task_m100_544_stage1.md` | 본 보고서 |

## 7. 다음 단계

작업지시자 입력 (위 4가지) 수신 후 Stage 2 진행:
- A안 부분 적용 → fix → Stage 1 RED 테스트 GREEN
- Stage 3: 광범위 회귀 검증 + 최종 보고서

---

승인 후 Stage 2 진행합니다.
