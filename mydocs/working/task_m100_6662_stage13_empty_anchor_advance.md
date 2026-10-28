---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_6662_stage13_empty_anchor_advance.md
last_verified: 2026-09-06
---

# 열린 이슈 재검증 13단계: 빈 그림 앵커의 실제 줄 전진

Issue: #6712. 시작 HEAD: `efdb066d0`.

## 분석과 계획

- 한국어 p8은 빈 Square 그림 앵커지만 저장 vpos=10679, lh=1000, ls=600이며
  다음 p9의 vpos=12279이다. 그림 전체 높이를 예약하지 않는 것과 이 한 줄의
  1600 HU advance를 제거하는 것은 다르다.
- 중국어의 대응 앵커도 lh+ls와 다음 vpos 차이를 대조한다. 실제 단일 저장 줄,
  단일 Square Picture, 인접 wrap text, 정확한 다음 줄 사다리가 증명된 경우만
  줄 전진을 보존한다. synthetic/좌표 reset/다른 높이는 일반 guide 처리로 남긴다.
- 공통 판정을 측정, cell unit, 일반/partial 렌더에 적용한다. nonempty 그림 문단의
  텍스트 높이를 빈 앵커처럼 버리지 않는다. 그림 높이 자체의 중복 예약은 복구하지 않는다.
- 실물 소스 좌표 관계 테스트를 red/green으로 확인하고 83개 대조군 및 sweep을 재실행한다.
  baseline 완화는 하지 않으며 실패가 발생하면 결과와 다음 조치를 명시한다.

로그 및 중간 산출물은 `/tmp`만 사용한다. 이 단계는 #6712 전체 해결 선언이 아니다.

## 수정 전 재현

- 최초 suite 선택이 잘못되어 0 tests, exit 4였다. 이를 통과로 세지 않는다.
  생성된 manifest에서 #6712의 실제 target인 `regression_suite_019`를 확인했다.
- 정정 후 1 test failed, 0.035초, exit 100. 한국어 p8→p9의 실제 y 차이는 0,
  저장 사다리는 1600 HU였다. 기존 두 문서 트리에서도 이 위치 중복을 확인했다.
- 정상 문단처럼 텍스트를 가진 그림 앵커까지 빈 줄 높이 0으로 처리하던 측정 조건을
  빈 문단으로 한정하고, 단일 빈 앵커는 다음 저장 vpos가 정확히 lh+ls만큼 증가할 때
  그 줄 전진만 보존한다. 그림 높이는 별도 더하지 않는다.

## 검증과 판정

- focused 및 overlap/overflow/off-canvas 16개 partition 대조군: 84 passed,
  997 filtered/skipped, 89.528초, exit 0.
- 이후 HWPX에는 기존 unit 소유권을 유지하도록 profile guard를 보완했다.
  최종 코드 전체 nextest: 9,086 passed, 46 skipped, 500.321초, exit 0.
  빌드 5분 37초는 test summary 시간에 포함되지 않는다.
- 전체 실행 명령: `cargo nextest run --locked --cargo-profile release-test
  --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`.
- 한국어 p8/p9 1600 HU, 중국어 p40/p41 1280 HU의 실제 줄 전진을 실물 계약으로 확인했다.
  두 문서 모두 기준 PDF와 같은 2쪽이다. baseline 완화는 없다.
- 현재 CLI SHA-256:
  `102e8da5337531e9a790aea72c467e9b68d58e15e125c4b8bcd7c7d2e8e95d15`.
  12단계에서 수정한 Chrome viewport/font 처리를 적용해 각 문서 1~2쪽 sweep을 새로 산출했다.
- 한국어 p1 일부 관리 문단은 기준보다 약 9.9px, 끝의 예방 문단은 약 26.9px 위에 남는다.
  p2 보라색 표의 텍스트는 약 6.7px 아래다. 중국어 p2는 하단 테두리가 footer 글자와 교차한다.
  평균 오차나 페이지 수 일치만으로 이를 통과로 처리하지 않는다.
- 따라서 이 단계의 빈 앵커 보정은 통과하지만 #6712 전체 해결 판정은 보류한다.
  다음 단계에서 남은 줄 전진/테두리 소유권을 별도로 분석한다.
- 로그, audit JSON, sweep 중간 파일은 `/tmp`에만 유지하며 커밋하지 않는다.
  PR 직전 필수 lint 묶음은 아직 수행하지 않았으며 이 기록은 push 승인 근거가 아니다.
