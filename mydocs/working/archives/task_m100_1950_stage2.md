# #1950 Stage 2 — 유의 결함(2955331) 정밀 원인 + 수정 설계

- 브랜치: `local/task1950` / 범위: 2955331(탭 376px) 단독 (razor-thin 6건 분리 승인됨)

## 1. 근본 원인 — char 위치 **단위 불일치**(logical 1-unit vs code-unit 8-unit)

`char_offsets`/char_shape `start_pos` 의 단위가 **소스 파서에 따라 다르다**:

- **HWP5 파서**: `char_offsets` = UTF-16 **code-unit 위치**(`code_unit_pos = pos/2`).
  탭은 8 code-unit(0x0009 + 확장 7). char_shape `start_pos` 도 code-unit 기준.
  (`src/parser/body_text.rs:275`)
- **HWP3 파서**: char_offsets/char_shape 위치를 **1-unit(logical, 탭=1)** 로 산출.
  ir-diff 실측: `char_offsets[17] A(원본)=17 vs B(변환)=24` — 원본은 탭=1, 변환·재파스는 탭=8.

**직렬화기**(`serialize_para_text`)는 탭을 **항상 8 code-unit 으로 확장**하고, char_shape
`start_pos` 는 **IR 값 그대로**(`serialize_para_char_shape`, body_text.rs:282) 쓴다.

→ HWP5-origin IR(code-unit)은 텍스트(8-unit)와 정합. 그러나 **HWP3-origin IR(1-unit)** 은
char_shape `start_pos=31`(logical)을 8-unit 텍스트(cc=88)에 그대로 써서, **char_shape[1]
(id=155, 자간 0%)이 code-unit 31(탭 확장 중간)부터 적용**된다. 원래 자간 −5%(id=154)여야 할
탭 run 이 자간 0% 로 바뀌어 **탭 폭이 달라지고 run 이 3+1 로 쪼개져 376px 변위**.

## 2. 원본 렌더가 정답인 이유

원본 HWP3 IR(1-unit)은 렌더가 char_offsets(1-unit)·char_shape(1-unit)로 **자기정합** →
탭 4개 한 run, 자간 −5% 정상. 변환본만 단위가 어긋나 이탈. → **IR 단위를 통일**하면 해소.

## 3. 수정 설계 (Stage 3 대상)

**핵심**: IR 의 char 위치 단위를 HWP5 시멘틱(code-unit, 탭=8)으로 **통일**한다.

- **1차안(권장, CLAUDE.md 정합)**: **HWP3 파서가 char_offsets·char_shape `start_pos` 를
  code-unit 위치로 산출**(탭 등 다중유닛 컨트롤 확장 반영). → IR 이 HWP5 와 동일 단위 →
  직렬화·재파스 정합, 렌더는 소스 불문 동일 단위라 원본 렌더 불변. 수정 위치 `parser/hwp3`.
- 대안(비권장): 직렬화기가 HWP3-origin 한정 logical→code-unit 변환. 소스 플래그 의존 +
  공통 직렬화기에 HWP3 분기 → CLAUDE.md 위배 소지.

## 4. 검증 계획 (Stage 3)

- 2955331 render-diff --via hwp PASS(±1px, 탭 run 4개 유지, 자간 −5%).
- **원본 HWP3 렌더 불변**(char 단위 변경이 원본 렌더를 바꾸지 않아야 함) — export-render-tree
  before/after 동일성.
- hwp3 표본 회귀(탭 포함 문서 광역): `hwp5-roundtrip`·`render-diff --via hwp` 게이트.
- hwp5/hwpx 무회귀(이들은 이미 code-unit 이라 불변 기대).

## 5. 리스크

- **HWP3 파서 char 단위 변경은 HWP3 렌더 전반에 영향** — 탭 보유 HWP3 문서 다수 회귀 위험.
  원본 렌더 불변을 엄격 게이트(export-render-tree 동일성)로 보증.
- 탭 외 다중유닛 컨트롤(개체/필드)도 HWP3 파서에서 동일 단위 처리 필요할 수 있음 — Stage 3
  에서 탭 우선, 여타는 회귀 발견 시 확장.

→ Stage 3(HWP3 파서 char 위치 code-unit 통일 + 게이트) 진행 승인 요청.
**주의**: HWP3 char 모델 변경은 회귀면이 넓어 앞의 두 이슈보다 리스크가 크다.
