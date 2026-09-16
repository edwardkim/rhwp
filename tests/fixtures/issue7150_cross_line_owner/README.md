# #7150 / PR #7199 줄 경계 소유자 진단

두 파일은 **합성 경계 입력**이다. 한컴에서 저장·재저장하거나 PDF로 변환한 자료가 아니다.
상속된 HWPX 저장 프로그램 메타데이터는 이 합성 변경을 한컴이 생성했다는 증거가 아니다.

기존 Git 입력 `samples/issue2470/36382471_masked.hwpx`에서 첫 쪽 결재표 두 개를
포함한 바깥 셀 문단만 변경했다. 원본을 이름만 바꾼 중복 파일이 아니다.

- 기존 작은 표 `1189058980`과 큰 표 `1189058985`, 사이 tab은 유지했다.
- 작은 표를 복제해 ID `2189058980`, zOrder `4`의 세 번째 표를 큰 표 뒤에 추가했다.
- 문단 마지막에 텍스트 `X`를 추가했다.
- 저장 줄을 추가했다: `textpos=17`, `vertpos=12178`, `vertsize=textheight=11578`,
  `baseline=9841`, `spacing=600`, `horzpos=0`, `horzsize=48760`, `flags=393216`.
  첫 줄의 두 표(8+1+8 UTF-16 단위)와 둘째 줄 표를 구분한다.
- 두 파일 사이에는 **첫 줄 큰 표의 상하 바깥여백만** 다르다.
  `previous_line_margin_140.hwpx`는 top/bottom=140/140 HU,
  `previous_line_margin_240.hwpx`는 240/40 HU다. 여백 합·표 높이·줄 높이와 둘째 줄은 같다.

첫 줄 끝 큰 표와 둘째 줄 시작 작은 표는 가시 문자 위치 1로 투영되지만 원시 줄 소속은 다르다.
`stored_tac_line_assignment`와 실제 `run_tacs`는 첫 줄 `[0, 1]`, 둘째 줄 `[2]`로 구분한다.
다른 줄의 표 여백만 바꿔 둘째 줄 표의 y가 달라지면 소유자 탐색과 실제 배치의 줄 소속이 어긋난다.
이 기대는 특정 PDF 좌표나 새 구현의 수식으로 정하지 않았다.

| 입력 | SHA-256 |
| --- | --- |
| 원본 | `43572dad5e17395aa02d1b0000b736b8467278931086604776ef30393dd0f54b` |
| previous_line_margin_140.hwpx | `3905fd2630b7614899f2389b039fe45620a1bb7bd09b3dea317c2b75a5451904` |
| previous_line_margin_240.hwpx | `d678945bd376aa1015811272ad4702f0bebd25e7d98131f9542f6f7f42e6e9ed` |

재현:

```bash
venv/bin/python mydocs/pr/assets/pr7199_review/check_cross_line_owner.py /path/to/rhwp
```

- devel `6cd3c0692`: 둘째 줄 y=320.0/320.0px, PASS.
- PR #7199 적용 `ca0db01b2`: y=318.7/320.0px, FAIL(exit 1).

원본 실물의 한컴 PDF 시각 대조와 이 합성 불변성 진단은 [리뷰](../../../mydocs/pr/archives/pr_7199_review.md)에서 구분했다.
