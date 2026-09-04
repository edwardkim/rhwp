---
kind: investigation
status: active
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-08-20
---

# #5692 마스터 페이지 그림 hit-test 보정

Issue: #5692

## 재현

`samples/2025 행정업무운영 편람(최종).hwp`의 물리 37쪽에서 본문을 클릭하면
캐럿이 나타나지 않고 입력할 수 있다. 물리 38쪽의 같은 본문 클릭은 정상이다.

## 원인

37쪽의 페이지 전체 이미지가 `plane: 1`, `wrap: inFrontOfText`로 직렬화되어
`findPictureAtClick`의 foreground 후보가 되었다. CanvasKit 재생은 master-page
레이어를 본문 뒤에 그리므로, 렌더 순서와 입력 hit-test의 해석이 서로 달랐다.

## 보정

header/footer 개체가 아닌 `plane: 1` master-page 장식은 그림 선택 후보에서 제외한다.
일반 본문 foreground 이미지는 기존처럼 선택하도록 별도 회귀 테스트로 고정한다.

## 검증 결과

- `cd rhwp-studio && npx tsc --noEmit`이 통과했다.
- `cd rhwp-studio && node --test tests/master-page-picture-hit.test.ts`가 2건 통과했다.
- `cd rhwp-studio && npm test`가 1,021건 통과와 기존 skip 1건으로 종료했다.
- `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check`이 통과했다.

원본 HWP를 브라우저 파일 선택기로 재주입하는 동작은 로컬 파일 업로드이므로 자동화하지 않았다.
원인 재현에서 확인한 page 37의 master-page control과 hit-test 결과, 그리고 동일 정책을 실행하는
회귀 테스트로 보정 범위를 검증했다.
