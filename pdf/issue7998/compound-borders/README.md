# #7998 다축 stroke의 셀 경계 편집 대조군

`generate.py`는 기존 `../nested-dash-parent.pdf`의 부모 경계 6개를 하나의 `S`로 묶는다.
부모 축은 PDF point 기준 x=20/200/380, y=30/180/330이며 선 굵기는 0.5pt다.
자식 2×2 표·셀 문구와 바깥 본문은 유지한다. 독립 MoveTo마다 dash 위상이 다시 시작한다.

- `compound-solid`: 독립 직선 subpath 6개의 불투명 실선.
- `compound-dash`: 같은 경계의 `[2 2] 0` 점선.
- `compound-alpha`: `[2 2] 1.5`, stroke alpha 0.4.
- `compound-connected-axis`: 한 세로 subpath를 연속된 LineTo 두 개로 나눔. 이번 편집 인증에서 보류한다.
- `compound-interior`: 셀 내부의 장식 선을 같은 source에 추가. 경계만 담은 source로 인증하지 않는다.
- `compound-extension`: 상단 선이 격자 밖으로 연장된다. 원문 paint를 유지한다.
- `compound-joined`: subpath 사이에 대각선 연결을 추가한다. 원문 paint를 유지한다.

재생성: 저장소 루트에서 `python3 pdf/issue7998/compound-borders/generate.py`.
입력은 UTF-8 텍스트 변환 없이 PDF 바이트로 관리한다. 생성 SHA는 `provenance.json`에 있다.

`print/`의 12개 PDF는 각 원본의 첫 셀을 실제 native setter로 편집한 HWP/HWPX를 한컴
2020에서 각각 인쇄한 독립 기준이다. `left`는 왼쪽 변 지우기, `right`는 오른쪽 공유 변을
빨강 실선으로 교체, `all`은 네 변 지우기다. source 형식·해시·job ID·한컴 버전·인쇄 방법은
`provenance.json`에 기록한다. serializer가 만든 PDF를 기준으로 사용하지 않는다.

시각·실제 명령·양 저장과 정식 수집 경로의 회귀 근거는
[stage20 기록](../../../mydocs/working/task_m100_7998_stage20.md)에 연결한다.
편집 owner의 V4 sidecar는 rhwp 왕복 계약이다. 한컴이 비표준 원본 선을 표준 셀 테두리로
편집한다는 뜻이 아니다. 일반 connected join·곡선·부분 병합/중첩 이웃은 이 절편의 완료
범위로 주장하지 않는다.
