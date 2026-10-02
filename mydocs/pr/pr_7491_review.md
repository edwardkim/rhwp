---
kind: report
status: active
last_verified: 2026-10-02
---

# PR #7491 리뷰 — 편집 문단 들여쓰기

## 최종 판정

머지 보류 — 실패 assertion의 독립 기대값 및 편집 후 시각 증거를 확인해야 한다. 새 코드 회귀로 단정하지 않는다.

검토일: 2026-10-02. 작성자: semanticist21. 대상: devel.
기준 devel: `e5098bc91be44a49367a7f2895a14fcd4f4c2c7f`.
누적 진단 head: `c6ef30ea943308c37e5d68c8304dfdabdd7b8f74`.

누적 실행 명령·로그·제한은 [일괄 검토 기록](pr_semanticist21_20261002_review_impl.md#누적-검증-결과)에 연결한다.
원 PR의 exact-head 녹색 CI와 누적 진단 head의 결과는 별개다. 누적 head는 7건을 포함하며
원 PR 또는 최종 수용 그룹의 전체 CI 통과로 간주하지 않는다. 메인터너 source/test 보정은 없다.

원 PR code head: `c4367ec03a28369cc6f26b17eca46553ac61514c`.
[원 PR](https://github.com/edwardkim/rhwp/pull/7491) · [exact-head Build & Test](https://github.com/edwardkim/rhwp/actions/runs/36738946303/job/110182439149).
CI 집계 실패·진행 중 없음(확인 당시). 원 PR head는 최초 접수 이후 바뀌지 않았다.
Reviewer edwardkim 지정. 원격 GitHub 승인 이벤트는 아직 게시하지 않았다.

## 범위와 호출 경로

#7490 종료를 제안하는 PR이다. 기능 commit 4개를 적용했다.
`mark_indented_lines`는 문단 속성과 저장 LineSeg 기록으로 재조판 bit20을 정한다.
`restamp_indentation`은 명시적인 들여쓰기 변경을 본문·셀·머리말/꼬리말·각주·복원/외부 붙여넣기에 반영한다.
저장 기록상 들여쓰기 없는 #6190 표 호스트 예외는 원 저장본의 플래그를 근거로 보존한다.

## 실행 관측과 실패 분류

focused 10개 중 9 PASS, 1 FAIL. 별도 #6190 원 저장본 대조군 1/1 PASS.
실패는 `editing_keeps_hancom_record_of_unindented_line`의 편집 후 표 우변 검사다.
`samples/issue6190/center_align_first_line_indent.hwp`를 실제 API로 읽고 같은 두 편집을 실행했다.

| 경계 | devel e509 | 누적 c6ef | 판정 |
| --- | --- | --- | --- |
| 편집 전 대상 표 x / 우변 | 98.2933 / 695.4133px | 동일 | 차이 없음 |
| 제목 편집 후 대상 표 | 98.2933 / 695.4133px | 동일 | 차이 없음 |
| 표 호스트 맨 앞 글자 입력 후 | 113.48 / 710.6px | 동일 | 기존 동작 재현 |
| 본문 우변 | 699.2px | 동일 | 실패 기준 |

LineSeg flag는 393216(0x60000)으로 들여쓰기 bit20이 꺼져 있다. 따라서 ‘bit20이 다시 켜져
표가 밀렸다’거나 ‘이번 PR이 처음 만든 회귀’라고 보고하지 않는다. 이 입력은 원래 표 앞에
글자를 새로 넣는 편집이다. 독립적인 한컴 편집 후 출력 없이는 해당 우변 assertion이
편집 의도를 정확히 나타내는지 확정할 수 없다. 테스트·baseline·허용치를 변경하지 않았다.

## 시각·해제 조건

원본 SVG 불변/코퍼스 해시는 편집 후 출력의 정답지가 아니다. 편집 사례의 독립 PDF와
Native/fresh WASM 시각 게이트는 미검증이다. 기여자가 이를 마련하고 실패 계약의 근거를
확인해야 한다. 판정 수정과 코드 수정 필요 여부는 그 증거로 결정한다. merge 미실행.

## 승인 후 게시 기록

2026-10-02 작업지시자의 댓글 게시 승인 후 [보류 사유 comment](https://github.com/edwardkim/rhwp/pull/7491#issuecomment-5944041501)를 게시했다.
게시 직전 원 head가 그대로 OPEN임을 확인하고 API 재조회로 한글 본문·BOM/치환 없음 및
작성 문안과의 일치를 확인했다(파일 끝 개행만 정규화). 코드 변경·push·GitHub 승인·merge 없음.
