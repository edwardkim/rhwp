# 실패가 남은 회귀 세 건의 후속 보존

대표 이슈: #7445. 사용자의 2026-10-19 지시에 따라 보정에 성공한 항목은 유지하고,
재검증 후에도 실패가 남은 계약 세 건만 후속으로 분리했다. 출력 결함 해결이나 피델리티
통과로 판정한 것이 아니며, 기존 upstream의 90% 미달을 새로 입증한 것으로 보고하지 않는다.

## 이관 범위

| 하위 이슈 | 활성 경로에서 분리한 검사 | 보존 원문 |
| --- | --- | --- |
| [#8046](https://github.com/edwardkim/rhwp/issues/8046) | `equation_command_goldens_lock_parse_layout_svg` | `equation-command-goldens-before.rs.txt` |
| [#8047](https://github.com/edwardkim/rhwp/issues/8047) | `wmf_emf_goldens_lock_current_engine` | `wmf-emf-goldens-before.rs.txt` |
| [#8048](https://github.com/edwardkim/rhwp/issues/8048) | `embedded_cell_inline_shapes_follow_stored_lines` | `stored-cell-inline-shapes-before.rs.txt` |

이전 source는 `b855468043`이다. 원본 source 파일명은 각 보존 원문의 내용 및 위 이슈에 연결했다.
WMF/EMF 파일은 실행 검사가 한 개뿐이므로 파일 전체를 활성 수집에서 제거했다. 수식 파일의
정직 태그·카탈로그 개수·필수 명령·ID 유일성 검사는 유지한다. 합성 셀 도형 파일에서도 실패한
저장 줄 위치 함수 하나만 분리하고 inline/floating 내용 보존·같은 줄 전진·정렬 계약은 유지한다.
공용 카탈로그·기존 golden·fuzz seed와 다른 입력의 정상 parser·serializer·보안 검사는 유지한다.
검사 수 감소는 정확히 3개이며 `ignore`, 조용한 return이나 공차 완화를 사용하지 않았다.

## 실제 실패와 보정 성공의 구분

전체 Rust 실행 `stage4003`은 10,955개 중 10,949개 통과, 6개 실패, 기존 건너뜀 45개였다.
실패 중 #3930·#7387·#2291은 TextRun 분할을 잘못 읽는 선택자를 보정했고, 같은 세 검사와
대조군을 포함한 집중 `stage4010`의 5개가 모두 통과했다. 이들은 이관하거나 제거하지 않았다.
남은 세 실패만 위 이슈로 연결했다. 이관 뒤 전체 회귀와 lint는 별도 실행 결과로 확인한다.

원 PR #5412·#5458·#5438은 해당 golden을 한컴 정답 오라클이 아닌 당시 엔진 잠금이라고
명시했다. 현재 실제 출력을 정상 golden으로 일괄 갱신하지 않는다. 수식 카탈로그의 미달 영역과
WMF/EMF의 root 크기·행렬·펜 차이를 별도로 보정한다. 합성 저장 줄 입력은 Print 기준을
확보하지 못했으므로 측정 불가를 0%나 90% 통과로 취급하지 않는다.

## 입력·독립 Print 보존

- `metafile-golden-controls.hwpx`와 `metafile-golden-controls-2020.pdf`: 원래 9종 builder의
  바이트를 삽입하고 한컴 2020 MCP Print로 출력한 대조군.
- `metafile-mime-control.hwpx`와 `metafile-mime-control-2020.pdf`: 같은 그림의 MIME 표현을
  바꾼 별도 Print 대조군. 원본과 변경점을 혼동하지 않는다.
- `original-stored-lines.hwpx`: 실패한 합성 저장 줄 검사의 필드를 유지한 입력.
- `original-stored-lines-saved-2020.hwp`: 앞 입력을 한컴으로 저장한 별도 진단 입력.
  원 HWPX 또는 이 HWP의 정상 Print PDF는 확보하지 못했으며 PDF를 만들어 넣지 않았다.

실행 로그·MCP receipt·중간 JSON은 ignored
`output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/`의
`stage3927`·`stage3932`·`stage3934`·`stage3935`·`stage4003`·`stage4010` 계열에 보존한다.
로그와 중간 JSON은 커밋하지 않는다. 아래 파일은 원문과 바이트 동일하다.

## 복원 조건

각 하위 이슈에서 독립 기대값과 정상 Print 기준을 확보하고 공통 구현을 보정한다.
렌더링 회귀의 관련 전쪽 Native/fresh WASM raw 최저 90% 이상·쪽수 일치·직접 확대 판독 뒤
관계 검사로 복원한다. 완료 페이지의 양쪽 98% 조건 없이 절대 px나 전체 SVG 해시로 고정하지
않는다. 이관된 함수는 현재 PR의 해결·검증 완료 범위나 `Closes` 대상에 포함하지 않는다.

## 보존 파일 SHA-256

| 파일 | SHA-256 |
| --- | --- |
| `equation-command-goldens-before.rs.txt` | `d814fbb30d84c95be997ba46c4ef83171dbdb64b1d61fefd17a955b242978a46` |
| `metafile-golden-controls-2020.pdf` | `b8ec430d16e8819c6457799954dee4c98fc87c0daa318929f6da3c96208a8b9e` |
| `metafile-golden-controls.hwpx` | `baa07cb609edacc7138d201151234becb9e7a50ab6ac318a0c9bd56229ebf120` |
| `metafile-mime-control-2020.pdf` | `095763ef5210d61a4a02658bcb8250f405f533cb6a158b0c74517fec4d36e969` |
| `metafile-mime-control.hwpx` | `b91cc0a6012b486575bcbaadf0413d42749ab93f1c712b76ff2aa82bced9de1e` |
| `original-stored-lines-saved-2020.hwp` | `e2c5812f1302d957fa89cf1f14457a175ed34a5429602ef8f2ffcb9fcc7fca1e` |
| `original-stored-lines.hwpx` | `ac18790848382ace4f6e8fbcdbdb01f37cf1d889a1bcf4787d909d6561cc7844` |
| `stored-cell-inline-shapes-before.rs.txt` | `4fb45287cf8af2dabfc14ad1d22b99407b363e53e91516ead1f7991b393ae979` |
| `wmf-emf-goldens-before.rs.txt` | `cd8a0ed65a77ab821436ec3458a0a75ab56fe45094351108fe2b4ea977db8730` |
