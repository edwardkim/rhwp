---
kind: working
status: active
issue_or_pr: 2522
stage: 1
last_verified: 2026-07-21
---

# PR #2522 RawSvg 프리페치 보완 Stage 1

## 목적

PR #2522의 순수 RawSvg 프리페치 보완만 최신 `devel` 위에 반영한다. 이미 merge된
PR #2654의 조기 재렌더와 중복되지 않도록, contributor 작업 로그와 증적 PNG는 가져오지 않는다.

## 확인 결과

- Studio가 사용하는 `getPageLayerTree`는 `paint/json.rs` LayerTree schema를 사용한다.
- RawSvg `bbox` 키는 `x`, `y`, `width`, `height`다.
- PR #2522의 프리페치 코드는 이 계약을 사용하므로, 구형 `getPageRenderTree` JSON의 `w/h`와
  혼동해 보정하지 않는다.

## 이번 단계 범위

1. PR #2522의 `page-renderer.ts` 프리페치 코드만 무커밋 cherry-pick한다.
2. 실제 LayerTree bbox 계약을 넣어 SVG data URL 생성 결과를 확인하는 focused 회귀 테스트를 추가한다.
3. Studio 단위 테스트와 TypeScript 검사로 보완 범위를 확인한다.
