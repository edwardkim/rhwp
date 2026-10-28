---
kind: reference
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-26
---

# PR #3323 메인터너 보정 계획

## 적용 순서

1. 최신 `upstream/devel` `99732b2a1189`에서 `review/lpaiu-cs-hf-field-20260726`를 만들었다.
2. contributor의 #3212 보정 revert `d9c5b325`와 #3216 본체 `ca8219d232`를 순서대로 적용했다.
3. AutoNumber(Page) 치환은 blanket marker replacement를 제거하고 컨트롤 placeholder 위치만
   `display_text` run으로 분리했다. 명시 필드와 AutoNumber가 공존하는 회귀 테스트를 추가했다.
4. native `insertFieldInHf` 응답에 `insertedAt`·`insertedLength`를 추가했다. Studio history command는
   redo의 원 cursor 좌표와 undo의 실제 marker 범위를 분리해 저장한다. trailing inline control 회귀와
   Studio source contract test를 추가했다.
5. contributor가 최신 `devel` merge 뒤 P2 `5ce61c9`를 push했다. 이미 base에 있는 merge는 중복 적용하지
   않고 P2만 `cf2e52cc7`으로 적용해, split 조각별 PUA display 재계산 및 page text 추출 배선을 포함했다.
6. 최초 #3325 CI의 #1692 실패를 재현했다. `TextRunNode.display_text`가 render-tree JSON에 빠져 있었으므로
   raw `text`는 모델 좌표로 유지하면서 `displayText`를 조건부 직렬화하고, #1692는 표시 문자열 우선으로 footer
   AutoNumber(Page)를 검증하게 했다.
7. 최종 전체 Rust에서 #1100의 `fwSpace` SVG 앵커 회귀를 확인했다. AutoNumber의 raw run을 쪼개지 않고
   `display_text`만 재구성해 연속 glyph advance를 보존했다. 분리된 명시 field marker는 비반올림 bbox 폭 및
   bbox 끝 기반 캐럿 경계를 사용해 표시 폭과 모델 한 글자 경계를 일치시킨다.

## 검증·PR 준비

1. Rust release-test 집중 회귀(#3216 5 tests, #1100 3 tests, #1692 1 test), 기존 #3216·#1144 회귀,
   전체 Rust test(lib 2,923 passed/7 ignored 포함), `cargo fmt --check`와 Native Skia 필수 3종(57·2·4)을
   최종 head에서 완료했다. Studio build/test는 P2 이후 Studio 파일을 바꾸지 않아 기존 통과 결과(637 tests)를 유지한다.
2. `samples/SO-SUEOP.hwpx` 5쪽을 Native Skia PNG로 만들고 육안 확인한 뒤 review 기록에 inline asset으로
   넣었다. AutoNumber(Page) `5`, 머리말 제목·밑줄, 꼬리말 학교명이 보이는 것을 확인했다.
3. source/test 보정과 review 문서를 별도 commit으로 정리해 `devel` 대상 통합 PR의 최신 head를 push한다.
4. 최신 head의 required CI가 통과하고 작업지시자가 승인한 뒤에만 merge한다.

## rollback

- display/model 규약 보정 전체를 되돌릴 때는 메인터너 보정 commit과 contributor commits를 역순으로
  revert한다.
- review 문서·asset만 되돌릴 때는 문서 commit만 revert하며 source·test 변경과 섞지 않는다.
