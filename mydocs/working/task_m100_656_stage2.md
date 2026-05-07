# Task #656 Stage 2: typeset/layout advance 모델 통일 시도 — 회귀 발생, 후퇴 보고서

## 작업 영역

구현계획서 Stage 2 영역 — typeset 단단/다단 advance 분기 제거 + layout 본문 단락 마지막 줄 trail_ls 제외.

## 변경 영역 (적용 후 후퇴)

### 1. typeset.rs (3 영역)

| 위치 | 변경 |
|------|------|
| line 991 | `if col_count > 1 { height_for_fit } else { total_height }` → `height_for_fit` |
| line 1025 | 동일 |
| line 1041 | 동일 |

### 2. layout/paragraph_layout.rs (1 영역)

| 위치 | 변경 |
|------|------|
| line 2640-2654 | `is_cell_last_line` → 본문 단락 (cell_ctx.is_none()) 마지막 줄도 trail_ls 제외 |

## 회귀 검증 결과

### 광범위 회귀 영역 비교

| 샘플 | 베이스 페이지 | Stage 2 페이지 | 베이스 OVERFLOW | Stage 2 OVERFLOW | 평가 |
|------|--------------|----------------|-----------------|------------------|------|
| synam-001.hwp (35p) | 35 | 35 | 25 | 20 | 개선 (-5건) |
| kps-ai.hwp (80p) | 80 | 78 | 12 | 62 | **회귀 (+50건, -2페이지)** |
| k-water-rfp.hwp (27p) | 27 | 26 | 3 | 44 | **회귀 (+41건, -1페이지)** |
| exam_eng.hwp p8 | 8 | 8 | 1 | 0 | 개선 (-1건) |

**총 회귀**: kps-ai +50건, k-water-rfp +41건. 페이지 감소는 typeset 이 더 잘 들어간다고 판단하여 한 페이지에 욱여넣은 영역.

### 분리 진단 (typeset-only 영역)

layout 변경 원복 후 typeset 만 변경:

| 샘플 | typeset-only OVERFLOW | Stage 2 양쪽 OVERFLOW |
|------|----------------------|----------------------|
| k-water-rfp full | 55 | 44 |

→ layout 변경이 일부 회귀 (~11건) 흡수. 그러나 typeset 변경 자체가 본질 회귀 origin.

## 회귀 본질 정밀 분석

### 회귀 영역 정황

```
LAYOUT_OVERFLOW: page=0, col=0, para=32, type=FullParagraph, y=1054.4, bottom=1028.9, overflow=25.5px
LAYOUT_OVERFLOW: page=0, col=0, para=33, type=FullParagraph, y=1185.6, bottom=1028.9, overflow=156.7px
```

본문 단락 (cell_ctx.is_none()) 의 layout y 진행이 typeset 추정보다 멀리 진행. typeset 이 fit 판정 → 한 페이지에 배치 → layout 에서 본문 영역 초과.

### 표면적 모델 비교 (정합 가설)

매 본문 단락당 advance:

| 영역 | Stage 2 후 advance |
|------|-------------------|
| typeset 단단 | `height_for_fit = total - trail_ls = sb + sum(lh+ls) - trail_ls + sa` |
| layout 본문 (col_top 아닌 경우) | `sb + sum(lh+ls)[n-1 줄] + lh[n번째 줄] + sa` = `sb + sum(lh+ls) - trail_ls + sa` |

**표면 모델 동일**. 그러나 회귀 발생 → 다른 영역 비대칭 존재.

### 잠재 비대칭 영역 (Stage 2 변경 영향 외)

본 stage 변경 외 영역에서 typeset/layout 어긋남 발생 가능:

1. **typeset 의 표 (typeset_table) advance** — 표 host paragraph spacing_before/after 영역
2. **typeset 의 PartialTable 영역** — row 단위 누적 모델
3. **typeset 의 atomic_tac (Picture/Shape)** — line 1023 의 `+= height_for_fit` 변경 의존
4. **typeset 의 partial split (line 1094-1096)** — `part_height = sb + line_advances_sum + part_sp_after` 가 모든 줄 lh+ls 포함, layout 의 partial 끝줄 trail_ls 제외와 비대칭
5. **layout 의 vpos correction (layout.rs:1474-1505)** — `prev_vpos_end = vpos + lh + ls` 의 trail_ls 처리
6. **typeset 의 안전 마진 (LAYOUT_DRIFT_SAFETY_PX = 10.0)** — 본 stage 후 정합 향상 시 마진 영역 조정 필요
7. **layout 의 footnote/footer 영역** — 본문 advance 외 영역 추가 진행

