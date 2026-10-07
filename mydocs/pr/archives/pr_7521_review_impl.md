---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-05
---

# PR #7521 메인터너 보정

## 근거와 범위

작업지시자가 보류 사유의 메인터너 보정을 승인했다. 기존 `flush_text_to_paragraphs`의 `FLUSH_LINE_CHAR_CAP=4000`은 문자 수를 기준으로 긴 입력을 나눠 조판 비용을 제한하는 import 계약이다. 원 변경의 `flush_inline_run` 태그 분기는 이 제한을 우회한다. HTML 서식 파싱 후 모델의 기존 `Paragraph::split_at`으로 내용·UTF-16 서식 원점·인라인 그림 소유를 함께 분할한다. 짧은 서식 입력, 명시적 `<br>`, 일반 plain 입력은 기존 계약을 유지한다. 특정 문서나 픽셀 좌표 분기는 추가하지 않는다.

## 단계

1. 같은 Native public paste API에서 긴 loose bold 입력의 수정 전 실패를 확인한다.
2. 파싱 결과를 기존 문자 수 제한으로 분할한다. HTML 인라인 파싱/CSS 부분을 자식 module로 옮겨 수정 코드 파일을1000줄 이내로 유지한다.
3. 서식 경계·비BMP 문자·그림 경계·기존6개를 실행해 내용/서식/개체 보존을 확인한다. 이는 import 계약 검증이며 한컴 조판 일치로 주장하지 않는다.
4. 필수 lint/build/정책, 새 fresh WASM과 public JS 동기화, 실제 public paste 재확인 결과를 개별 review에 연결한다. 기존 #7491의90% 미달과 px 검사 실패는 이 보정으로 해결됐다고 쓰지 않는다.

## 실행 결과

Native public paste에서 기존6 PASS/긴 비BMP bold1 FAIL(`[8001]` vs `[4000,4000,1]`)을 확인했다. 보정 후 서식 경계·비BMP·그림3999/4000/4001 경계를 포함한8개가 모두 PASS다. 최초 단순 split에서 그림4000의 원점이 새 문단 끝으로 옮겨지는 실패도 기록했고, 경계 그림의 선행 확장 제어문자 공간을 기존 모델 helper로 확보한 후 재실행했다. 파일은 html_import915+8줄/inline_content513줄이며 기준4,000자를 변경하지 않았다.

원 로그: `output/pr-review/semanticist21-20261005/logs/7521-cap-before-prepared.log`, `7521-cap-boundaries-final.log`, `7521-cap-complete.log`. prepared되지 않은 suite의0 tests는 증거에서 제외했다. fmt·diff check PASS. 필수 lint/build/정책 및 fresh WASM은 #7491 보정과 함께 최종 source에서 순차 수행하며 아직 완료로 기록하지 않는다.
