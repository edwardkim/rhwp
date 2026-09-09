# pdf/issue5652 — #5652 B2 엔진 판정 한컴 2022 변환본

[#5652](https://github.com/edwardkim/rhwp/issues/5652) B2-엔진 본구현의 한컴 판정 PDF 32건이다.
원본(판정 입력)은 [`samples/issue5652/`](../../samples/issue5652/), 판정 원장은
[`samples/issue5652/MANIFEST.json`](../../samples/issue5652/MANIFEST.json), 결론은
[`mydocs/report/task_m100_5652_report.md`](../../mydocs/report/archives/task_m100_5652_report.md) 다.

## 이 자료가 증명하는 것

> 엔진이 `structure:true` 로 만든 행·열·라벨 구조 편집 산출(12변종 × 2포맷 + 변환본)을 한글 2022 가
> 오류 없이 열고, **13 판정 단위 전건이 대조군과 다르게(= 편집대로) 그려지며**, 같은 변종의 `.hwp` 와
> `.hwpx` 는 **12/12 픽셀 동일**, HWPX→HWP 변환본도 원본 HWP 편집본과 픽셀 동일이다.
>
> 그리고 **25/25 가 [#5447](../issue5447/) 스파이크 판정 PDF 와 픽셀 동일**이다 — 엔진이 스파이크와
> 같은 차트 XML 을 만들었고(10변종), 위치 기반 계열삭제(2변종)도 한컴이 같은 그림으로 그린다.

파일명은 `<기준문서>-<변종>-<hwp|hwpx>-2022.pdf`. 래스터 해시는 원장이 정본이고 여기 복사하지 않는다.

## 이 자료에 없는 것

경계 2종(원형 계열추가·주식형 계열삭제)은 엔진이 거부하므로 변환본이 없다 — 한컴이 그것을 어떻게
잘못 그리는지는 [`pdf/issue5447/`](../issue5447/) 가 보여 준다. 편집기 행·열 수(판정 지시표 (d))는
래스터로 잴 수 없어 원장의 `editor_observation` 에 사람이 적는다.
