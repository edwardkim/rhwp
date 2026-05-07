# Task #656: typeset/layout height 측정 모델 통일 — 구현 계획서

## 본질 영역 정밀 (수행계획서 후속)

수행계획서 §단계 영역 의 5 단계 영역 중 **Stage 3 (clamp pile → break)** 은 이미 Task #332 Stage 4b (`paragraph_layout.rs:866-870`) 에서 처리 완료:

```
// Task #332 Stage 4b: clamp 제거. 단 하단을 초과하는 줄은 그대로 그린다
// (시각 경계 약간 넘김 허용). 기존의 `text_y = col_bottom - line_height`
// 클램프는 여러 overflow 줄을 같은 y 에 piling 해 글자 겹침을 만들었으나,
// 클램프 없이 원래 y 에 그리면 piling 자체가 발생하지 않는다.
```

→ 본 구현계획서는 **4 단계** 로 재조정.

## 현 코드 영역의 advance 모델 정밀

### typeset (`src/renderer/typeset.rs`)

| 분기 | advance | 영역 |
|------|---------|------|
| 단단 fit (line 991, 1027, 1043) | `total_height` | Task #359 (k-water-rfp p3 311px drift 차단) |
| 다단 fit (line 991, 1027, 1043) | `height_for_fit` | Task #391 (exam_eng 8p 정상 단 채움 복원) |
| fit 판정 (line 981, 961) | `height_for_fit` | Task #359 (마지막 항목 trailing_ls 무의미) |

### layout (`src/renderer/layout/paragraph_layout.rs:2638-2693`)

| 분기 | advance | 영역 |
|------|---------|------|
| 본문 단락 모든 줄 | `line_height + line_spacing` (마지막 줄도 trail_ls 가산) | Task #452 (#332 의 layout-only trailing 제외 회복) |
| 셀 내 마지막 문단 마지막 줄 | `line_height` (trail_ls 제외) | 셀 높이 모델 정합 |
| spacing_after | `+= spacing_after` | 문단 뒤 간격 |

### 본질 어긋남

- typeset 단단 = `total_height` (lh+ls 모든 줄 + spacing_b/a)
- layout 본문 = `lh + ls 모든 줄 + spacing_after` (spacing_before 는 별도 경로?)

→ **typeset 의 spacing_before vs layout 의 spacing_before** 처리 영역도 정밀 점검 필요. Stage 1 진단 영역.

### `compute_cell_line_ranges` (`table_layout.rs:2244`)

분할 표 셀 안의 line range 결정 영역. typeset 의 `split_end_content_limit` 는 `avail_content` 추정 영역. layout 의 line_h 누적과 미세 어긋남 → Task #485 의 epsilon 영역. 본 타스크가 이 어긋남 자체를 제거.

## 4 단계 구현 영역

---

### Stage 1: 본질 정밀 측정 + 회귀 베이스 영역 구축 (소스 변경 0)

**목표**: typeset/layout drift 의 정확한 영역을 정량 측정 + 모든 회귀 검증 샘플의 현재 베이스 영역 수집.

**작업 영역:**

1. **drift 정밀 측정 진단 영역**
   - `RHWP_TYPESET_DRIFT=1 cargo run -- export-svg samples/synam-001.hwp -p 14` → drift trace 수집
   - 마찬가지로 `kps-ai.hwp p56`, `k-water-rfp.hwp p3`, `exam_eng.hwp p7` (다단)
   - drift 영역의 본질: typeset cur_h vs layout y 진행의 차이

2. **분할 표 영역 정밀**
   - synam-001 p15 분할 표 (pi=140) 의 `compute_cell_line_ranges` 정밀 측정
   - typeset 의 `split_end_content_limit` 와 layout 의 line_h 누적 차이 정량
   - 차이의 본질: typeset advance 모델 vs layout advance 모델 어긋남

3. **회귀 베이스 영역 수집**
   - 검증 샘플 출력 SVG 영역 수집 → `output/svg/baseline_656/` 영역
   - 페이지 수, 클립 영역, 빈 페이지 영역 식별

4. **보고서 작성**
   - `mydocs/working/task_m100_656_stage1.md`
   - 항목: drift 정량, 분할 표 영역 어긋남, 회귀 베이스 영역, Stage 2 진입 영역 영향 평가

**커밋 영역:**
```
git commit -m "Task #656 Stage 1: 본질 정밀 측정 + 회귀 베이스 영역 구축"
```

**승인 요청 시점**: Stage 1 완료 시.

---

### Stage 2: typeset advance 통일 (단단/다단 → height_for_fit) + layout advance 통일

**목표**: typeset 의 단단/다단 분기 제거 → 모두 `height_for_fit`. layout 의 본문 단락 마지막 줄 trail_ls 제외.

