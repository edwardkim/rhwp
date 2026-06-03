# Task #352 Stage 1 완료보고서: 원인 확정

> 2026-04-28 | Branch: `local/task352`

---

## 절차

1. `local/task352` 브랜치 생성
2. `paragraph_layout.rs:996` 직전과 `text_measurement.rs:209` 직전에 임시 `eprintln!` 삽입 (`RHWP_DEBUG_352` 환경변수 게이트)
3. `cargo build --release`
4. `RHWP_DEBUG_352=1 ./target/release/rhwp export-svg samples/exam_eng.hwp -p 4 -o /tmp/p5_352/ 2> /tmp/issue_352_log.txt`
5. 추가로 전 페이지 export 하여 dash 사용 통계 수집
6. 코드포인트·메트릭 검증 후 임시 로그 모두 revert
7. `git diff --stat src/` 빈 결과 확인 → 클린 빌드 통과
8. 보고서 작성

---

## 핵심 발견

| 항목 | 값 |
|------|-----|
| 문제 라인 | s0p221 **L6** (이슈 본문의 L10 은 인접 라인. L6 가 실제 블랭크 라인) |
| 라인 코드포인트 | `[45 × 29, 32, "of being …"]` (ASCII U+002D 29 개) |
| `interior_spaces` | **3** (≠ 0) → **Branch A** 발동 |
| `total_text_width` | 561.00 px |
| `available_width` | 408.21 px |
| 자연 폭 초과 | **153 px** (이미 자연 폭이 사용 가능 폭 초과) |
| dash advance (HY신명조) | **12.747 px** (embedded 메트릭 직반환) |
| dash advance (Times New Roman) | 5.093 px |
| 메트릭 위치 | `font_metrics_data.rs:3848` `FONT_276_LATIN_0[13] = 853` (= 853/1024 em) |

전 문서 dash 통계 (export-svg 전 페이지): HY신명조 1017 개, Times New Roman 295 개.

---

## 가설 판정

| 가설 | 결과 |
|------|------|
| H1 (Justify Branch B) | **기각** |
| H2 (메트릭 None 폴백) | **기각** |
| H3 (메트릭 자체가 큼) | **확정** |

---

## Stage 2 설계 (구현계획서 §2-1 보강)

채택안: **3 개 이상 연속 dash 시퀀스에 한해** 좁은 advance(`font_size × 0.3` 1차) 적용.

근거: 메트릭 직접 수정 시 1017 개 정상 dash 회귀 우려. leader 패턴(반복) 만 식별하여 영향 범위 최소화.

---

## 산출물

- [x] `mydocs/troubleshootings/issue_352_root_cause.md`
- [x] `mydocs/working/task_m100_352_stage1.md` (본 문서)
- [x] 임시 `eprintln!` revert 완료 (`git diff --stat src/` 빈 결과)
- [x] `cargo build --release` 클린 빌드

---

## 다음 단계

Stage 2 — `text_measurement.rs` 에 leader-aware dash 좁은 advance 도입 후 PDF 비교 + cargo test 회귀.
