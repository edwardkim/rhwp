# Task M100 #6788 — 혼합 글자 서식 보존 최종보고서

- Issue: [#6788](https://github.com/edwardkim/rhwp/issues/6788)
- 작성일: 2026-09-06
- 상태: 최초 구현 검증·[PR #6814](https://github.com/edwardkim/rhwp/pull/6814) 생성 후
  [리뷰 보정 검증 중](../pr/archives/pr_6814_review_impl.md). 아래 최초 검증 수치는 새 보정 후보와 구분한다.
- 계획: [수행계획](../plans/task_m100_6788.md), [구현계획](../plans/task_m100_6788_impl.md).
- 단계 증빙: [1단계](../working/task_m100_6788_stage1.md),
  [2단계](../working/task_m100_6788_stage2.md), [3단계](../working/task_m100_6788_stage3.md).
- 기준 devel: `51ad998e33ef7f5191b0e1b0b656dc44cef33a1c`.
- 제품 source: `ff716d797dedb60ee8db236b28428be88089af9f`.
  이후 Rust 변경은 순수 조회 API의 passthrough 가드 분류 1건이며 변경 후 필수 검증을 재실행했다.

## 결과와 원인

혼합 글자색 선택에 형광펜을 적용해도 원래 색·굵기 등 미지정 속성이 유지되고,
Undo/Redo는 형광펜 적용 전후의 구간별 모양을 복원한다. 선택 밖 글자는 바뀌지 않는다.

코어는 선택 시작의 모양 ID 하나로 범위 전체를 덮었고, Studio history도 문단당 ID 하나만
저장했다. 두 계층을 각각 수정했다. 코어는 기존 구간별로 지정 속성만 병합하고,
Studio는 구간 경계·ID 목록을 capture/restore한다. 본문·일반/중첩 셀과 기존 F5·머리말/꼬리말
경로를 검증했다. renderer/layout 정책은 바꾸지 않았다.

## 의도된 동작 — Chrome·Firefox 실제 화면

두 브라우저의 새 문서에서 `다라`는 보라색, `나다라마`만 노란 형광펜이다.
위에서 적용 전 → 형광펜 → Undo → Redo. 실제 UI를 조작하고 본문 crop에 상태 라벨만 추가했다.

![Chrome·Firefox 형광펜 적용·Undo·Redo](../pr/assets/issue6788_browser_behavior.png)

각 브라우저에서 사용자가 저장한 HWP·HWPX를 에이전트가 직접 다시 열었다.
네 파일 모두 보라색·형광펜이 유지되며, 문자별 색상 데이터 검사도 통과했다.

![Chrome·Firefox HWP·HWPX 재열기](../pr/assets/issue6788_browser_reopen.png)

## 검증 요약

| 검증 | 결과 |
| --- | --- |
| fmt·native/WASM/workspace-all-targets Clippy·workspace build·manifest | 모두 통과. |
| Rust 전체 nextest | 9071 passed, 0 failed, 46 skipped. |
| Native Skia lib / placeholder / direct PDF | 3930 + workspace 182 / 2 / 4 passed; lib 13 ignored. |
| Studio 전체 / binding 계약 | 1427 / 22 passed, 실패·skip 0. |
| 실제 WASM+Studio history focused | 13개 시나리오 통과. Rust focused 15개, Studio 관련 62개 통과. |
| Studio·Firefox 확장 빌드 | 통과. web/Node/Firefox WASM 동일 해시. |
| Chrome·Firefox 직접 UI | 각 새 문서의 전체·부분 형광펜 적용, Undo/Redo 정상. |
| 브라우저에서 저장한 HWP·HWPX 네 파일 | 해당 브라우저의 파일 열기로 재열기 정상, 7글자 전체 색상·형광펜 보존. |
| HWP/HWPX 4상태 × 2포맷 | CLI 재저장 8개 IR diffCount 0, 1페이지 유지. 재적재 후 7글자의 이슈 대상 5속성 일치. |
| Native PNG 전체 페이지 비교 | before=undo, highlight=redo, 동일 상태 HWP=HWPX: 총 8쌍 모두 0픽셀 차이. |

아래는 CLI 렌더링이다. 왼쪽 HWP/오른쪽 HWPX, 위에서 적용 전/형광펜/Undo/Redo.
Chrome 스크린샷이나 한컴 독립 정답지가 아니다.

![저장 후 재열기 8개 파일의 native 렌더링](../pr/assets/issue6788_cli_roundtrip.png)

## 한계와 후속 게이트

- 브라우저 검증 대상은 로컬 Studio다. 기존 확장은 교체하지 않았다.
- Docker daemon 부재로 locked native `--no-opt` WASM을 사용했다. 최적화 배포 산출물 검증은 아니다.
- nextest 성공 실행에 `issue_2007_single_cell_continuation_does_not_repaint_boundary_fragments`
  LEAK 1건이 있었다. 잔류 출력 핸들/자식 프로세스의 원인은 확정하지 않았다.
- 최초 HWPX export의 fillType/patternColor/patternType 차이는 적용 전부터 관찰됐다.
  이슈 대상 textColor/shadeColor/bold/fontSize/fontFamily 보존과 전체 속성 무손실을 구분한다.
- 브라우저 webfont와 native 시스템 fallback의 글꼴 모양은 다를 수 있다. 광범위 렌더링 fidelity
  또는 전체 visual sweep 통과를 주장하지 않는다.
- PR 준비까지 완료하며 push·PR 생성·CI·merge·이슈 종료는 별도 승인/후속 게이트다.
  PR 번호 확정 전 archive self-review 및 오늘할일을 만들지 않는다.

로컬 Studio 서버는 사용자 추가 확인용으로 유지한다. 상세 명령·초기 실패·재실행·산출물 경로는
3단계 보고서에 있다. 원시 로그·중간 JSON·중복 PNG는 커밋하지 않는다.

## PR 제출 기록

자동 검증 마감 commit은 `c3b1398e4745f6d0030321df525e787d575f8ab3`이다.
이후 Chrome·Firefox 직접 검증을 추가하고 [PR 본문 초안](../working/task_m100_6788_pr_body.md)을
핵심 변경·검증·실제 UI 패널 중심으로 줄였다. 이미지·보고서 URL은 새 증적 commit SHA에 고정한다.
문서·이미지 전용 후속 commit은 제품/test source를 바꾸지 않는다. 사용자 승인 후
`4936663ea4b6019ddc83c0ca0fafe41a0bae3058`을 upstream 작업 branch로 push하고 Open PR #6814를
생성했다. 게시한 한글 본문과 두 이미지의 SHA 고정 원격 blob이 로컬과 같음을 확인했다.

다음 제출 명령을 실행했다. 제출 전 최신 devel `6a193a648dba3df6d5c4cffa0182bc02f3e011ff`와
merge simulation은 충돌 없이 통과했고 검증 이후 제품/test source 변경은 없었다.

```bash
git push upstream HEAD:codex/6788-preserve-mixed-char-format
gh pr create --repo edwardkim/rhwp --base devel \
  --head codex/6788-preserve-mixed-char-format \
  --title "fix: preserve mixed character formatting through highlight and undo (#6788)" \
  --body-file mydocs/working/task_m100_6788_pr_body.md
```

채번 후 [self-review](../pr/archives/pr_6814_review.md)를 같은 branch에 추가한다.
최신 head의 GitHub Actions와 merge 승인은 남아 있으며 원격 이슈를 종료하지 않았다.
