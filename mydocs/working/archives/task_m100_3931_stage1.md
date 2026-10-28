# Task M100 #3931 — Stage 1 기준선과 RED 계약

- 날짜: 2026-08-15 KST
- 기준: `upstream/devel` `fbca0aa6c22db9a30e6c417190ae4ddfe924773e`
- 범위: 진단·테스트·문서만, 조판 동작 변경 없음

## 결론

과거 #3927의 "한컴은 16줄을 한 쪽에 약 18.8px pitch로 배치한다"는 판정을 기각한다.
한컴 2020 PDF를 물리 p287~p288에서 직접 대조하면 대상 셀의 첫 문단 12줄은 p287, 둘째 문단
4줄은 p288에 있다. p287 연속 줄의 `yMin` 간격은 약 18.36pt, CSS 96dpi 환산 약 24.48px로
HWP 저장 LINE_SEG의 24.51px pitch와 일치한다.

따라서 `content=392.3px`, padding 포함 `required=422.5px`는 16줄 전체의 정상 측정값이다.
수정 대상은 셀 줄높이가 아니라 저장된 물리 분할을 보지 못하고 표 전체를 새 쪽으로 보내는
declared 이월 경로다.

## 최신 devel 기준선

| 계약 | 현재 결과 |
| --- | --- |
| HWP 전체 | 393쪽 |
| HWPX 전체 | 386쪽 |
| 한컴 2020 KoPub 설치 PDF | 383쪽 |
| sec=10 pi=23 r4/c2 | 저장 16줄(12+4), content 392.3px, required 422.5px |
| 대상 셀 두 문단 소유권 | 둘 다 rhwp page index 291 |
| 정답 소유권 | 첫 문단 p287, 둘째 문단 p288 |

`RHWP_DIAG_ROWH=1 RHWP_DIAG_ROWH_LINES=1`은 과대 선언 대비 실제 요구 높이가 큰 셀에 한해
`pi`, `ci`, 깊이, 문단별 저장 줄 수와 `vpos`, 각 줄의 높이·간격·pitch·최대 글자 크기·문단
줄간격을 기록한다. 환경변수 미설정 시 비용과 출력 변화가 없다.

## 래칫

`tests/issue_3931_declared_rowbreak.rs`가 다음을 고정한다.

- 통과 기준선 3건: 저장 16줄·24.5px pitch, 현재 두 문단의 같은 쪽 소유, 현재 393쪽.
- ignore RED 2건: 두 문단의 인접 쪽 소유, 한컴 2020과 같은 383쪽.

통과 기준선은 3/3 통과했다. 두 RED는 각각 현재 `291 == 291`과 `393 != 383`으로 의도대로
실패했다. 페이지 소유권 검사는 393개 render tree를 순회하지 않고 셀 커서 위치 메타데이터를
직접 조회한다.

## Stage 2 실행 지점

`typeset.rs`에서 native HWP5 다행 RowBreak 표가 빈 host에 있고, 내부 저장 `vpos` reset과
다음 문단 되감김이 있으며, 저장 span 하단이 현재 쪽에 들어가는 경우를 구조적으로 식별한다.
이 조건에서만 flow anchor를 저장 상단으로 재동기화해 기존 fragment scanner가 12+4 물리
조각을 만들게 한다. 전역 줄높이 공식과 tolerance는 변경하지 않는다.