**작업 영역:**

1. **typeset 영역 변경** (`src/renderer/typeset.rs`)
   - line 991, 1027, 1043 의 `if st.col_count > 1 { fmt.height_for_fit } else { fmt.total_height }` → `fmt.height_for_fit` (3 영역)
   - 단단 케이스 (Task #359 회귀 영역) 회귀 발생 가능 → Stage 1 의 k-water-rfp p3 베이스 영역 비교 필수

2. **layout 영역 변경** (`src/renderer/layout/paragraph_layout.rs:2640-2652`)
   - 본문 단락 마지막 줄도 `y += line_height` (trail_ls 제외)
   - 단 셀 안 마지막 문단 마지막 줄은 기존 영역 유지 (이미 trail_ls 제외)
   - Task #452 의 회귀 영역 (pagination 과 1 ls drift) 재발 가능 → 정밀 점검

3. **회귀 검증 영역**
   - 단위 테스트: `cargo test` 통과 확인
   - 시각 회귀: k-water-rfp p3 (단단 trailing_ls drift), exam_eng 8p (다단), kps-ai p56/p67-73 영역 비교
   - LAYOUT_OVERFLOW 로그 영역 0 인지 확인 (현재 베이스 영역과 비교)

4. **회귀 영역 발생 시 후퇴**
   - typeset 만 변경 vs layout 만 변경 의 단독 진입 영역 점검
   - 양 영역 모두 변경했을 때만 정합되는 영역 (호환 영역) 정밀

5. **보고서 작성**
   - `mydocs/working/task_m100_656_stage2.md`
   - 항목: 변경 영역, 회귀 영역 비교, drift trace 비교 (Stage 1 vs Stage 2)

**커밋 영역:**
```
git commit -m "Task #656 Stage 2: typeset/layout advance 통일 (height_for_fit)"
```

**승인 요청 시점**: Stage 2 완료 + 회귀 0 입증 시.

**위험 영역**: Task #359 (k-water-rfp p3) 의 trailing_ls 누적 drift 가 단단 케이스에서 재발 가능. 이 경우 Stage 2 후퇴 후 Stage 3 (vpos correction) 에서 본질 영역 우회 처리.

---

### Stage 3: vpos correction 정합 (vpos_end 의 trail_ls 처리)

**목표**: `layout.rs:1474-1505` 의 vpos correction 영역에서 `vpos_end` 의 trail_ls 처리 정합. trail_ls 가 advance 모델에서 제외되었으므로 vpos correction 도 정합해야 함.

**작업 영역:**

1. **vpos_end 산정 영역 점검**
   - `prev_vpos_end = seg.vertical_pos + seg.line_height + seg.line_spacing` (line 1447)
   - Stage 2 후 layout advance 가 `lh` 만 누적이라면 vpos_end 도 trail_ls 제외 필요
   - 단 lazy_path (line 1486) 영역 별도 점검

2. **양방향 보정 영역 평가**
   - 분석 문서의 §시도한 fix 와 실패 경로: "양방향 보정 → col 1 의 pi 가 같은 y 로 collapse" 영역
   - 현재 단방향 (`end_y >= y_offset - 1.0`) 이 본질 정합인지, 양방향이 필요한지 정밀
   - Stage 2 의 변경이 단방향만으로 충분히 정합 가능한지 확인

3. **회귀 검증**
   - hwp-multi-001 (force_page_break + vpos-reset 영역)
   - 다단 단 경계 (exam_eng) 의 col 0/col 1 collapse 영역 점검

4. **보고서 작성**
   - `mydocs/working/task_m100_656_stage3.md`

**커밋 영역:**
```
git commit -m "Task #656 Stage 3: vpos correction 정합 (vpos_end trail_ls 처리)"
```

**승인 요청 시점**: Stage 3 완료 + 회귀 0 입증 시.

**Stage 3 외면 가능 영역**: Stage 2 만으로 분할 표 epsilon 영역이 자연 해소되고 vpos correction 영역 회귀 0 시 Stage 3 외면 가능. Stage 1 진단 결과 따라 결정.

---

### Stage 4: epsilon 도입 회피 입증 + 광범위 회귀 검증 + 최종 보고

**목표**: 본 타스크 본질 — Task #485 의 epsilon 휴리스틱이 본 타스크 변경분 만으로 불필요해졌는지 입증.

**작업 영역:**

1. **epsilon 도입 회피 입증**
   - Task #485 의 `compute_cell_line_ranges` epsilon (origin/pr/task-485 의 53effd17 commit) 을 본 베이스에 적용하지 **않은** 상태에서:
   - synam-001 p15 의 셀 마지막 줄 클립 발생 영역 점검 (베이스: 클립 발생, 본 타스크 변경 후: 클립 미발생 입증)
   - synam-001 p20·p21 (Task #485 회귀 영역) 회귀 미발생 입증

2. **광범위 회귀 영역 검증**
   - 주요 분할 표 샘플 영역: aift.hwp, biz_plan.hwp, kps-ai.hwp, synam-001.hwp
   - 일반 페이지네이션 영역: k-water-rfp.hwp, exam_eng.hwp, exam_science.hwp, hwp-multi-001.hwp
   - golden SVG 영역 비교 (`UPDATE_GOLDEN` 미적용 상태에서 회귀 영역 식별)
   - 시각 회귀 0 입증

3. **Task #485 PR 처리 영역**
   - 본 타스크 머지 시 Task #485 의 epsilon 도입 자체가 불요로 자연 해소
   - Task #485 PR (origin/pr/task-485) close 영역 영역 처리 보고 (작업지시자 결정 영역)

4. **최종 보고서 작성**
   - `mydocs/report/task_m100_656_report.md`
   - 항목: Stage 별 변경 영역, 회귀 영역 비교, epsilon 도입 회피 입증, Task #485 영역 처리 결정

5. **오늘할일 갱신**
   - `mydocs/orders/yyyymmdd.md` (작업 종료 시점 날짜)

**커밋 영역:**
```
git commit -m "Task #656 Stage 4: 광범위 회귀 검증 + 최종 보고서 + orders 갱신"
```

**머지 영역:**
- 작업지시자 시각 판정 통과 시 → `local/devel` 머지 → `devel` 머지 → push origin devel
- Issue #656 close 영역
- Task #485 (origin/pr/task-485) PR 처리 영역 (close + 사유 보고)

---

## 검증 영역 (전 Stage 공통)

### 자동 회귀

```
cargo test                          # 단위 테스트 영역
cargo test --release                # 릴리즈 빌드 테스트 영역
```

### 시각 회귀 (각 Stage 마다)

```
rhwp export-svg samples/synam-001.hwp -p 14 -o output/svg/stage{N}/synam-001/
rhwp export-svg samples/kps-ai.hwp -p 55 -o output/svg/stage{N}/kps-ai/
rhwp export-svg samples/k-water-rfp.hwp -p 2 -o output/svg/stage{N}/k-water-rfp/
rhwp export-svg samples/exam_eng.hwp -p 7 -o output/svg/stage{N}/exam_eng/
```

### drift 진단 (Stage 1, 2 영역)

```
RHWP_TYPESET_DRIFT=1 RHWP_TYPESET_DRIFT_LINES=1 \
  rhwp export-svg samples/synam-001.hwp -p 14 2>&1 | tee output/diagnostic/synam-001-p15-stage{N}.log
```

### 분할 표 IR 영역 비교

```
rhwp dump-pages samples/synam-001.hwp -p 14
rhwp dump samples/synam-001.hwp -s {S} -p {P}     # 의심 문단 영역
```

## 위험 영역 정밀 (선행 시도 회귀 영역 회피)

| 회귀 영역 | 발생 가능 Stage | 회피 영역 |
|-----------|----------------|-----------|
| k-water-rfp p3 trailing_ls 누적 drift (Task #359) | Stage 2 (단단 → height_for_fit) | typeset 만 변경 시 layout 과 어긋남 — 양 영역 동시 변경으로 정합 |
| exam_eng 8p 다단 단 채움 (Task #391) | Stage 2 | 다단 advance 는 이미 height_for_fit, 변경 영향 없음 |
| col 1 의 pi 가 같은 y 로 collapse (Task #331 시도) | Stage 3 양방향 보정 | 단방향 보정 유지 + 본질 정합 |
| 빈 페이지 (Task #359 단독 항목 페이지) | Stage 2 | LAYOUT_DRIFT_SAFETY_PX 가드 영역 유지 (skip_safety_margin_once 영역) |
| 분할 표 회귀 (Task #361 PartialTable) | Stage 2 | prev_is_partial_table 가드 영역 유지 |

## 작업지시자 승인 요청 영역 (구현계획서)

1. **단계 영역 4 단계** 적정성
2. **Stage 진입 단위 커밋 영역** — 각 Stage 별 단독 커밋, 회귀 발생 시 단독 revert 가능
3. **Stage 외면 정책** — Stage 3 (vpos correction) 외면 가능 영역, Stage 1/2/4 만으로 본 타스크 본질 영역 달성 가능 시 외면
4. **Task #485 PR 처리 영역** — 본 타스크 완료 후 close 처리 결정

승인 후 Stage 1 진입.
