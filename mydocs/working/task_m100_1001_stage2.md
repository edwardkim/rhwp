# Task #1001 Stage 2 — Fix 후보 평가

이슈: [#1001](https://github.com/edwardkim/zhwp/issues/1001)
Stage 1: [`task_m100_1001_stage1.md`](task_m100_1001_stage1.md)

## 1. Sample sweep — paper-based + bit 1/2 분포

전체 sample 의 `pgbf.attr` 비트 분포 (sweep 결과):

| 파일 | attr | bit 0 | bit 1 | bit 2 | spacing |
|------|------|-------|-------|-------|---------|
| hwp3-sample16.hwp (HWP3 원본) | `0x00` | 0 | 0 | 0 | 1420 |
| hwp3-sample16-hwp5.hwp | `0x01` | 1 | 0 | 0 | 1420 |
| aift.hwp | `0x01` | 1 | 0 | 0 | 1417 |
| exam_kor.hwp | `0x01` | 1 | 0 | 0 | 0 |
| exam_math.hwp | `0x01` | 1 | 0 | 0 | 0 |
| exam_eng.hwp | `0x01` | 1 | 0 | 0 | 0 |
| biz_plan.hwp | `0x01` | 1 | 0 | 0 | 1417 |
| 통합재정통계(2014).hwp | `0x01` | 1 | 0 | 0 | 1417 |
| 복학원서.hwp | `0x01` | 1 | 0 | 0 | 1417 |

**핵심**: paper-based sample 은 모두 `bit 1 = 0, bit 2 = 0` (머/꼬 미포함). 한컴 default 패턴.

## 2. Fix 후보 평가

### 후보 A — bit 1/2 처리 추가 (정공법, spec 정합)

```rust
let paper_based = (pbf.attr & 0x01) != 0;
let header_inside = (pbf.attr & 0x02) != 0;
let footer_inside = (pbf.attr & 0x04) != 0;
// ... base / spacing 계산 (기존) ...
// 최종 clip:
if !header_inside {
    let header_bottom = layout.body_area.y;  // body_area.y = header_area 끝
    let new_top = by.max(header_bottom);
    bh -= new_top - by;
    by = new_top;
}
if !footer_inside {
    let footer_top = layout.body_area.y + layout.body_area.height; // body_area 하단 = footer_area 시작
    let new_bottom = (by + bh).min(footer_top);
    bh = new_bottom - by;
}
```

**장점**:
- HWP5 spec 표 136 정확 구현
- HWPX `headerInside` / `footerInside` 의미 정합 (한컴 권위 자료)
- 모든 paper-based sample (bit1=0, bit2=0) 의 외곽선이 머/꼬 영역 진입 차단 — 한컴 정합

**단점 / 회귀 risk**:
- Body-based + bit 1/2 = 0 케이스 (HWP3 sample16): body_area 가 이미 머/꼬 영역 제외 → clip 적용해도 변동 없음 (안전)
- Paper-based + bit 1/2 = 0 케이스 (모든 attr=0x01 sample): 외곽선이 작아짐. **시험지 등 spacing=0 sample 의 한컴 실제 모양 확인 필요**
- 시험지 외곽선이 한컴에서 어떤 모양인지에 따라 회귀 가능성

**시험지 회귀 가능성 분석**:
- spacing=0 + paper-based + footer_inside=0 → border bottom = footer_area.y
- 시험지의 한컴 실제 외곽선이 paper 전체를 둘러싸는 모양이면 → 회귀
- 시험지의 한컴 실제 외곽선이 body 영역만 둘러싸는 모양이면 → 정합 (현재 회귀 해소)
- → Stage 4 sweep + 작업지시자 시각 판정 필요

### 후보 B — bit 1/2 처리 + spacing>0 한정 (방어적)

```rust
if !header_inside && spacing_top > 0 { /* clip top */ }
if !footer_inside && spacing_bottom > 0 { /* clip bottom */ }
```

**장점**:
- 시험지 (spacing=0) 회귀 차단
- 본 issue (sample16-hwp5 spacing=1420) 해소

**단점**:
- spec 에 spacing>0 조건 명시 없음 — 정합도 떨어짐
- 한컴 동작과의 추가 검증 필요

### 후보 C — paper_based 반전 (#920 회귀 history)

```rust
let paper_based = (pbf.attr & 0x01) == 0;  // 반전
```

**장점**: 변경 1줄
**단점**:
- Task #952 / #987 의 sample16 정답지 결정 history 무효화
- HWP5 spec 명시적 위배

→ **기각**.

### 후보 D — HWP5 변환본 식별 분기

`file_version` 또는 파일 generator metadata 로 변환본 식별 후 분기.

**단점**:
- 식별 신호 정확도 불확실
- 일반 HWP5 (attr=0x01) 와 변환본 구분 어려움
- spec 무시한 우회

→ **기각**.

## 3. 선정 — 후보 A

### 3-1. 선정 이유

1. **Spec 정합**: HWP5 spec 표 136 의 bit 1 (머리말 포함) / bit 2 (꼬리말 포함) 의미 그대로 구현
2. **HWPX 권위 자료 정합**: `headerInside` / `footerInside` 의미 일치
3. **원리적 회귀 안전성**: body-based 케이스는 변동 없음 (body_area 자체가 이미 clip 효과)
4. **격차 해소의 본질**: rhwp 의 spec 미구현 (bit 1/2 무시) 자체를 정정 — 우회/방어적 분기 없음
5. **시험지 회귀 가능성은 Stage 4 sweep + 시각 판정으로 판단**:
   - 회귀 발견 시 후보 B (spacing>0 한정) 로 후퇴
   - 회귀 없으면 후보 A 그대로 유지 (한컴 정합 최선)

### 3-2. 변경 범위 (예상)

`src/renderer/layout.rs:986-1020` `build_page_borders`:
- 변경 줄 수: ~15 줄 추가 (header_inside/footer_inside 변수 + clip 로직)
- 검증 가능 단위: SVG 출력 시 외곽선 사각형 좌표 비교

### 3-3. RHWP_DEBUG_PAGE_BORDER 확장

기존 debug 출력에 bit 1/2 + clip 적용 후 좌표 추가 (Task #987 RHWP_DEBUG_PAGE_BORDER 영구화 정책 정합).

## 4. 단위 검증 시나리오

### 4-1. 격차 A 정합 확인
```bash
rhwp export-svg samples/hwp3-sample16-hwp5.hwp -p 15 -o /tmp/diag_after/
# SVG 의 외곽선 <rect> 좌표 확인: 하단 y < page number y (외곽선 밖)
```

### 4-2. HWP3 원본 비회귀 확인
```bash
rhwp export-svg samples/hwp3-sample16.hwp -p 15 -o /tmp/diag_after/
# 외곽선 좌표 변동 없음 확인
```

### 4-3. 시험지 회귀 확인 (위험 케이스)
```bash
rhwp export-svg samples/exam_kor.hwp -p 0 -o /tmp/diag_after/
rhwp export-svg samples/exam_math.hwp -p 0 -o /tmp/diag_after/
rhwp export-svg samples/exam_eng.hwp -p 0 -o /tmp/diag_after/
# 외곽선 위치 변동 + 작업지시자 시각 판정 (한컴/PDF 정답지 대조)
```

### 4-4. aift / biz_plan / 통합재정통계 회귀 확인
```bash
for f in aift biz_plan; do
    rhwp export-svg samples/$f.hwp -p 0
done
```

## 5. Stage 3 진입 계획

후보 A 구현:
1. `layout.rs:986-1020` clip 로직 추가
2. `RHWP_DEBUG_PAGE_BORDER` 확장 (bit 1/2 + clipped 좌표 출력)
3. 단위 검증 (4-1, 4-2)
4. Stage 3 보고서: `mydocs/working/task_m100_1001_stage3.md`

Stage 4 진입 시 4-3, 4-4 회귀 검증.
