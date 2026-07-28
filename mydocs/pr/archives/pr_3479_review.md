# PR #3479 검토 — 분할 표 셀 매치가 실제 렌더 쪽을 보고

Issue: #3403 / author: planet6897 / reviewer: edwardkim / milestone: v1.0.0
연작: planet6897 7건 누적 검토(2026-07-28), 체리픽 3순위

## metadata (작성 시점 참고값)

| 항목 | 값 |
|---|---|
| 기능 커밋 | `f3b16fea8` → 누적 `ab6e28aa8` |
| 규모 | +177 -13 (`queries/grep.rs` +102, 테스트 +88) |
| CI | 원 PR head SUCCESS, mergeable CLEAN |

## 변경

표가 쪽 경계에서 분할될 때 grep/search 의 셀 매치가 표의 시작 쪽 번호를 일괄 보고해,
분할 후반부 셀의 인용이 실제 렌더보다 한 쪽 앞을 가리켰다(RAG 인용 어긋남). 수정은 매치된
셀이 실제로 그려진 쪽을 렌더 분할 결과에서 역참조한다. 쿼리 표면만 변경한다.

## 검증

- focused 2건 통과 (분할 표 후반부 셀의 쪽 번호 정정, 비분할 표 무회귀)
- 누적 branch 전체 게이트: release-test 4253 passed / 0 failed, fmt·clippy 클린

## 시각 판정

불필요 — 쿼리 결과의 쪽 번호 메타데이터 변경, 렌더 출력 무변경. 표 분할 자체는 기존
레이아웃 결과를 읽기만 한다.

## 권고

**merge (통합 PR 경유).**
