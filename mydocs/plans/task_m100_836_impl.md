# Task #836 구현 계획서

**선행 문서**: [task_m100_836.md](task_m100_836.md) (수행계획서, 승인 완료)
**브랜치**: `local/task836` (base: `local/task825_826`)

## 개요

5단계. 본질이 미확정이므로 **Stage 1 (사전조사 + RED) 비중이 큼**. Stage 1 결과로 Stage 2 의 구체 수정 위치 확정.

## 단계별 상세

### 단계 1 — 사전조사 + RED 테스트

**목적**: 본질 결함 위치/카테고리 식별 + 회귀 테스트 작성.

**1-1. fixture 추가** (4 sample + 4 PDF)
```bash
git add samples/3-09월_교육_통합_2022.hwp samples/3-09월_교육_통합_2023.hwp \
        samples/3-10월_교육_통합_2022.hwp samples/3-11월_실전_통합_2022.hwp \
        pdf/3-09월_교육_통합_2022-2022.pdf pdf/3-09월_교육_통합_2023-2022.pdf \
        pdf/3-10월_교육_통합_2022-2022.pdf pdf/3-11월_실전_통합_2022-2022.pdf
```

**1-2. 진단 helper 작성** — `examples/diag_836.rs` (또는 dump-pages 확장):
- `cc.items` 별 누적 (현재 typeset used vs IR vpos 진행)
- 카테고리: FullParagraph (lh+ls 합산) / PartialParagraph / Shape (수식 height) / Table / 기타
- per-item 의 `measured_height` vs `ir_height_consumed` (다음 item vpos - 현재 vpos)
- 격차 큰 item 식별 (예: 측정 0px / IR 50px → 본질 후보)

**1-3. 4 sample 교차 진단**:
- 4 sample 모두 동일 카테고리에서 차이 발생하는지 확인
- 가장 단순한 sample (페이지 적은 것) 우선 분석

**1-4. 회귀 테스트 작성** — `tests/issue_836.rs`:
- per-page column `cc.used_height` vs `compute_hwp_used_height` 차이 ≤ 허용 오차 (예: ±10px)
- 4 sample 모든 page 검증
- **현재 상태에서 FAIL** (RED 의도)

**1-5. 본질 가설 확정** — Stage 1 보고서에 가설 + 근거 명시

**커밋**: `Task #836 Stage 1 (RED): 4 sample fixture + 진단 helper + 회귀 테스트 + 본질 가설`

---

### 단계 2 — GREEN (Stage 1 결과 적용)

**목적**: Stage 1 가설로 식별된 본질 결함 정정.

**작업** (Stage 1 결과로 결정 — 아래는 후보):

**후보 A**: Shape (수식) 의 인라인 측정 결함
- `paragraph_layout.rs` 의 TAC 객체 height 처리 확인

**후보 B**: 빈 paragraph height 측정 결함
- `height_measurer.rs` 의 빈 line_segs 기본 높이 (현재 400 HU = 5.3px) 확인 — IR 의 빈 paragraph vpos 진행과 비교

**후보 C**: 미주/footnote 의 column height 영향
- `typeset.rs` 의 footnote 영역 처리 확인

**후보 D**: 머리말 표 height → body_area.y 영향
- `page_layout.rs` 의 body_area 계산이 머리말 height 와 분리 정합 확인

**후보 E**: 문단 spacing_before/spacing_after 처리
- 첫 paragraph 의 spacing_before 적용 여부

**커밋**: `Task #836 Stage 2 (GREEN): {본질 명} 정정 + 4 sample 회귀 테스트 PASS`

---

### 단계 3 — 회귀 검증

**목적**: 광범위 sweep + 기존 테스트 회귀 부재.

**작업**:
1. `cargo test --release` 전체 통과
2. `cargo clippy --release -- -D warnings` clean
3. `cargo test --test issue_836` 4 sample × 모든 page PASS
4. **광범위 sweep**: 보유 7 fixture (또는 `tests/golden_svg/` 의 fixture 들) × 170 page SVG 회귀 — diff 0 또는 정합 변경만
5. WASM 재빌드 (필요 시)

**커밋**: `Task #836 Stage 3 (회귀): cargo test + clippy + sweep 회귀 부재`

---

### 단계 4 — 시각 검증 (작업지시자)

**작업**:
1. WASM 재빌드 + Vite hot-reload
2. 작업지시자 시각 판정 요청:
   - 4 sample 각 1~9 페이지 → 한컴오피스 PDF (`pdf/3-XX월_*-2022.pdf`) 시각 정합
   - column 하단 공백 해소 확인
   - 머리말 banner 형식 정합
   - 페이지 분할 정합 (페이지 수 한컴오피스 viewer 와 일치)
3. 시각 판정 통과 후 Stage 5 진입

**커밋**: `Task #836 Stage 4 (시각): 작업지시자 시각 판정 통과`

---

### 단계 5 — 최종 보고서 + PR

**작업**:
1. `mydocs/report/task_m100_836_report.md` 최종 보고서
2. `mydocs/orders/20260511.md` (또는 진행 시점 날짜) Task #836 entry 추가
3. 커밋: `Task #836 Stage 5 (최종): 보고서 + closes #836`
4. `git push origin local/task836`
5. PR 생성: base=devel, head=jangster77:local/task836, `closes #836`

**산출물**: `_report.md`, orders 갱신, PR

---

## 단계별 commit 계획 요약

| 단계 | commit 메시지 | 변경 파일 |
|---|---|---|
| 1 | `Task #836 Stage 1 (RED): 4 sample fixture + 진단 helper + 회귀 테스트 + 본질 가설` | samples/, pdf/, tests/issue_836.rs, examples/diag_836.rs (또는 dump-pages 확장), `_stage1.md` |
| 2 | `Task #836 Stage 2 (GREEN): {본질 명} 정정 + 4 sample 회귀 테스트 PASS` | src/renderer/... (Stage 1 결과로 결정), `_stage2.md` |
| 3 | `Task #836 Stage 3 (회귀): cargo test + clippy + sweep 회귀 부재` | `_stage3.md` |
| 4 | `Task #836 Stage 4 (시각): 작업지시자 시각 판정 통과` | (코드 변경 가능 시 commit, 없으면 stage 4 보고서만) |
| 5 | `Task #836 Stage 5 (최종): 보고서 + closes #836` | `_report.md`, orders, body 의 `closes #836` |

## 위험 / 가정

- **위험**: Stage 1 진단으로 본질이 너무 깊은 경우 (예: pagination 엔진 전체 재설계 필요) Stage 2 범위 재평가 (별도 task 분리 가능)
- **위험**: 4 sample 의 본질이 서로 다를 경우 — Stage 1 에서 4 sample 모두 진단 후 공통/차이 분리
- **가정**: PR #832 의 picture render 변경 (`header_footer_ref` 등) 이 본 task 에 회귀 영향 없음
- **가정**: 4 sample 의 한컴오피스 PDF 가 정확한 권위 자료 (한글 2022 인쇄본)

## fixture 크기 검토

- HWP 파일 4개: 각 ~수백 KB 예상 (시험지)
- PDF 파일 4개: 각 ~수 MB 예상 (인쇄본)
- 합계 ~10~30MB 추정 — `.gitattributes` LFS 임계 50MB 미만 (정상)