### 본질 결론

**Stage 2 의 단순한 단단/다단 통일 + 본문 마지막 줄 trail_ls 제외만으로는 정합 불가**. typeset/layout 의 advance 어긋남은 본문 단락 advance 영역 외 다수 영역의 누적 영역에 분산됨.

전 영역 통일은:
- typeset 의 표/PartialTable/atomic_tac/partial split 영역 모두 동일 모델로 통일
- layout 의 partial/vpos correction/footnote 영역 모두 동일 모델로 통일
- typeset 의 LAYOUT_DRIFT_SAFETY_PX 영역 재조정

→ **광범위한 영역 통일 작업 필수**. 본 stage 의 4 영역 변경만으로는 부족.

## 후퇴 영역

```bash
git checkout -- src/renderer/typeset.rs src/renderer/layout/paragraph_layout.rs
```

베이스 영역 회복 입증:
- k-water-rfp full OVERFLOW = 3 건 (베이스), 27 페이지

## 본 stage 의 영역 본질

1. **분할 표 영역 (synam-001 p15) 회귀 0** — `LAYOUT_OVERFLOW=0` 입증. 본 타스크 본질 영역의 유효성 확인.
2. **다단 영역 (exam_eng p8) 정합** — `LAYOUT_OVERFLOW 1→0`. 다단 영역 정합 본질 입증.
3. **단단 영역 회귀 (kps-ai, k-water-rfp)** — 단단 본문 단락 advance 모델 통일이 다른 영역 비대칭 노출.

## 다음 진입 영역 결정 요청 (작업지시자)

3 가지 후보:

### 후보 A: 영역 외면 (본 타스크 종결)

Task #485 의 epsilon 영역 그대로 유지. 본 타스크 은 "구조 부채" 본질 영역인 데 시각 결함 부재이므로 외면 가능.

- Issue #656: 우선도 낮음으로 close (or 영구 보존)
- Task #485 PR (origin/pr/task-485) 별도 처리

### 후보 B: 영역 축소 (분할 표 영역만)

`compute_cell_line_ranges` (table_layout.rs) 의 break 조건만 본 타스크 영역으로 통일:
- typeset 의 `split_end_content_limit = avail_content` 영역과 layout 의 line_h 누적 정합
- 본문 단락 advance 모델 영역은 외면
- Task #485 의 epsilon 보다 더 본질적인 정정 가능

→ 본 타스크 본질 (분할 표 영역) 의 부분 진입.

### 후보 C: 광범위 영역 통일 (본 stage 영역 확대)

본 stage 의 4 영역 외 추가 영역 통일:
- typeset 의 표/PartialTable/atomic_tac/partial split 영역
- layout 의 partial/vpos correction 영역
- LAYOUT_DRIFT_SAFETY_PX 영역 재조정

→ 광범위 작업, 회귀 영역 다수 발생 가능. 단계 영역 6+ 확장 필요.

## 권장 영역

**후보 B 권장** — 본 타스크의 본질 (분할 표 영역) 만 정밀 정정. 본문 단락 advance 영역의 광범위 통일 (후보 C) 은 별도 타스크로 분리. 후보 A 는 Task #485 의 epsilon 휴리스틱 영역을 그대로 유지하는 영역.

본 stage 의 발견 영역 (분할 표 영역의 자연 해소 — synam-001 p15 OVERFLOW 1→0) 을 후보 B 의 베이스로 활용 가능.

## 자동 회귀 베이스 (후퇴 후)

```
cargo build --release  # 통과
```

(테스트는 변경 후 재실행 필요 — 본 단계는 후퇴 후 베이스 회복 점검 후 stage2 보고서 작성 영역까지)
