# Task M100-2439 Stage 5 — PDF 오라클 정정과 잔여 차이 분석

- 이슈: [#2439](https://github.com/edwardkim/rhwp/issues/2439)
- 브랜치: `fix/2439-split-table-flow`
- 작성일: 2026-07-20
- 재현 HWP SHA-256: `674eabe66ea0ba783ad2cd398519c9893ba94956a22d9cb94b084db00d4d2c3d`
- 정답 PDF SHA-256: `f36a747c5f848d90e755abe2e730d932429a324fad9b7e822964934cd8f8eca4`

## 1. 오라클 재확인

정답지는 한컴 2024에서 가로 방향으로 인쇄한 `Microsoft: Print To PDF` 출력이다.

- A4 landscape 841.92×595.32pt
- 총 10쪽
- 10쪽 텍스트: `5.응급 및 긴급한 상황시 7920으로 연락한다.`

Stage 4의 “10쪽은 빈 페이지” 판정은 잘못됐다. 1차 rhwp 9쪽 출력은 마지막 문장을
누락한 것이 아니라 한컴보다 앞 페이지로 끌어올린 것이며, 문서 전체 pagination이 아직
일치하지 않은 상태였다.

## 2. 페이지별 기준 흐름

| PDF 쪽 | 기준 내용 |
|---|---|
| 1 | `02-0147` 양식. 서명란은 다음 쪽으로 이동 |
| 2 | 앞 양식 서명란, `02-0202` 양식. 마지막 우측 비고 셀의 마지막 줄은 다음 쪽으로 이동 |
| 3 | `체크 권장` continuation, 서명란, `02-0183(2)` 양식. 마지막 `드롭센서 기능 체크` 행은 다음 쪽으로 이동 |
| 4 | `드롭센서 기능 체크` continuation, 서명란, `18-0042` 양식과 안내문 3번 본문 |
| 5 | 3번 continuation과 4·5번, `07-0270` 양식 및 다음 안내문 continuation |
| 6 | 앞 안내문 4·5번, `18-0902` 양식 및 다음 안내문 continuation |
| 7 | 앞 안내문 4·5번, `03-0279` 양식 및 안내문 4번까지 |
| 8 | 앞 안내문 5번, `24-0004` 양식 및 안내문 4번까지 |
| 9 | 앞 안내문 5번, `07-0264` 양식 및 안내문 4번까지 |
| 10 | 마지막 안내문 5번 한 줄 |

따라서 양식 코드의 출현 순서만 비교해서는 안 되고, 표 fragment와 안내문 continuation의
페이지 귀속까지 함께 확인해야 한다.

## 3. 잔여 원인

### 단일 positive-offset 빈 host RowBreak 표

표가 실제로 그려진 하단과 outer bottom, 저장 LineSeg 진행량이 다음 일반 문단 fit에
충분히 반영되지 않았다. 이 때문에 마지막 행·서명문·안내문이 한컴보다 앞쪽 페이지에
남았다. 구조 조건이 일치하는 native HWP5 경로에서 painted bottom을 엄격한 fragment
fit 기준으로 사용해야 한다.

### native HWP5 두 표 visible host

같은 visible host의 zero-offset/positive-offset 두 표에서 두 번째 표의 outer top/bottom과
host LineSeg 간격이 순차 flow에서 빠졌다. 두 표와 뒤의 서명문을 typeset/layout 양쪽에서
같은 하단까지 소비해야 한다.

### native HWP5 저장 LineSeg 들여쓰기

번호 줄글은 문단 스타일 margin보다 큰 저장 `LineSeg.column_start`를 사용한다.
정답지와 HWP 저장값의 대응은 다음과 같다.

- 제목 줄: `1900HU` → 63.09px ≈ 47.32pt
- 번호 줄글: `10320HU` → 175.36px = 131.52pt

비합성 full-width 일반 본문 줄만 저장 시작점을 권위 값으로 사용하고, 표 셀,
wrap/control, 합성 LineSeg, HWP3/HWPX는 제외해야 한다.

## 4. 완료 기준 갱신

- 하드코딩 없이 `dump-pages` 10쪽
- 표 fragment와 서명·안내문 continuation의 페이지 귀속 정합
- 표 하단과 텍스트 겹침 없음
- 번호 줄글 들여쓰기 정합
- 1~10쪽 visual sweep과 사람 직접 review

구현과 최종 결과는 [Stage 6](task_m100_2439_stage6.md)에 기록한다.
