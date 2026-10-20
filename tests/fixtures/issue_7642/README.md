# 상대 크기 그림의 한컴 정상 저장 반례

`samples/p122.hwp`의 진단 사본에서 저장 LineSeg를 제거하고 그림 너비를 절대값으로,
높이를 용지 25%로 바꿨다. 그림 앞·뒤 문단의 명시적 쪽 나눔을 설정하고,
`affect_line_spacing` false/true를 각각 한컴 2020 MCP로 정상 저장한 뒤 같은 저장본을 Print했다.
원본 p122는 수정하지 않았다. 정상 저장본의 상대 높이 선언은 2500, 현재 높이와 저장 줄 높이는
21047HU다. 비율을 HWPUNIT으로 잘못 사용하는 보정을 검출하는 대조군이다.

Native/fresh WASM 각각 두 입력의 3쪽 전체가 100%이며, before/after review를 직접 판독했다.
이 증거는 저장 조판 경로의 계약이며, 수동 NO_LS 입력의 재조판 통과를 대신하지 않는다.

| 입력 | Print PDF | Save job | Print job | 입력 SHA-256 | PDF SHA-256 |
| --- | --- | --- | --- | --- | --- |
| [relative-quarter-affect-false-hancom-2020.hwp](relative-quarter-affect-false-hancom-2020.hwp) | [relative-quarter-affect-false-hancom-2020.pdf](relative-quarter-affect-false-hancom-2020.pdf) | f42dce62-4ce2-4448-a325-ffc58591ea63 | 152a67c9-534c-4711-b455-a129a8f0fa4f | `635be3d491c4e1b27db74c788ad1449ac82c4c704121f7bfc00e6b42579f0254` | `b98261c943b4848ea79b67ade090ec8883d6a8131b58fa43f004f9d90ae87865` |
| [relative-quarter-affect-true-hancom-2020.hwp](relative-quarter-affect-true-hancom-2020.hwp) | [relative-quarter-affect-true-hancom-2020.pdf](relative-quarter-affect-true-hancom-2020.pdf) | 1e705155-5427-41e3-b0c4-483965785a86 | de33c20d-dfb8-4b7e-8dd2-0c2b67eaa4ac | `c36df27fe8e4a076c3aae737619e9c5743a80344ee3a823f7c3cc4b3a399315e` | `0ba5e08ef0a59c49877d1053d60480a2a0c6aeb055b31aef4f5b68ef2c6a4e19` |
