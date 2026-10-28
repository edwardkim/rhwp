# Stage 213: HWPX/HWP RowBreak CellUnit 원장 진단

## 목적

Issue #3820/#3930의 `2025 행정업무운영 편람(최종)` HWPX/HWP는 같은 저장 셀
프레임과 line segment를 가지지만, section 10 paragraph 4의 `1x1` `RowBreak`
표가 HWPX에서만 tail-only page를 추가한다. 저장용 TABLE raw attribute high bit는
padding으로 재구성되는 값이라 pagination 근거가 될 수 없다.

## 진단 추가

`RHWP_DIAG_CELL_UNITS=<셀 본문 일부>`를 지정하면 최상위 표 셀의 실제 CellUnit
원장을 출력한다.

- compatibility profile
- 표 구조와 page-break 의미
- unit별 높이, 셀 문단/줄 범위
- hard break, stored frame, vpos gap, empty spacer, TopAndBottom flow 표식

이 진단은 출력만 추가하며 pagination이나 render tree를 바꾸지 않는다. 같은
패턴으로 HWPX와 HWP를 실행해 source unit의 첫 차이를 직접 대조한다.

## 실행

```sh
RHWP_DIAG_CELL_UNITS='1. 문서관리 분야' \
  target/release-test/rhwp dump-pages 'samples/2025 행정업무운영 편람(최종).hwpx'
```

HWP 입력에도 같은 명령을 실행한다. 두 원장의 최초 차이를 다음 Stage의
fragment-cut 보정 근거로 사용한다.
