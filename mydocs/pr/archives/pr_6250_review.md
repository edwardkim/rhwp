---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6250 review - font/border 인덱스 OOB 방어

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6250
- 작성자: `kevin9327`
- 원 PR head: `9fc79fdd477b`
- 통합 검토 브랜치: `review/open-ci-green-20260828`
- 최신 기준: `upstream/devel@1a43a507c9da`
- 원 PR 상태: non-draft, 실패·진행 check 0건

## 검토 판단

**수용 권고.** 폰트 메트릭 range 정의 불일치, 손상된 table cell row/col, 비정상 border width 값이
직접 인덱싱으로 renderer panic을 만들 수 있는 경로를 `.get()`/경계 가드/fallback으로 바꿨다.
보안성·안정성 관점의 방어적 수정이며 정상 값 경로의 의미는 유지된다.

## 증적과 검증

- 원 PR 시각 보고서:
  `mydocs/report/bug-font-border/{before,after}.png`
- PR 설명은 특정 한컴 기준 PDF와의 페이지 fidelity가 아니라 font/border 인덱스 OOB 방어다.
  따라서 버전별 MCP 기준 PDF/visual sweep을 별도 산출할 대상은 아니며, `.get()`/fallback 경계
  방어와 통합 head 검증을 판단 중심으로 둔다.
- 검토자가 직접 확인한 대표 after: 전체 페이지 이미지에서 font/border OOB 방어가 의도한 입력 방어
  성격과 맞고, 눈에 띄는 렌더 회귀는 확인되지 않았다.
- focused/unit 검증:
  - `index_matches_legacy_linear_scan_exhaustively` 1 pass
  - `issue4149_fast_path_parity_giant_cell`를 포함한 `cursor_rect` 16 pass / 5 ignored
- 통합 head 공통 검증: fmt, unit tier, suite manifest, clippy, 전체 nextest, Native Skia 3종,
  WASM build 통과.

## 코멘트 처리

merge 후 코멘트에는 font/border OOB 방어 성격, focused/unit 검증 통과, `bug-font-border/after.png`
직접 확인 결과를 함께 남긴다. 이 PR은 기준 PDF 대조가 수용 판단의 중심이 아니므로 visual sweep
이미지는 코멘트 필수 증적에서 제외한다.

## 후속

추가 보정 필요 없음.
