# PR #3488 검토 — 빈 누름틀 안내문을 인쇄 등가 프로필에서 미출력

Issue: #3375 / author: planet6897 / reviewer: edwardkim / milestone: v1.0.0
연작: planet6897 7건 누적 검토(2026-07-28), 체리픽 5순위

## metadata (작성 시점 참고값)

| 항목 | 값 |
|---|---|
| 기능 커밋 | `278794900` → 누적 `9920b5453` |
| 규모 | +99 -0 (`paragraph_layout.rs` +5, `svg.rs` +15, `svg_layer.rs` +1, 테스트 +78) |
| CI | 원 PR head SUCCESS, mergeable CLEAN |

## 변경 — 렌더 영향 있음

한컴은 빈 누름틀 안내문(빨간 이탤릭)을 편집 화면에만 보이고 인쇄·PDF 에는 내보내지 않는다.
그림 placeholder 는 #2225/#2297 이 `editor_only` + `RenderProfile::shows_editor_visuals()`
프로필 계약으로 이미 처리했는데, **누름틀 안내문 축이 그 계약 밖에 있었고 SVG 렌더러가
렌더 트리를 직접 순회해 계약을 우회**했다.

수정: 안내문 노드에 `with_editor_only()` 표시, SVG 순회 진입에서 프로필 게이트, 원시
`SvgRenderer` 기본값은 표시 유지(legacy 편집 렌더 무변경).

## 시각 판정 — 신규 sweep 생략, 근거 4가지

[intake_and_review 2.6](../../manual/pr_review/intake_and_review.md) 상 렌더 변경이라 판정
대상이다. **생략을 판단**하며 근거를 남긴다.

1. **확립된 정책과 동형 확장** — 한컴 placeholder 인쇄 억제 분기(2026-07-12 확립,
   #2225/#2297 그림 placeholder 프로필 계약)를 누름틀 축으로 넓힌 것. 새 시각 규약이 아니다.
2. **screen/편집 프로필 무변화** — PR 실측(56자 유지)과 누적 branch 의 svg_snapshot golden
   무변화가 교차 확인.
3. **흐름 불변식이 테스트로 고정** — 안내문은 흐름 폭에 기여하지 않는 마커 노드라 억제해도
   쪽수·줄바꿈이 프로필에 따라 갈리지 않는다. 세 번째 테스트(본문 동일성)가 이를 고정.
4. **print 프로필의 의도 변화는 계약 테스트로 고정** — 억제 자체가 목적이며 실측(56→31자)이
   PR 본문에 있다.

시각 판정 선택 적용 원칙(전수 기계 적용 금지, PR 목적 기준)에 따른 판단이다. 작업지시자
승인 전에는 최종 통과로 단정하지 않는다.

## 검증

- focused 3건 통과 (편집 프로필 유지 / 인쇄 프로필 억제 / 두 프로필 차이가 안내문뿐)
- 누적 branch 전체 게이트: release-test 4253 passed / 0 failed, golden 무변화, fmt·clippy 클린

## 권고

**merge (통합 PR 경유).** 확립된 프로필 계약의 마지막 구멍(SVG 직순회 경로)을 닫는 정합
수정이다.
