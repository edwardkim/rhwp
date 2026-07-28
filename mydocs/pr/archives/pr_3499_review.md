# PR #3499 검토 — 한컴 PUA 사각 안 숫자를 텍스트 표면에서만 가독화

Issue: #3385 / author: planet6897 / reviewer: edwardkim / milestone: v1.0.0
연작: planet6897 7건 누적 검토(2026-07-28), 체리픽 7순위

## metadata (작성 시점 참고값)

| 항목 | 값 |
|---|---|
| 기능 커밋 | `5c67929c6` → 누적 `b74b50983` |
| 규모 | +137 -1 (`queries/rendering.rs` +6, `renderer/composer.rs` +40, 테스트 +92) |
| CI | 원 PR head SUCCESS, mergeable CLEAN |

## 변경

`export-text` 가 한컴 PUA 사각 안 숫자(U+F02B1~F02C4)를 원문 그대로 내보내, 폰트 없는
소비자(RAG·LLM·grep)에게 읽을 수 없는 코드포인트가 유출됐다(실측: 국립국어원 업무계획
35쪽 5건). 수정은 **텍스트 표면에서만** 가독 문자로 매핑한다.

**경계 판단이 정확하다.** 렌더의 raw PUA passthrough 는 Task #509 → 캡스톤 F-1
(2026-05-16)에서 표준 ①~⑳ 매핑을 일부러 되돌린 의도된 결정이다 — 매핑하면 1순위 폰트의
"원 안" 글리프가 나와 한컴의 "사각 안" 모양과 달라진다. 이 PR 은 그 결정을 건드리지 않고
(`map_pua_bullet_char` 불변) 텍스트 추출 표면만 분리해 처리한다. 컨트리뷰터가 이슈의
"미매핑 tofu" 프레임을 스스로 교정하고 렌더 불변을 명시한 점이 좋다.

## 검증

- focused 4건 통과 (텍스트 표면 매핑, 렌더 결정 불변, 비 PUA 무회귀, 경계 코드포인트)
- 누적 branch 전체 게이트: release-test 4253 passed / 0 failed, **svg_snapshot golden
  무변화** — "렌더 결정은 불변" 주장을 golden 이 교차 확인
- fmt·clippy 클린

## 시각 판정

불필요 — 렌더 출력 불변이 golden 으로 실증됐고, 변경 표면은 텍스트 추출뿐이다.

## 권고

**merge (통합 PR 경유).** 과거 캡스톤 결정을 존중하면서 텍스트 소비자 문제만 정확히 닫았다.
