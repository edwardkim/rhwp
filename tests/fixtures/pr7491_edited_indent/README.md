# PR #7491 편집 후 독립 출력 입력

기존 public sample과 빈 문서를 실제 DocumentCore 편집 API로 수정한 저장본이다.
LineSeg·문단 속성을 수동 작성하지 않는다. 생성기는
[generate_inputs.rs](../../../mydocs/pr/assets/pr7491/generate_inputs.rs)다.

- typed-flat/first/hanging: 새 문서에 동일 본문을 입력하고 indent 0/3000/-3000 적용.
- 6190-edited: 원본 문단 4 끝과 문단 7 앞에 각각 `가` 입력 후 저장. 저장 보정이 기본 1단 정의를 발행한다.
- 6190-edited-one-column: 같은 편집 뒤 실제 `set_column_def_native(0, 1, 0, true, 0)`를 적용한 독립 대조군.
- 6190-explicit-break-one-column: 원본 문단 7 앞에 `가\n` 입력, 실제 1단 명령 적용 후 저장.
- prefix-spaces/blank-break/punctuation-break: 원본 문단 7 앞에 각각 공백 두 칸/명시적 개행/`.\n`을 실제 입력 후 저장.
- biz-edited: samples/biz_plan.hwp 문단 51 끝(67)에 `가` 입력.

원본 #6190은 [기존 sample](../../../samples/issue6190/center_align_first_line_indent.hwp)을 사용한다.
각 HWP와 같은 이름의 `-2020.pdf`는 해당 HWP의 동일 바이트를 한컴 MCP engine 2020으로 변환한 독립 기준이다.
입력/출력 해시는 [input-provenance.json](../../../mydocs/pr/assets/pr7491/input-provenance.json)에 고정한다.

## 보정 전 실패 증거

`6190-edited-without-column`은 저장 보정 전 `4060db2d9d46b3ccd2987538286a11f836348056`의 실제 편집 저장본이다.
원본과 이 저장본에는 본문 ColumnDef가 없다. 저장 좌우 여백은 7086HU(25mm)이지만,
한컴 출력에서는 8504HU(30mm)가 관측된다. `6190-original-2020.pdf`와 원본을 한컴에서
다시 저장한 `6190-original-hancom-normalized.hwpx`는 편집 전에도 같은 차이가 있음을 보존한다.
이 출력과 rhwp 25mm 조판의 실패를 통과로 바꾸어 보고하지 않는다.

실제 1단 명령 대조군은 원래 25mm를 한컴도 유지한다. 이 관측을 근거로 저장기가
재구성하는 구역에 기본 1단을 명시하며, 최종 `6190-edited.hwp`의 한컴 출력으로 직접 재검증한다.
수정하지 않은 원본의 raw 스트림은 그대로 보존하고, 내보내기는 live 모델을 바꾸지 않는다.

## 추가 페이지 경계 계약

`growth-8` / `growth-20`은 public `samples/issue6882/synth_cell_enter_table_growth.hwp`의
31번 셀 마지막 문단에서 실제 Enter 명령을 각각 8/20회 실행한 저장본이다. 대응 한컴
2020 PDF는 3/4쪽이며, 8회 표의 2쪽 상단은 52.383pt(69.844px)다.
정식 회귀는 **live 편집**에서 prefix의 중복/누락과 표의 다음 쪽 원점·본문 끝을 확인한다.
이 저장본을 rhwp에서 재개방할 때 셀 성장 높이가 유지되지 않는 기존 문제와, 20회 입력의
표 높이·후속 내용 차이는 해결 주장에 포함하지 않는다. 전체 페이지 시각 통과 자료가 아니다.
