---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-25
---

# PR #7388 검토 — 저장 높이 0 도형의 속성 왕복과 되돌리기

## 최종 판정

**메인터너 보정 후 수용 가능.** 원 head `b19a3ecd0075417cf4e05d2b000b511bd592ca26`는 저장 높이 0 도형을 확대한 뒤 undo할 때 높이가 200으로 바뀌는 경계를 해결하지 못했다. 메인터너 보정 `f4ddd1f12945783f51a4124c4fa9727848f6cff9`를 포함한 통합 code head `459d5e581eac979e8a3c69a72e71fb7cc3c9ea88`에서 실제 객체의 0→400→0→400 편집과 회귀가 통과했다. 원 PR head 단독 merge 대상은 아니다. 이 판정은 GitHub approve가 아니며 통합 PR의 최신 head CI·mergeability와 작업지시자 원격 승인은 별도다.

## 접수 정보와 체리픽

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7388](https://github.com/edwardkim/rhwp/pull/7388), `planet6897`, `devel` 대상 |
| 원 head / 통합 적용 | `b19a3ecd0075417cf4e05d2b000b511bd592ca26` / `dc5369f942baec5a1a7d6c577c075a40bf706aa3` (`-x`) |
| 메인터너 보정 | `f4ddd1f12945783f51a4124c4fa9727848f6cff9` |
| 통합 순서 | 8건 중 첫 번째; 선행 PR 의존 없음, 체리픽 충돌 없음 |
| 규모·작성 시점 원격 참고값 | 5파일, +154/−12; OPEN, MERGEABLE/CLEAN, Draft 아님 (2026-09-25) |
| 검토 base / code head | `b3e3d4e2170a43ca449e3d832440a9274e4e8ee4` / `459d5e581eac979e8a3c69a72e71fb7cc3c9ea88` |

원 head CI는 누적 통합 head의 검증을 대체하지 않는다. [보정·실행 기록](pr_7388_review_impl.md)에 수정 전후 경계와 처리 순서를 구분했다.

## 변경과 조판 원칙

`clamp_degenerate_size`와 picture/shape setter가 이미 저장된 0 크기를 속성 되먹임에서 보존한다. 메인터너 보정은 `ResizeObjectCommand.undo`에 저장 0 복원 의도를 전달하고, 실제 높이 4 HWPUNIT의 가로선을 양수→0 clamp 대조군으로 추가했다. 저장 0→0과 일반 편집의 양수→0을 구분한다. 문단 측정·배치·paint 조판 규칙은 변경하지 않으므로 조판 원칙은 **비해당**이다. 편집 명령의 원래 크기 복원은 **충족**, 미검증한 다른 편집 UI 경로는 **미검증**이다.

## 검증 입력과 결과

입력은 모두 이 통합 commit에 이미 추적된 HWP다: `samples/basic/interview.hwp` (SHA-256 `fc71c2a0ebced5e913fbcbbc39a28244f5d73144f74ba1a4ebc548998d408d3c`), `samples/issue6023/30269_reform_recommendation.hwp` (`9e7cc9a3ea8c9d67a9df33fe49921ec87e6f5c38d5cc53b9885a37ef5b285ef7`), `samples/21_언어_기출_편집가능본.hwp` (`905454045ca2e236839a7cab59750678116d08af3db31dbf846819af355b8d15`). 순서대로 allowOverlap, 저장 0 도형, 실제 가로선 대조군에 사용했다. 개인 경로에만 있는 입력은 수용 근거로 쓰지 않았다.

- 보정 전에는 저장 0 도형의 확대 후 undo가 200으로 복원돼 FAIL이었다. 보정 뒤 `issue_6806`을 포함한 집중 release-test에서 관련 검사 20건이 PASS했다.
- 통합 code head의 전체 release-test nextest **10,229/10,229 PASS·50 skip**, fmt·Native/WASM/workspace Clippy·workspace build·manifest base 비교가 PASS했다.
- Studio `npm test` **1,770 PASS·2 skip**, `tsc --noEmit`와 build PASS. 같은 code head에서 새로 만든 Mac 로컬 WASM을 쓴 headless Chrome E2E가 저장 0→확대 400→undo 0→redo 400을 모두 확인했다. 실행 로그는 무시되는 `output/pr-review/planet6897-20260924/diagnostics/approved-v2-browser-e2e-fresh.txt`에 있다.

## 시각 증적과 남은 범위

이 변경은 문서의 조판·paint 계산이 아니라 편집 속성 복원을 바꾸므로 한컴 PDF 대비 Visual Sweep은 **비해당**이다. 실제 WASM 브라우저의 객체 크기 전이가 사용자 화면 경계의 직접 증거다. 이 결과로 #6806 전체 수정 완료나 다른 renderer PR의 시각 일치를 주장하지 않는다.

## Merge 후 contributor PR comment 계획

통합 PR이 실제 merge된 뒤에만 merge SHA와 CI URL, 원 기여의 속성 왕복 수정과 메인터너의 저장 0 undo 보정 범위를 구분해 한국어 존댓말로 알린다. #6806 전체 이슈 종료 표현은 사용하지 않는다. 작업지시자 게시 승인 뒤 UTF-8 본문 파일을 `--body-file`로 전송하고 API로 내용을 다시 확인한다. 현재 댓글·approve·push·merge는 하지 않았다.
