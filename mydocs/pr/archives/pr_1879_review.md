# PR #1879 처리 보고서 — HWPX→HWP roundtrip 페이지네이션 fidelity 하니스 (도구 전용)

- PR: https://github.com/edwardkim/rhwp/pull/1879
- 제목: `tools: HWPX→HWP roundtrip 페이지네이션 fidelity 하니스 추가`
- 작성자: planet6897
- base ← head: `devel` ← `tools/roundtrip-fidelity-harness`
- 처리일: 2026-07-04

## 1. 처리 결정

**admin merge.** `hwpx-roundtrip`(HWPX→HWPX 뼈대 보존)과 상보적으로 **HWPX→HWP 변환 후
페이지네이션 일치**를 측정하는 하니스 + 두-바이너리 전이 분류 도구. **Rust 변경 0**(Python 2
+ 매뉴얼 1), 충돌 0, CI pass, 로컬 실전 스모크 통과. #1684(한글 PDF baseline 인프라) 동형 패턴.

## 2. 변경 범위 (3 files +297/-0, Rust 0 — 주장 검증 완료)

| 파일 | 내용 |
|---|---|
| `tools/roundtrip_fidelity_harness.py` (+152) | HWPX→`rhwp convert`→HWP 후 양쪽 `dump-pages`의 `(sec,pi)→첫 페이지` 대조. SAME/PI_MOVED/PAGE_DELTA/ERR 판정, TSV 산출, `PI_MOVED+PAGE_DELTA>0 → exit 1` |
| `tools/roundtrip_fidelity_diff.py` (+82) | 기준 vs 후보 바이너리 TSV 전이 분류(IMPROVED/REGRESSED/STILL_*), `REGRESSED>0 → exit 1` |
| `mydocs/manual/verification/roundtrip_fidelity_harness.md` (+63) | 사용 가이드 + oracle 아님 명시 |

## 3. 검증 (로컬, Linux — pyhwpx 불필요라 직접 실행 가능)

| 항목 | 결과 |
|---|---|
| GitHub CI | 전부 pass |
| 충돌 / Rust 변경 | 0건 / 0건 |
| `py_compile` 2개 | OK |
| **실전 스모크(하니스)** | 실샘플 3건(rowbreak-problem-pages, pr-1674, issue1853) → **SAME=3** (pr-1674 35=35쪽 등), TSV 정상, exit 0 |
| **실전 스모크(diff)** | self-compare → STILL_SAME=3, exit 0 (PR 주장 일치) |
| 자기검열 | 통과 (하드코딩 로컬경로/과장 표현 없음) |

## 4. 평가

- **스코프 정직**: "self-consistency proxy 이며 한컴 편집기/PDF 대비 시각 정합(oracle)이
  아니다"를 본문·매뉴얼 양쪽에 명시 — `feedback_self_verification_not_hancom` 룰을 도구
  설계 단계에서 스스로 반영.
- **기존 게이트와의 관계 명확**: hwpx-roundtrip(직렬화 뼈대) ↔ 본 하니스(변환 후 페이지네이션)
  상보 구도를 매뉴얼에 정리. HWPX↔HWP5 파서/typeset divergence(빈-앵커 host_line_spacing
  소스-의존 계열)를 정량 감시하는 자산.
- **Linux 동작**: pyhwpx/한컴 불필요(순수 rhwp 바이너리 기반) — 메인테이너·컨트리뷰터 모두
  로컬/CI에서 실행 가능. exit code 계약으로 게이트화 여지.

## 5. 산출물

- 본 처리 보고서: `mydocs/pr/pr_1879_review.md`
