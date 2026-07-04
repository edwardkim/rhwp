# PR #1881 처리 보고서 — 렌더 결함 3건 (#1230/#1835/#1838) + OVR 게이트 첫 적용

- PR: https://github.com/edwardkim/rhwp/pull/1881
- 제목: `렌더 결함 3건: #1230 Square-wrap 그림 겹침 / #1835 TAC stale 높이 확장 / #1838 글리프 보존 가드`
- 작성자: planet6897
- 연결: closes #1230, closes #1835 (#1838 은 가드만 — 잔여 축 실파일 대기)
- base ← head: `devel` ← `fix/1230-square-wrap-emf-overlap`
- 처리일: 2026-07-04

## 1. 처리 결정 — 시각 판정 통과 → admin merge

**admin merge (작업지시자 시각 판정 통과, 2026-07-04).** 이슈별 클린 커밋 3건 분리(#1573 때
요구한 방식 준수), CI 11 pass, 충돌 0, 오라클 PDF 명명 규약(`pdf/{stem}-2022.pdf`) 준수, 로컬
전 게이트 green + **OVR 개체 회귀 0건** + #1835 before/after/오라클 3자 대조 통과.

## 2. 변경 요지 (9 files +362/-3, 소스 4파일 +100)

| 이슈 | 수정 | 가드 폭 |
|---|---|---|
| #1230 | `picture_flow_frame_size_hu` — 비-TAC + Square/Tight/Through + common>0 만 common 프레임(기존 max(common,current) 과대 해소). **emit/배치 두 경로 동일 적용**(`feedback_fix_scope_check_two_paths` 정합) | #1122 문26(TAC, common 손상) 회귀 케이스 명시 + 유닛 4건 |
| #1835 | `TAC_SHRINK_MAX_OVERFLOW_RATIO=1.5` — 내용이 저장 높이 1.5× 초과 시 축소 대신 확장. 한글 2022 오라클 직접 생성해 확정 | 경미 초과(2~150%) 축소 불변(#672 존중) |
| #1838 | 진단(글리프 탈락 없음 — per-cluster 방출로 부분문자열 검색이 구조적 실패) + 보존 가드 테스트 | 코드 무변경 |

## 3. 검증 (로컬, Linux WSL2)

| 항목 | 결과 |
|---|---|
| GitHub CI | 11 pass / 충돌 0 |
| 신규 테스트 | issue_1835 2 + picture_flow_frame 유닛 4 — 통과 |
| 전체 `cargo test --tests` | **FAILED 0** (2817 passed) |
| fmt / clippy | clean |
| **OVR 개체 회귀 게이트 (--no-hwp, 첫 적용)** | devel(727726e1) baseline vs PR — rowbreak-problem-pages(8개체)·pr-1674(5)·exam_science(13) **총 26개체 회귀 0건** — "blast 없음" 주장 기계 검증 |

## 4. 시각 대조 (#1835, 오라클: pdf/issue1835_tac_stale_height-2022.pdf 한글 2022)

| | 첫 표(4×3 TAC, common.height 1/1.8 훼손) |
|---|---|
| before(devel) | 행 과압축 — 헤더 텍스트가 행 경계와 겹침 ❌ |
| after(PR) | 내용 높이로 확장, 4행 균등 ✅ |
| 오라클(한글 2022) | after 와 정합 (PR 실측 Δ<2pt 주장과 시각 일치) |

자료: `output/poc/task1881/{before,after}_p1.png`, `oracle_p-1.png`.
#1230 은 재현 실샘플 부재(KICE) → 합성 회귀 테스트로 커버, 저장소 샘플 무변동(OVR 확인).

## 5. 평가

- 이슈별 커밋 분리 + 오라클 직접 생성 + 좁은 가드 + 회귀 케이스 명시 — 처리 요구사항을
  모두 반영한 모범적 구성.
- #1838 을 "결함 아님(검증 방법의 함정)"으로 정직하게 진단하고 가드만 추가한 판단도 타당.
- OVR 게이트 첫 적용: cherry-pick/merge 전 무회귀를 개체 geometry 로 기계 증명하는 절차가
  이번부터 표준 단계로 동작함을 확인.

## 6. 산출물

- 본 처리 보고서: `mydocs/pr/pr_1881_review.md`
