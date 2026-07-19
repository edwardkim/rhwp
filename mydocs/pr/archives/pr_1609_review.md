# PR #1609 처리 보고서 — HWPX 3차원 무손실 검증 보고서 + 렌더링 -1쪽 갭 조사

- PR: https://github.com/edwardkim/rhwp/pull/1609
- 제목: `HWPX 3차원 무손실 검증 보고서 + 렌더링 -1쪽 갭 조사 (통제셋·게이트)`
- 작성자: planet6897 (collaborator)
- 연결: #1600(조사, CLOSED), #1608(요인 A 분리)
- base ← head: `devel` ← `planet6897:pr/devel-1600-squash`
- 처리일: 2026-06-28

## 1. 처리 결정

**admin merge.** 문서·테스트 자산만 추가하는 PR(소스 코드 변경 없음). HWPX 무손실 3차원 검증
결과와 렌더링 -1쪽 갭 다요인 조사, 재사용 통제셋/게이트 자산을 반영한다. CLEAN + CI 전부 pass
+ 충돌 0건.

## 2. 변경 범위 (6 files +477/-0, src 변경 0)

| 파일 | 내용 |
|---|---|
| `mydocs/report/hwpx_lossless_3axis_20260627.md` | 무손실 검증 — roundtrip 99.97% PASS, 페이지 붕괴율 0%(이전 ~16%, #1597 효과), 렌더 90.7% 정합 |
| `mydocs/tech/investigations/issue-1600/render_minus1_page_gap.md` | -1쪽 갭 다요인 조사(요인 A is_hwp3_origin 오탐지 → #1608, 요인 B footer ~60px 부족) |
| `mydocs/plans/task_m100_1600.md`, `mydocs/working/task_m100_1600_stage1.md` | 계획/단계 |
| `tests/fixtures/render_page_controlset.tsv` | 통제셋 92건(한글 PageCount 정답지) |
| `tools/render_page_gate.py` | rhwp vs 정답지 confusion matrix 게이트 |

## 3. 검토

- **소스 변경 0** → 빌드/테스트 영향 없음.
- `render_page_gate.py` py 구문 OK, 통제셋 TSV 형식 정상(92건), 자기검열 통과.
- **통제셋 실문서 비공개는 정상 설계**: 게이트가 `--root C:/Users/planet/hwpdocs`(컨트리뷰터
  로컬) 기준 동작. opengov 13,867건 실문서는 대용량/공공이라 커밋 불가 → 한글 정답지
  (hangul_pages)만 fixture 로 보존, 실문서는 로컬 root 지정. 대규모 회귀 게이트의 합리적 방식.
- #1597 페이지 붕괴 해소(0%)를 정량 검증, -1쪽 갭을 다요인으로 분해해 #1608 분리.

## 4. 검증

| 항목 | 결과 |
|---|---|
| GitHub CI (Build&Test/CodeQL/Analyze/Canvas visual diff) | 전부 pass |
| 충돌 시뮬레이션 | 0건 (CLEAN) |
| src 변경 | 0건 |
| 도구/픽스처 무결성 | py 구문 OK, TSV 92건 정상 |

## 5. 의의

통제셋(`render_page_controlset.tsv`)·게이트(`render_page_gate.py`)는 후속 -1쪽 갭 수정 시
−1쪽 해소 − 회귀 > 0 판정에 재사용될 회귀 자산이다.

## 6. 산출물

- 본 처리 보고서: `mydocs/pr/archives/pr_1609_review.md`
