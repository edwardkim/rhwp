# PR #1597 처리 보고서 — HWPX 직렬화 무손실 + 페이지붕괴 군집 해소 (#1586 통합 squash)

- PR: https://github.com/edwardkim/rhwp/pull/1597
- 제목: `HWPX 직렬화 무손실 개선 + opengov 회귀 말뭉치 + 페이지붕괴 군집 해소 (#1586 통합 squash)`
- 작성자: planet6897 (collaborator)
- 연결: Closes #1586. 통합: #1584/#1587/#1588/#1592/#1594/#1595/#1596/#1598 + #1564/#1589/#1591
- base ← head: `devel` ← `planet6897:pr/devel-1586-squash`
- 처리일: 2026-06-27

## 1. 처리 결정

**admin merge.** devel 통합 작업을 upstream/devel 기준 단일 squash 로 정리(PR #1586 대체).
HWPX 직렬화 무손실 개선과 "IR diff=0 인데 한글에서만 페이지 붕괴"하는 IR-invisible 결함 군집을
해소한다. CI 전부 pass + 전체 회귀 통과 + opengov/baseline 게이트 통과 + 충돌 0건.

## 2. 변경 범위

68 files (src 10 · tests 3 · tools 1 · docs 48), +2886/-64.

src 는 전부 HWPX 직렬화/파서 영역: `serializer/hwpx/{field,section,shape,table,picture,
roundtrip,context,mod}.rs`, `parser/hwpx/section.rs`, `model/control.rs`. HWP3 룰·공통 모듈 침범 없음.

## 3. 코드 검토 — 핵심: IR-invisible 페이지 붕괴 군집(~16%) 분해 해소

한글 오라클(PageCount) + 단락 이진탐색으로 4종 직렬화 결함 분리:

1. **#1595 ClickHere** `field.rs` `"CLICKHERE"`→`"CLICK_HERE"`(언더스코어 누락). 파서 관대로
   IR diff=0 미검출이던 **지배원인**(붕괴파일 96%, 오라클 37/40 해소). 테스트 `type="CLICK_HERE"` 갱신.
2. **#1594 holdAnchorAndSO** 하드코딩 "0"→IR `prevent_page_break` 방출. **`diff_documents` 게이트
   강화**: `diff_hold_anchor()` 추가로 IR-invisible 갭 봉인(재발 방지). roundtrip 게이트 테스트 추가.
3. **#1596 generic-shape**(polygon/curve) lineShape/fillBrush/shadow/꼭짓점 직렬화 복원.
4. **#1598 ellipse/arc** center/축/시작끝점 파서 미적재+직렬화 드롭 복원 (신규 `issue_1598` 테스트).

추가: **#1587 Ruby**(덧말) 모델 확장(main_text/pos_type/align) + 파서/serializer, **#1588** 선 도형
shapeComment, **#1592** 빈 문단 spurious(0,0), **#1564** opengov 회귀 말뭉치+스냅샷 게이트.
조사: **#1589** 페이지 붕괴 군집(`tools/verify_hangul_pages.py`), **#1591** para0 북마크 hoist(순효과 0, 롤백).

## 4. 검증 (로컬)

| 항목 | 결과 |
|---|---|
| GitHub CI (Build&Test/CodeQL/Analyze) | 전부 pass |
| 충돌 시뮬레이션 | 0건 (CLEAN) |
| 신규 `issue_1598_ellipse_geometry_roundtrip` | 통과 |
| `opengov_corpus_snapshot` (말뭉치 게이트) | 2/2 |
| `hwpx_roundtrip_baseline` | 4/4 |
| `visual_roundtrip_baseline` | 3/3 |
| 전체 `cargo test --tests` | **FAILED 0건** (lib 1970 passed) |
| fmt / clippy | clean |

## 5. 게이트 강화 평가

단순 결함 수정을 넘어 `diff_documents` 에 `diff_hold_anchor`(prevent_page_break) 검사를
추가해, IR-invisible 이던 갭을 게이트가 검출하도록 강화 → 동종 회귀 재발 방지. 우수.

## 6. 주의 — 시각/한글 오라클 권위

페이지 붕괴 해소(오라클 92.5%)는 한글 환경(PageCount) 측정이다. 로컬에서는 게이트 강화 +
roundtrip + visual baseline 통과로 간접 검증했고, 한글 페이지 붕괴 자체는 작업지시자 한글
오라클이 권위다(`feedback_self_verification_not_hancom`). squash 라 task 단위 히스토리는 원본
devel 브랜치에 보존됨(PR 본문 명시).

## 7. 산출물

- 본 처리 보고서: `mydocs/pr/archives/pr_1597_review.md`
