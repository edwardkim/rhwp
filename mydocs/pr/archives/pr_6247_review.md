---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6247 review - CellContext 빈 경로 panic 방어

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6247
- 작성자: `kevin9327`
- 원 PR head: `6d3149551ea6`
- 통합 검토 브랜치: `review/open-ci-green-20260828`
- 최신 기준: `upstream/devel@5645e1f5b`
- 원 PR 상태: non-draft, `CLEAN`, 실패·진행 check 0건

## 검토 판단

**수용 권고.** `CellContext`의 빈 `path`에서 `self.path[0]` 또는 `unwrap()`으로 renderer/cursor
query가 panic하던 경로를 `Option` 기반 API로 바꾸고, 호출부를 `first()`/`innermost()` 가드로
정리했다. 빈 경로가 비정상 입력이더라도 렌더러 프로세스 종료로 이어지지 않게 하는 방어적 수정이다.

## 증적과 검증

- 원 PR 보고서:
  `mydocs/report/bug-layout-empty-path/{before,after}.png`
- 검토자가 직접 확인한 대표 after: 빈 경로 상황 설명과 after 산출물이 포함되어 있고, panic 대신
  `None` 경로로 내려가는 설계가 코드와 맞는다.
- 관련 focused/unit 검증: `cursor_rect` lib tests 16 pass / 5 ignored
- 통합 head 공통 검증: fmt, unit tier, suite manifest, clippy, 전체 nextest, Native Skia 3종,
  WASM build 통과.

## 코멘트 처리

merge 후 코멘트에는 `CellContext` 빈 경로가 panic 대신 `Option` 흐름으로 처리된다는 코드 판단과
`mydocs/report/bug-layout-empty-path/after.png` 확인 사실을 적는다. 별도 visual sweep 산출이 필요한
renderer 비교 사안으로 확대되면 `visual_sweep_guide.md#github-merge-comment`에 따라 대표 PNG와
summary를 `mydocs/pr/assets`에 보존한 뒤 merge SHA 고정 raw URL을 사용한다.

## 후속

추가 보정 필요 없음. `CellContext` public surface의 반환 타입 변경은 통합 head에서 모든 호출부가
컴파일·clippy를 통과했다.
