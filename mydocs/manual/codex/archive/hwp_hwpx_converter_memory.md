---
kind: snapshot
status: historical
canonical: mydocs/manual/codex/README.md
last_verified: 2026-07-17
---

# HWPX To HWP Converter Memory

## Main Lesson

HWPX 출처 IR을 HWP 직렬화기에 그대로 넣는 단순 어댑터 접근은 한계가 있다.

필요한 것은 단순 필드 보강이 아니라 HWP5 저장에 적합한 구조를 만들어 내는 본격 변환기다.

## Current Problem Boundary

작업지시자의 현재 판단:

- HWPX 파일 파싱은 문제가 아니다.
- HWPX를 IR로 매핑하는 것도 문제가 아니다.
- rhwp-studio에서 IR을 렌더링하는 쪽도 문제가 아니다.
- 문제는 clone해서 HWP로 저장할 때 빠지거나 잘못 매핑되는 것이다.

즉, 다음 조사는 `parser`나 `renderer`가 아니라 저장용 materialization 경로에 집중해야 한다.

## HWPX vs HWP Difference Memory

HWPX와 HWP는 동일 의미를 담더라도 저장 구조와 ordinal이 다를 수 있다.

예: 배경 무늬 패턴

- HWP 쪽 값과 HWPX 쪽 값의 기수가 다를 수 있다.
- 이전 분석에서 HWP `-1` 의미와 HWPX `0` 의미를 혼동할 수 있음이 드러났다.
- 한컴 에디터에서 문단 > 배경 > 무늬모양이 2번째로 잡히는 문제가 있었고, 0번째로 맞추면서 렌더링이 정상화되었다.
- 0번째가 "무늬없음"인지 "빈무늬"인지는 스펙과 한컴 도움말 교차 확인이 필요했다.

## Stage 4 Materialization Memory

Task #854 Stage 4에서 중요했던 materialization:

- HWPX-origin 표에 대해 HWP5 `TABLE` record 말미의 `borderFillID + nZones`를 materialize
- 표 셀 `LIST_HEADER`에 HWP5 확장부를 materialize
- `expense_report.hwpx`의 쪽 배경 관련 `PAGE_BORDER_FILL`를 맞춤

결과:

- 한컴 에디터 파일 손상 판정이 사라짐
- 이후 표 배치 비정상도 Stage 4 구현으로 해결됨

## Avoid Repeating Failed Probe Pattern

Stage 6에서 실패한 접근:

- 증상 주변을 보고 여러 곳을 임의로 수정
- header/footer raw `LIST_HEADER`
- 첫 문단 control/text order
- NewNumber control code
- cell width/ref 관련 재시도
- fixed width space code
- line seg trap 의심

이런 시도들은 사용자 검증에서 계속 2페이지 파일 손상을 해결하지 못했다.

다음 재시도는 "무엇을 고칠지"보다 먼저 "POC 성공물과 현재 실패물의 구조 차이"를 증거로 잡아야 한다.

## External Mapping Reference

권위 있는 매핑 참고 자료:

```text
/home/edward/vsworks/hwp2hwpx
```

메모:

- Java 기반 `hwplib` / `hwpxlib`
- dogfoot/neolord0 계열
- Apache 2.0
- 직접 포팅 대상이 아니라 매핑 명세 참고 자료로 사용
