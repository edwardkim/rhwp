# Task #359 Stage 1 — 드리프트 origin 정량화

## 목표

`samples/k-water-rfp.hwp` 의 LAYOUT_OVERFLOW 발생 원인 정량화.

## 진단 도구 활용

- `RHWP_TYPESET_DRIFT=1` (Task #321 시점 도입) — typeset 의 fit 누적 정량
- `RHWP_LAYOUT_TRACE=1` (Stage 1 임시 추가, 분석 후 제거) — layout entry 의 col_content.items 추적

## 1차 가설 검증 (모순 식별)

### 모순 1 — dump-pages vs LAYOUT_OVERFLOW 보고

`dump-pages -p 0` (페이지 1):
- items=33 (pi=0~31 + Shape pi=31 ci=0)
- used=728.1px, hwp_used 미표시

`LAYOUT_OVERFLOW` (export-svg `-p 2` 시):
- `page=0 col=0 para=34 y=1288.0 bottom=1028.9 overflow=259.1px`
- `page=0 col=0 para=35 y=1316.8 overflow=287.9px`

**page=0 인데 pi=34, pi=35** — 두 보고가 모순.

### 모순 해결 — page_index 의 의미

`PageContent.page_index` 는 **section 내 local index**, 글로벌 page index 가 아님.

- `-p 0` (글로벌 페이지 1) → section=0, page_num=1, **page_index=0**, items=33
- `-p 1` (글로벌 페이지 2) → section=0, page_num=2, **page_index=1**, items=25
- `-p 2` (글로벌 페이지 3) → section=1, page_num=1, **page_index=0**, **items=36**

`-p 2` 의 LAYOUT_OVERFLOW 가 `page=0` 으로 표시된 이유 = section 1 의 첫 페이지가 local index 0. 두 번 호출이 아니라 **다른 section 의 첫 페이지**.

## 2차 가설 검증 (진짜 원인)

### 페이지 3 (section=1, page_num=1) 의 dump

```
=== 페이지 3 (global_idx=2, section=1, page_num=1) ===
  body_area: x=80.0 y=113.3 w=640.0 h=915.5
  단 0 (items=36, used=915.5px, hwp_used≈1226.7px, diff=-311.2px)
```

| 지표 | 값 |
|------|----|
| col_height (가용) | 915.5 |
| pagination 의 used | 915.5 (= col_height 한도와 동일) |
| **hwp_used (LINE_SEG.vpos 추정)** | **1226.7** |
| **drift** | **-311.2** (pagination 이 311.2 px 적게 산정) |

→ pagination 단계가 **311.2 px 만큼 fit 산정을 적게** 함. 36 items 가 한 페이지 안에 들어간다고 잘못 판정. 실제 콘텐츠는 1226.7 px 필요 → layout 단계의 y 진행이 col_bottom 을 초과.

### LAYOUT_OVERFLOW 의 정량

| pi | y | overflow |
|----|---|----------|
| 32 | 1054.4 | 25.5 |
| 33 | 1185.6 | 156.7 |
| 34 | 1288.0 | 259.1 |
| 35 | 1316.8 | 287.9 |

마지막 pi=35 의 overflow 287.9 ≈ pagination drift 311.2 ± 약간. layout 의 vpos 보정이 일부 흡수 (drift > overflow).

### TYPESET_DRIFT 의 페이지 1 정량

페이지 1 (section=0) 의 typeset_drift 출력에서:
- 모든 `diff=+3.2` 또는 `diff=+5.3` (line_spacing 누적이 fmt_total 에 포함되었으나 vpos_h 에는 없는 정상 trail_ls 차이)
- pi=0~31 누적 후 `cur_h=639.0` (col_height 905.5 이내)
- pi=32 부터 새 페이지 (페이지 2) `cur_h=0.0 first_vpos=0`

→ section 0 의 페이지 1 자체는 정상.

### 페이지 3 (section=1, page_num=1) 의 typeset_drift

- pi=0~35 가 한 페이지 (section 1 의 첫 페이지) 에 들어감
- 마지막 누적 `cur_h ≈ 905~915px` 정도로 col_height 안에 들어간다고 판정
- 그러나 hwp_used (LINE_SEG.vpos 누적) 는 **1226.7**

**drift origin**: typeset 의 fit 산정 (line_height + line_spacing 합) 과 LINE_SEG.vpos 의 누적이 다르게 진행. typeset 이 **311.2 px 만큼 짧게 측정**.

## 핵심 모순 (Stage 2 의 분석 대상)

`format_paragraph::total_height` 산정과 `LINE_SEG.vpos` 누적이 **section 1 의 첫 페이지에서 311.2 px 어긋남**:

- 빈 문단 다수 (h=4.0~18.7px) 의 누적
- 각 문단의 line_spacing trail (10~15px) 누적
- spacing_before / spacing_after 누적
- 표 / 그림 (TAC) 의 fit 반영

본 page 의 36 items 중 어느 누적이 311 px drift 의 origin 인지가 Stage 2 의 핵심 질문.

## layout.rs 정합성 우려 (Stage 2 추가 점검 — 작업지시자 결정 C)

작업지시자 지시: 컨트리뷰터들 (planet6897, oksure, postmelee, seanshin) 의 누적 변경이 layout.rs 의 정합성을 깨뜨렸을 가능성. Stage 2 안에서 **코드 중복 + 문제 여지** 점검 추가.

| 자료 | 크기 |
|------|------|
| `src/renderer/layout.rs` | 3,178 줄 (단일 파일) |
| `src/renderer/typeset.rs` | 2,121 줄 |
| `src/renderer/layout/paragraph_layout.rs` | 2,796 줄 |
| `src/renderer/layout/table_layout.rs` | 2,223 줄 |
| `src/renderer/layout/shape_layout.rs` | 2,188 줄 |
| **layout 영역 합계** | **18,889 줄** |

최근 6 commit 중 5건이 layout.rs 직접 변경 (Task #347, #321~#332, #291, #318, #295). 누적 변경의 정합성 우려.

## 정리 사항

- 진단 trace (`RHWP_LAYOUT_TRACE`, page entry trace) 정리 완료 (Stage 2 후 재제거)
- 코드 변경 0 (분석만)

## 다음 단계 (Stage 2)

확장된 범위:

1. **drift origin 정량 분석** — 36 items 중 어느 항목들이 311.2 px drift 의 누적 origin
2. **layout.rs / typeset.rs / paragraph_layout.rs 의 height 산정 path 점검** — 같은 데이터 (line_height, line_spacing, vpos) 가 여러 함수에서 다른 방식으로 산정되는지
3. **코드 중복 + 정합성 우려 식별** — 컨트리뷰터별 변경의 충돌 확인
4. **수정 방향 확정** — fit 산정 통일 + 코드 중복 정리 (있다면)
