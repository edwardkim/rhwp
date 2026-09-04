# Task #2809 Stage 0 — 나눔정렬 재현과 원인 계측

- 브랜치: `task/2809-distribute-align`
- 기준: `upstream/devel` @ `58991a768`
- 입력: 이슈 첨부 `jubo_20260104.hwp`
- 상태: 완료

## 1. 재현

- 문서 파싱 및 6쪽 렌더 성공.
- 문제 문단 예: section 0의 표 셀 `row=6, col=10, colSpan=2`, 텍스트
  `다 같 이`, `paraPrIDRef=6`.
- HWPX 진단 변환에서 셀 크기 `6980 HU`, 셀 안 여백 좌우 각 `141 HU`, 저장
  `LINE_SEG.horzpos=0`, `horzsize=6972 HU`로 확인했다.
- ParaShape 6은 HWP5 값 5, HWPX `DISTRIBUTE_SPACE`, IR `Alignment::Split`이다.
  좌우 문단 여백과 들여쓰기는 모두 `0 HU`다.

## 2. 기준 증적

- HWP 2020 기준 PDF: 6쪽, SHA-256
  `a73d50620bf8fe96beaff72ba0e40cd34f396ec75de9798ac1fd0402e28f8e2b`.
- 수정 전 rhwp 2쪽 SVG:
  - 반복 라벨 셀 clip 예: `x=415.6533`, `width=95.3867px`.
  - 라벨 `다/같/이` x 예: `417.5333 / 439.9333 / 462.3333px`.
  - 다른 반복부: `416.2533 / 427.4533 / 438.6533px`.
- HWP 2020 PDF에서는 같은 라벨의 세 글자가 셀 좌우에 걸쳐 분산된다.

## 3. 가설 판정

| 가설 | 판정 | 근거 |
|---|---|---|
| HWP 정렬 값 파싱 누락 | 기각 | 값 5가 `DISTRIBUTE_SPACE`/`Alignment::Split`로 보존됨 |
| 문단 좌우 여백으로 목표 폭 축소 | 기각 | left/right/intent 전부 0 HU |
| 셀 안 여백 과다 | 기각 | 좌우 141 HU, `available_width=91.627~92.907px`로 정상 |
| `condense=30` 전체 폭 축소 | 기각 | 실제 가용폭이 셀 inner 폭과 일치 |
| `Split` 마지막 줄 분배 억제 | **근본 원인** | `needs_justify`가 `Split`을 `Justify`와 묶어 마지막 줄에서 false 처리 |
| 반복부 간격 차이 | 설명 완료 | 측정폭이 30px와 60px로 달랐으나 둘 다 여분 공백 0인 동일 결함 |

## 4. 확정 계측

- `다 같 이`: `Split`, cell=true, `available_width=92.907px`,
  `total_text_width=30.000px`, chars=5, overflow 억제=false.
- 후반 반복부: `available_width=91.627px`, `total_text_width=60.000px`.
- 두 경우 모두 수정 전 마지막 줄 판정 때문에 공백 여분이 0이었다.
- 임시 `RHWP_DIAG_DISTRIBUTE` 출력은 원인 확정 후 소스에서 제거했다.
