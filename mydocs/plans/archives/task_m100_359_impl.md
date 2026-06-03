# Task #359 구현계획서

상위: [task_m100_359.md](task_m100_359.md)
브랜치: `local/task359` (← origin/devel)
이슈: [#359](https://github.com/edwardkim/rhwp/issues/359)

## 사전 상태

- `local/task359` 브랜치 origin/devel(`ce41fce`) 기준 생성 완료
- 트러블슈팅 사전 검색 완료 (typeset_layout_drift_analysis.md, multi_tac_table_pagination.md, hwpx_lineseg_reflow_trap.md, line_spacing_lineseg_sync.md 등 관련 자료 확인)
- 디버그 도구 `RHWP_TYPESET_DRIFT=1` (Task #321 시점 도입) 존재 확인 — Stage 1 에서 활용

## 핵심 분석 단서

### 증상 정량 (이슈 #359)

| 단계 | 값 |
|------|----|
| body_area.y / h | 113.3 / 915.5 |
| col_bottom (절대) | 1028.9 |
| pagination 의 used (페이지 1) | 728.1 |
| layout 의 마지막 y (pi=35) | 1316.8 |
| **드리프트 (절대 좌표 기준)** | 1316.8 − 113.3 = 1203.5 (used 728.1 보다 +475.4 over) |

페이지 1 items 33 (pi=0~31 + pi=31 ci=0 Shape). pi=34/35 가 layout 에서 페이지 1 에 잔존하여 col_bottom 초과. dump-pages 결과는 페이지 1 에 33 items 만 보고하지만 stderr 의 LAYOUT_OVERFLOW 는 pi=34/35 가 페이지 1 col 0 에 있음을 명시.

→ **두 보고가 모순** = pagination 결과의 PageItem 목록과 layout 단계의 실제 처리 항목이 다름. 핵심 질문:
1. layout 이 pi=34/35 를 어디서 가져왔는가? (pagination 의 PageItem 에 없는데 layout 이 처리)
2. 또는 dump-pages 가 33 items 만 보고하는데 layout 은 35 items 를 처리하는 path 가 따로 있는가?

## 작업 절차

---

## Stage 1 — 드리프트 origin 정량화

### 목표
페이지 1 의 layout 이 어떤 PageItem 들을 처리하는지 + 각 항목의 fit 산정 vs y 진행 비교 정량화.

### 작업

#### 1.1 RHWP_TYPESET_DRIFT 진단 훅 확인 + 활성화

```bash
RHWP_TYPESET_DRIFT=1 ./target/release/rhwp export-svg samples/k-water-rfp.hwp -p 0 -o output/debug/task359_p1/ 2>&1 | grep TYPESET_DRIFT | head -50
```

기대 출력: 페이지 1 각 문단의 `pi`, `cur_h`, `avail`, `fmt_total`, `vpos_h`, `diff` 정량 표.

#### 1.2 dump-pages 의 pi=34, pi=35 위치 확인

```bash
./target/release/rhwp dump-pages samples/k-water-rfp.hwp -p 0 2>&1 | grep -E "pi=3[3-9]|pi=4[0-5]"
./target/release/rhwp dump-pages samples/k-water-rfp.hwp -p 1 2>&1 | head -10
```

목표:
- 페이지 1 의 마지막 PageItem 까지 (예상 pi=33 까지)
- 페이지 2 의 시작 PageItem (예상 pi=34 시작)
- pi=34, pi=35 가 dump 보고 페이지와 layout 처리 페이지 사이 모순 여부

#### 1.3 LAYOUT_OVERFLOW 메시지 source 위치 식별

```bash
grep -rn "LAYOUT_OVERFLOW" /home/edward/mygithub/rhwp/src/ | head -10
grep -rn "LAYOUT_OVERFLOW_DRAW" /home/edward/mygithub/rhwp/src/ | head -10
```

→ 어느 함수가 어느 조건에서 보고하는지 확인 → page_index 가 어떻게 결정되는지 추적

#### 1.4 layout entry 와 pagination output 의 paragraph 순회 추적

`src/renderer/layout.rs` 의 main loop 가 `paginated_pages[page_idx].column_contents[col_idx].items` 를 순회하는지, 아니면 `paragraphs` 전체를 순회하면서 별도 page boundary 를 결정하는지 확인.

가능성:
- (P1) layout 이 pagination 결과를 그대로 따르나 vpos 보정 시 추가 항목 잠입
- (P2) layout 이 paragraph 전체를 별도 처리하면서 pagination 결과 무시
- (P3) `wrap_around_paras` 또는 `hidden_empty_paras` 같은 부수 set 을 통한 항목 추가

### 산출물

- `mydocs/working/task_m100_359_stage1.md`:
  - RHWP_TYPESET_DRIFT 출력 표
  - dump-pages 페이지 1/2 boundary
  - LAYOUT_OVERFLOW source 위치
  - layout 의 paragraph 순회 경로 (P1/P2/P3 식별)

### 완료 조건

- pi=34, pi=35 가 layout 에서 페이지 1 에 잔존하는 정확한 경로 식별
- 드리프트 origin 위치 (어느 코드 라인 / 어느 데이터) 확정

---

## Stage 2 — 원인 분석 + 수정 방향 확정

### 목표
Stage 1 의 정량 + 경로 식별 결과로 원인 확정 + 수정 방향 결정.

### 가설별 검증

| 가설 | 검증 방법 | 수정 방향 (해당 시) |
|------|----------|-------------------|
| A. `format_paragraph::total_height` 산정 누락 (line_spacing/spacing_after 등) | RHWP_TYPESET_DRIFT 의 fmt_total vs 실제 h 비교 | total_height 식 보강 |
| B. layout 의 vpos 보정이 pagination current_height 와 다른 기준 | layout 의 y_offset 갱신 위치 추적 | 동일 기준 (vpos 또는 fmt) 통일 |
| C. Shape (TAC tac=true) 의 fit 누적 누락 | pi=31 ci=0 Shape (89.1px) 가 used 에 포함되는지 | typeset_section 에 Shape 처리 추가 |
| D. 빈 문단 (h=16/18.7) 누적 누락 | pi=0~6, pi=10~21 등 빈 문단 22개의 sum vs used | 빈 문단 fit 산정 보강 |
| E. layout 이 pagination 결과 무시 (P2) | layout 의 entry 함수가 paragraphs 직접 순회 | layout 을 PageItem 기반으로 변경 |

### 수정 방향 후보

각 가설별 수정 범위 + 회귀 위험 평가:

- **A**: `src/renderer/typeset.rs::format_paragraph` 만 수정 (작은 범위, 안전)
- **B**: `src/renderer/layout.rs` 의 y_offset 갱신 + typeset 의 fit 산정 동시 점검 (중간 범위)
- **C**: typeset_section 의 Shape 처리 추가 (작은 범위)
- **D**: 빈 문단 fit 산정 보강 (단순, 그러나 다른 케이스 영향 우려)
- **E**: 가장 큰 변경 — typeset/layout 의 entry 점 통일

### 산출물

- `mydocs/working/task_m100_359_stage2.md`:
  - 가설별 검증 결과
  - 원인 확정
  - 수정 방향 + 코드 위치

### 완료 조건

- 단일 또는 복수 origin 확정
- 수정 방향 + 영향 범위 추정

---

## Stage 3 — 코드 수정 + 자동 검증

### 작업

1. Stage 2 확정된 수정 적용
2. `cargo build --release`
3. `cargo test --lib` (1008+ passed 확인)
4. `cargo test --test svg_snapshot` (6/6, 영향 시 UPDATE_GOLDEN 검토)
5. `cargo test --test issue_301` (z-table 가드)
6. `cargo clippy --lib -- -D warnings`
7. `cargo check --target wasm32-unknown-unknown --lib`
8. **회귀 점검** — 7 핵심 샘플 + form-002 + k-water-rfp 페이지 수 + LAYOUT_OVERFLOW 점검:

```bash
for f in 21_언어_기출_편집가능본.hwp exam_math.hwp exam_kor.hwp exam_eng.hwp basic/KTX.hwp aift.hwp biz_plan.hwp k-water-rfp.hwp; do
  ./target/release/rhwp dump-pages samples/$f 2>/dev/null | head -1
done
./target/release/rhwp dump-pages samples/hwpx/form-002.hwpx | head -1

# LAYOUT_OVERFLOW 0 건 확인
./target/release/rhwp export-svg samples/k-water-rfp.hwp -o /tmp/task359_kwater_full/ 2>&1 | grep -c "LAYOUT_OVERFLOW"
```

### 결정 기준

- LAYOUT_OVERFLOW 0 건 + 7 샘플 페이지 수 무변화 → Stage 4 진행
- 다른 샘플 페이지 수 변화 발생 시 → 의도성 판정 (Stage 2 수정 방향 점검)

### 산출물

- 코드 수정 (`src/renderer/typeset.rs` 또는 `src/renderer/layout.rs` 또는 양쪽)
- `mydocs/working/task_m100_359_stage3.md`:
  - 검증 결과 표
  - 페이지 수 변화 (있으면)
  - LAYOUT_OVERFLOW 카운트 (0 기대)

### 완료 조건

- 모든 자동 검증 통과
- LAYOUT_OVERFLOW 0 건 (k-water-rfp 포함)
- 회귀 0 또는 의도된 변경만

---

## Stage 4 — WASM 빌드 + 작업지시자 시각 판정 + 최종 보고서

### 작업

#### 4.1 WASM Docker 빌드

```bash
docker compose --env-file .env.docker run --rm wasm
```

#### 4.2 SVG export (작업지시자 시각 판정용)

```bash
rm -rf output/debug/task359_kwater
./target/release/rhwp export-svg samples/k-water-rfp.hwp -o output/debug/task359_kwater/

# 핵심 페이지: p1 (LAYOUT_OVERFLOW 영역), p2, p3
```

#### 4.3 작업지시자 시각 판정 (PDF 없이)

검증 항목:
- k-water-rfp p1: 페이지 분할 적정 위치, 본문 클램프 없음
- k-water-rfp p2, p3: 정상 흐름 (이전 페이지 잔존물 없음)
- 7 핵심 샘플 + form-002: 회귀 없음

#### 4.4 최종 보고서

`mydocs/report/task_m100_359_report.md`:
- 증상 / 원인 / 수정 / 검증
- 드리프트 정량 Before/After
- 페이지 수 변화 (있으면 의도 명시)
- 시각 검증 결과
- 학습 (typeset/layout 정합 패턴)

#### 4.5 트러블슈팅 등록

`mydocs/troubleshootings/task359_pagination_layout_drift.md`:
- 증상 패턴 (LAYOUT_OVERFLOW + dump-pages 모순)
- 원인 (Stage 2 확정 사항)
- 해결 방법
- 예방책 (RHWP_TYPESET_DRIFT 활용 가이드)

#### 4.6 orders 갱신

`mydocs/orders/20260426.md` 에 Task #359 섹션.

### 산출물

- WASM 빌드 (`pkg/rhwp_bg.wasm`)
- SVG export
- `mydocs/report/task_m100_359_report.md`
- `mydocs/troubleshootings/task359_pagination_layout_drift.md`
- `mydocs/orders/20260426.md` 갱신

### 완료 조건

- 작업지시자 시각 판정 통과
- 모든 산출물 작성

---

## 커밋 스킴

| Stage | 커밋 메시지 |
|-------|------------|
| Stage 1 | `Task #359: Stage 1 - 드리프트 origin 정량화 (RHWP_TYPESET_DRIFT)` |
| Stage 2 | `Task #359: Stage 2 - 원인 분석 + 수정 방향 확정` |
| Stage 3 | `Task #359: Stage 3 - 코드 수정 + 자동 검증` |
| Stage 4 | `Task #359: Stage 4 - WASM + 시각 검증 + 최종 보고서` |

각 stage 완료 시 작업지시자 승인 후 다음 stage 진행.

## 리스크 + 대응

| 리스크 | 대응 |
|--------|------|
| RHWP_TYPESET_DRIFT 가 layout 측 정량 지원 안 함 | layout.rs 에 임시 진단 출력 추가 (env 가드, 작업 후 제거) |
| 드리프트 origin 이 단일 위치가 아닌 누적 (가설 D 등) | Stage 1 의 정량 표로 누적 패턴 식별 — 우선순위 정렬 |
| 수정이 Task #321 vpos-reset 가드 또는 Task #347 좌표 정합과 충돌 | 가설별 검증 시 충돌 확인 후 양립 방안 도출 |
| 7 샘플 회귀 발생 | 의도성 판정 — KTX TOC (Task #279), form-002 (Task #324), exam_eng (Task #347) 영역의 의도된 변경이면 골든 갱신 + 시각 확인 |
| layout 측 paragraph 순회가 paginated_pages 가 아닌 직접 순회 (P2) | 가장 큰 변경. Stage 2 결과에 따라 수행계획서 재검토 가능 |

## 검증 명령 모음

```bash
# Stage 1
RHWP_TYPESET_DRIFT=1 ./target/release/rhwp export-svg samples/k-water-rfp.hwp -p 0 -o /tmp/task359/ 2>&1 | grep TYPESET_DRIFT
./target/release/rhwp dump-pages samples/k-water-rfp.hwp -p 0 | tail -10
./target/release/rhwp dump-pages samples/k-water-rfp.hwp -p 1 | head -10

# Stage 3 회귀
cargo test --lib
cargo test --test svg_snapshot
cargo clippy --lib -- -D warnings
cargo check --target wasm32-unknown-unknown --lib
./target/release/rhwp export-svg samples/k-water-rfp.hwp -o /tmp/task359_kwater/ 2>&1 | grep -c "LAYOUT_OVERFLOW"

# Stage 4
docker compose --env-file .env.docker run --rm wasm
./target/release/rhwp export-svg samples/k-water-rfp.hwp -o output/debug/task359_kwater/
```
