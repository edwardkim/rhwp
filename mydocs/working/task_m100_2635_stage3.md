---
kind: working-note
status: active
issue: 2635
stage: 3
---

# Task #2635 Stage 3: PR 최종 준비

## PR 범위

- [#2635](https://github.com/edwardkim/rhwp/issues/2635)의 순수 RawSvg 차트 첫 화면 지연을 보정한다.
  일반 raster 이미지 정책은 유지하고, RawSvg가 있는 페이지만 제한된 조기 재렌더를 예약한다.
- [PR #2508의 인라인 정정](https://github.com/edwardkim/rhwp/pull/2508#discussion_r3611951113)을
  같은 커밋에 반영한다. `lpaiu-cs`의 잘못된 noreply canonical을 제거하고 실이메일 기반 alias만 남긴다.

## 최신 기준과 검증

- 최신 `upstream/devel` `bc0c09f80` 위로 rebase했다. upstream의 `examples/edit_sweep.rs` 변경과
  충돌은 없었다.
- Studio `npm test` 456건, production build, E2E 매니페스트 검사, `쪼개진원형.hwp` 400ms first-paint
  headless E2E를 통과했다. 유채색 픽셀 비율은 `1.664%`였다.
- `git check-mailmap`과 `git shortlog`에서 `lpaiu-cs <lpaiu.cs@gmail.com>` 42건 통합을 확인했다.

## PR 본문 초안

제목: `fix(studio): 순수 RawSvg 차트 첫 화면 지연 보정`

- `Closes #2635`를 사용한다.
- RawSvg 전용 `0/32/96/240ms` 조기 재렌더와 400ms 회귀 E2E를 설명한다.
- 일반 이미지 decode prefetch와 1500ms 안전망을 유지한 사실을 명시한다.
- #2508 merge 후 발견된 `.mailmap` canonical 정정을 별도 후속 범위로 기록한다.

## 승인 대기

현재 PR 생성·remote push 전이다. 전체 Rust CI 성격의 검증은 작업지시자 승인 뒤에만 실행한다.
