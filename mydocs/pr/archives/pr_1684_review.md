# PR #1684 처리 보고서 — 한글 PDF baseline 인프라 + 별표4 잔여 over 분석 (#1658 후속)

- PR: https://github.com/edwardkim/rhwp/pull/1684
- 제목: `한글 PDF baseline 인프라 + 별표4 잔여 over 분석 (#1658 후속)`
- 작성자: planet6897 (collaborator, 30건/18 merged)
- 연결: #1658 (존속 — close 아님), PR #1670 후속
- base ← head: `devel` ← `planet6897:task/1658-pdf-baseline-infra`
- 처리일: 2026-06-30

## 1. 처리 결정

**admin merge.** PR #1670(법령 표 행분할 over 수정, merged) 후속으로 페이지네이션 측정 인프라
보강 + 별표4 잔여 over 성격 규명. **Rust 소스 변경 0** (분석 + Python 도구 + 문서). 충돌 0 +
CI 전부 pass + py 구문 OK + 자기검열 통과.

## 2. 변경 범위 (merge-base=1d8c2577 기준 5 files +349/-56, src 0)

| 파일 | 내용 |
|---|---|
| `tools/hangul_pdf_baseline.py` | +153 — pyhwpx `save_as(PDF)` + PyMuPDF(fitz) 로 한글 텍스트 줄높이 baseline 추출(pt→px, 다단 클러스터링), `rhwp export-svg` baseline 과 pitch 대조(`--compare`), PageCount 가드 |
| `mydocs/tech/investigations/issue-1658/task_m100_1658_capacity.md` | +52 — capacity 결손 분석 |
| `mydocs/tech/investigations/issue-1658/task_m100_1658_clipping_capacity_unified.md` | +36 — 클리핑≡capacity 통합 |
| `mydocs/manual/verification/hangul_pdf_baseline.md` | +54 — 도구 매뉴얼 |
| `mydocs/report/task_m100_1658_report.md` | 라운드 1 최종 보고서(±) |

## 3. 검토

- **"Rust 소스 변경 없음" 주장 정확** — src/.rs 0건. 빌드/테스트 영향 없음.
- **Python 도구**: `python3 -m py_compile` OK. 의존성 `pyhwpx`/`PyMuPDF`(Windows + 한컴)는
  컨트리뷰터 로컬 — CI 미설치 정상(대규모 한글 권위 게이트의 합리적 방식, `render_page_gate.py`
  선례 동형). `C:/Users/planet/...` 는 도구 인자 기본값.
- **자기검열 통과**: 외부 공개 부적합(최상급/공식/대량수집) 표현 없음. 비공개 실문서 미커밋.
- **#1658 존속 명시**: report 가 "라운드 1 완결 + #1658 존속"으로, 미해결 이슈 close 아님
  (`feedback_no_close_without_approval` 정합).

## 4. 데이터 기반 가설 정정 (핵심 가치)

- **줄높이 fidelity 정상**: byeolpyo1(권위 PDF 4쪽 = 편집기 4쪽) 한글 줄 pitch **28.80 = rhwp
  28.80** → drift 없음. → #1658 잔여(별표4 Δ+3, 클리핑)는 **줄높이 fidelity 가 아니라 cut 로직 /
  한글 PDF 배율 아티팩트**. 앞선 가설을 데이터로 정정.
- **클리핑 ≡ capacity**: 별표4 23.5px 클리핑(render>cut)과 capacity 결손(rhwp<한글)은 같은
  적재량 차이의 양면 → 수정 방향 상충, 단일 조정 불가.
- **차단 기록**: 한글 행높이 COM(CellShape/TableLowerCell 미동작), PDF 배율 100% 강제 실패
  (save_as 가 PrintMethod 무시) → 후속 진척의 선결 조건 명시.

## 5. 검증

| 항목 | 결과 |
|---|---|
| GitHub CI (Build&Test/CodeQL/Analyze/Canvas) | 전부 pass |
| 충돌 | 0건 |
| src/.rs 변경 | 0건 (분석·도구 PR) |
| `hangul_pdf_baseline.py` 구문 | py_compile OK |
| 자기검열 | 통과 |

## 6. 의의

페이지네이션 측정에 **한글 권위 줄높이 baseline**(PDF/COM)을 도입해, #1658 잔여 over 의 성격을
"줄높이 drift" 에서 "cut 로직/배율 아티팩트"로 재규명. 후속 라운드는 배율 100% PDF / COM 표
API 선결이 전제임을 명확히 했다. #1670 회귀 자산과 함께 후속 정합 판정에 재사용된다.

## 7. 산출물

- 본 처리 보고서: `mydocs/pr/pr_1684_review.md`
