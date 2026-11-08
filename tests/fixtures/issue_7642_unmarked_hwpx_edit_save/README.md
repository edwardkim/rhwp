# 마커 없는 HWPX 구역 첫 문단 편집 저장 입력

`wide_tac_table.hwpx`는 커밋된 `samples/tac-host-spacing.hwpx`의 합성 변형이다.
ZIP 항목 중 `Contents/section0.xml`만 바꿨고 나머지 항목의 내용은 원본과 같다.

- 구역 첫 문단의 `hp:bookmark name="fixture-anchor"`를 지웠다.
- 1×1 글자처럼 취급 표의 `hp:sz`·`hp:cellSz` 폭을 20000에서 47000으로,
  셀 문단 `hp:lineseg`의 `horzsize`를 19000에서 46718로 바꿨다.
- rhwp 생산자 마커(`META-INF/rhwp-hwp5-origin`, `META-INF/rhwp-hwp3-origin`)는 없다.

구역 첫 문단은 구역 정의·`hp:ctrl` 단 정의·넓은 글자처럼 취급 표로 시작하므로 맨 앞에
글자를 넣으면 [글자 줄, 표 줄] 두 줄이 된다.

| 파일 | 바이트 | SHA-256 |
| --- | --- | --- |
| `wide_tac_table.hwpx` | 4,163 | `a1bc3589f6add0676766388c092608037062161b3ffe5ff00d0bf57982042ce1` |
| 원본 `samples/tac-host-spacing.hwpx` | 4,183 | `a467f038345ab5aacbe3efac39fdf5d08018bc019249af64866b55ae4b82badd` |

이 합성 파일에 독립 한컴 정본이 있다고 주장하지 않는다.
`tests/cases/issue_7642_unmarked_hwpx_edit_save_line_starts.rs`는 렌더 출력이 아니라
메모리 안의 편집 결과와 저장본을 다시 연 모델의 줄 시작·글자만 비교한다.
