---
kind: report
status: active
last_verified: 2026-10-02
---

# PR #7504 리뷰 — 커닝 글리프·캐럿·등록 글꼴 비용

## 최종 판정

머지 보류 — renderer 변경의 신규 회귀에 필요한 독립 PDF/Native/fresh WASM 증거가 없다.

검토일: 2026-10-02. 작성자: semanticist21. 대상: devel.
기준 devel: `e5098bc91be44a49367a7f2895a14fcd4f4c2c7f`.
누적 진단 head: `c6ef30ea943308c37e5d68c8304dfdabdd7b8f74`.

누적 실행 명령·로그·제한은 [일괄 검토 기록](pr_semanticist21_20261002_review_impl.md#누적-검증-결과)에 연결한다.
원 PR의 exact-head 녹색 CI와 누적 진단 head의 결과는 별개다. 누적 head는 7건을 포함하며
원 PR 또는 최종 수용 그룹의 전체 CI 통과로 간주하지 않는다. 메인터너 source/test 보정은 없다.

원 PR code head: `7dc340284bd84e2ee475da3b577146005b989500`.
[원 PR](https://github.com/edwardkim/rhwp/pull/7504) · [exact-head Build & Test](https://github.com/edwardkim/rhwp/actions/runs/36895895873/job/110663859921).
CI 집계 실패·진행 중 없음(확인 당시). 원 PR head는 최초 접수 이후 바뀌지 않았다.
Reviewer edwardkim 지정. 원격 GitHub 승인 이벤트는 아직 게시하지 않았다.

## 범위와 실제 소비 경로

#7503 종료 제안. 5개 기능 commit을 누적 적용했다.
`glyph_fit_positions` → SVG textLength / web_canvas 수평 glyph fit은 자연 폭을 사용하고,
실제 layout_positions는 커닝 후 위치로 유지한다. 가운데 점·원문자 등 원래 점유 폭 보존 경계도 검토했다.
셀 fast caret은 등록 source로 커닝을 수행할 때 exact 경로로 이동한다.
`hashed_on_registration`은 등록 시 해시 고정한 immutable Arc registry만 true이며,
기본 provider는 false라 일반 외부 제공자의 검증을 생략하지 않는다.

## 검증과 증거 분류

focused 5/5 PASS와 관련 #4968 대조 30/30 PASS. 기능 결함이 이번 실행에서 검출된 것은 아니다.
원 exact-head Full CI도 성공이다. 누적 fresh WASM build는 PASS했지만 이 PR의 커닝/캐럿
경로를 실제 등록 글꼴과 Canvas에서 독립 한컴 기준으로 비교한 것은 아니다.

SVG/web_canvas paint를 실제로 수정하므로 PR 본문의 ‘Visual Sweep 비해당’은 수용하지 않는다.
신규 렌더링 회귀는 동일 입력의 독립 한컴 PDF와 Native/fresh WASM 관련 쪽 모두의
최저 90% 증거가 필요하다. 합성 문자 위치 assertion, 기존 출력 해시, CI 녹색으로 대체할 수 없다.
미지원 글꼴 추정만으로 font mismatch 예외를 적용하지 않는다.

## 해제 조건

기여자가 실제 적용 글꼴을 공급·검증하고 영향 경로의 비교 증거 및 head SHA 고정 대표 이미지를
제출해야 한다. 비용 개선과 배치/시각 증거는 별도로 판정한다. merge 미실행.

## 승인 후 게시 기록

2026-10-02 작업지시자의 댓글 게시 승인 후 [보류 사유 comment](https://github.com/edwardkim/rhwp/pull/7504#issuecomment-5944042128)를 게시했다.
게시 직전 원 head가 그대로 OPEN임을 확인하고 API 재조회로 한글 본문·BOM/치환 없음 및
작성 문안과의 일치를 확인했다(파일 끝 개행만 정규화). 코드 변경·push·GitHub 승인·merge 없음.
