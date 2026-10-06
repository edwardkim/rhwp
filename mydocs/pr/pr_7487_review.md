---
kind: report
status: active
last_verified: 2026-10-02
---

# PR #7487 리뷰 — Enter 후 빈 페이지 소유

## 최종 판정

머지 보류 — 독립적인 편집 후 기준 출력과 현행 시각 게이트 증거가 부족하며, 오늘할일 문서 충돌이 남는다.

검토일: 2026-10-02. 작성자: semanticist21. 대상: devel.
기준 devel: `e5098bc91be44a49367a7f2895a14fcd4f4c2c7f`.
누적 진단 head: `c6ef30ea943308c37e5d68c8304dfdabdd7b8f74`.

누적 실행 명령·로그·제한은 [일괄 검토 기록](pr_semanticist21_20261002_review_impl.md#누적-검증-결과)에 연결한다.
원 PR의 exact-head 녹색 CI와 누적 진단 head의 결과는 별개다. 누적 head는 7건을 포함하며
원 PR 또는 최종 수용 그룹의 전체 CI 통과로 간주하지 않는다. 메인터너 source/test 보정은 없다.

원 PR code head: `38c0af21a4370876da2fa34178f25d0ce0a782e0`.
[원 PR](https://github.com/edwardkim/rhwp/pull/7487) · [exact-head Build & Test](https://github.com/edwardkim/rhwp/actions/runs/36896647961/job/110497506699).
CI 집계 실패·진행 중 없음(확인 당시). 원 PR head는 최초 접수 이후 바뀌지 않았다.
Reviewer edwardkim 지정. 원격 GitHub 승인 이벤트는 아직 게시하지 않았다.

## 범위와 호출 경로

관련 #7486의 일부 수정이며 이슈 전체 종료 대상이 아니다.
`b28130e1`과 postmelee의 `45863eb2`를 누적 적용했다. 문서/asset commit `38c0af21`은
코드 진단에서 제외했지만 원 head의 본문 및 archive review를 직접 검토했다.
`stored_line_overflows_body` → 빈 꼬리 문단 흡수 판단 및 끝 빈 쪽 제거 판단으로 이어진다.
줄의 vpos+높이가 본문 높이를 넘는 경우 Hidden/Unadvanced 분기보다 먼저 소유를 보존한다.

## 확인과 보류 근거

- focused 4/4 PASS. 200%/300% 줄 높이에 따른 끝 쪽 소유를 검사한다.
- 기존 보정 commit은 이미 현재 원 head에 포함된다. 과거 archive의 ‘보정 미게시’ 문구를 현재 상태로 쓰지 않는다.
- e509 기준 merge-tree는 `mydocs/orders/20261002.md` add/add 충돌만 발생한다. source 충돌은 없다.
- Enter로 편집한 동일 문서의 독립 한컴 PDF, Native/fresh WASM 대응과 최저 90% 증거는 미검증이다.
  원본 p12 대조나 합성 빈 문단 계약은 이 증거를 대신하지 않는다.
- Studio Enter 캐럿/스크롤 reveal과 표 뒤 Enter의 저장 높이는 원 PR이 밝힌 별도 잔여 범위다.

## 해제 조건과 게시 계획

기여자가 실제 편집 사례의 동일 입력·독립 기준 PDF·시각 게이트를 제출하고 문서 충돌을 해결해야 한다.
원 기여/보정 범위와 남은 범위를 구분한 보류 안내를 작성했다. GitHub 승인·merge는 하지 않았다.

## 승인 후 게시 기록

2026-10-02 작업지시자의 댓글 게시 승인 후 [보류 사유 comment](https://github.com/edwardkim/rhwp/pull/7487#issuecomment-5944034528)를 게시했다.
게시 직전 원 head가 그대로 OPEN임을 확인하고 API 재조회로 한글 본문·BOM/치환 없음 및
작성 문안과의 일치를 확인했다(파일 끝 개행만 정규화). 코드 변경·push·GitHub 승인·merge 없음.
