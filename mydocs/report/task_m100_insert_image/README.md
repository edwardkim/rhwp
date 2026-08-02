---
kind: report
status: active
canonical: mydocs/report/task_m100_insert_image/README.md
last_verified: 2026-08-02
---

# `edit insert-image` 처리 기록 — 도장·서명 삽입 (로드맵 #3719 §6-5)

## 문제

에이전트가 실물 서식을 끝까지 채워도 **직인·서명을 얹을 표면이 없었다.** 누름틀
(`edit fill-fields`)·표 칸(`edit set-cell`)·체크박스는 모두 CLI 로 닿는데, 도장 하나
때문에 사람이 한컴을 열어야 했고 그 순간 자동화 사슬이 끊긴다. 제출용 문서에서
도장은 장식이 아니라 **접수 요건**이다.

코어에는 이미 검증된 그림 삽입 경로(`insert_picture_native`, studio 가 쓰는 것)가
있었으나 브라우저 밖에서 부를 방법이 없었다.

## 구현

새 삽입 로직을 만들지 않았다. 기존 코어를 그대로 부르고, 인자 파싱·저장·봉투·
`--verify`·`changedPages` 는 형제 명령 `edit set-cell`(#3381) 의 형태를 따랐다.

### 명령

```
rhwp edit insert-image <파일> --image <경로> [--page N] [--x N --y N]
       [--width N --height N] [-o <경로>] [--dry-run] [--verify] [--json]
```

봉투: `{schemaVersion, source, image, page, x, y, width, height, binDataId,
dryRun, overflow, changedPages, output, outputFormat, verify}`
(`output`·`outputFormat`·`verify` 는 실행했을 때만 — dry-run 은 산출이 없다.)

### 재사용한 코어

| 쓰임 | 함수 | 위치 |
| --- | --- | --- |
| 그림 삽입 | `DocumentCore::insert_picture_native` | `src/document_core/commands/object_ops/picture.rs:1433` |
| 쪽 → 앵커 문단 | `DocumentCore::dump_page_items_json` | `src/document_core/queries/rendering.rs:4907` |
| 눈검증 대상 쪽 | `DocumentCore::pages_covering_paragraphs` | `src/document_core/queries/changed_pages.rs:18` |
| 산출 형식 결정 | `edit_output_format` (#3383) | `src/main.rs` |
| 직렬화 | `edit_serialize` → `export_hwp_with_adapter` / `export_hwpx_native` | `src/main.rs` |
| 저장 후 자기검증 | `edit_verify_report` (#3702) → `diff_documents` | `src/main.rs` |

`insert_picture_native` 의 **본문 floating 분기**(용지 기준 offset,
`treat_as_char=false`, `TextWrap::Square`)를 쓴다. 이는 한컴 native 의 그림 삽입
기본값과 같고(사용자 시연 2026-05-30 근거로 #1151 v9 에서 맞춘 동작), 도장처럼
"본문 흐름과 무관하게 지정한 자리에 얹는" 개체에 정확히 맞는 배치다.

### 길이 단위 — HWPUNIT (이 명령의 최대 함정)

`--x/--y/--width/--height` 는 전부 **HWPUNIT(1/7200 inch)** 이며 픽셀이 아니다.
A4 세로 = 59528 × 84188 HWPUNIT. px 로 오해하면 도장이 1/75 크기로 찍히는데
**종료 코드는 0** 이라 에이전트는 성공으로 읽는다. 그래서 단위를 세 곳에 못 박았다:

- `--help` 본문 (`길이 단위는 모두 **HWPUNIT(1/7200 inch)** — 픽셀이 아니다`)
- MCP 도구 description 과 각 속성 description
- 인자 오류 문구 자체 (`--x 뒤에 0 이상의 정수가 필요합니다 (HWPUNIT, 1/7200 inch)`)

크기 규약(실측 §2·§3):

| 지정 | 결과 |
| --- | --- |
| 둘 다 생략 | 원본 픽셀 × 75 (96dpi 환산). 40×20px → 3000×1500 |
| 한쪽만 | 원본 비율 유지. 40×20px 에 `--width 8000` → 8000×4000 |
| 둘 다 | 그대로 |

어느 경우든 **최종 값을 봉투에 실어** 조용한 보정이 없게 했다.

### 쪽 지정(앵커) 방식

용지 기준 floating 그림은 **앵커 문단이 놓인 쪽**에 그려진다. 그래서 "몇 쪽" 을
"어느 문단" 으로 옮겨야 하는데, 그 환산은 이미 조판 결과가 알고 있다 — 새 조판
로직을 짜지 않고 진단 질의 `dump_page_items_json(Some(page))` 를 읽어 그 쪽의 첫
본문 항목을 앵커로 고른다. 미주(`isEndnote`)는 구역 뒤에 합성된 문단이라 제외하고,
항목이 하나도 없는 쪽(어울림 문단·감춘 빈 줄만 귀속된 쪽)은 `extras` 에서 찾는다.

검증은 선언이 아니라 결과로 한다 — `--page N` 이 `changedPages` 에 N 을 담는지
전 쪽(0·1·2) 순회로 확인한다(실측 §13: `[0]` `[1]` `[2]`).

### 쪽 밖 배치는 조용히 자르지 않는다

에이전트는 렌더 결과를 보지 않는다. 신호가 없으면 쪽 밖으로 나간 도장을 완성본으로
판단한다. 그래서 **자르지 않고**(요청 좌표 그대로 넣고) `overflow` 로 숫자를 보고한다:

```json
{"page":0,"paperWidthHu":59528,"paperHeightHu":84188,
 "rightHu":59529,"bottomHu":84189,"overflowXHu":1,"overflowYHu":1}
```

용지 크기는 `PageDef` 에서 읽되 `landscape` 면 가로·세로를 바꿔 쓴다
(`page_layout.rs:69` 와 같은 규칙). 경계에 **정확히 닿는** 배치는 넘침이 아니다
(실측 §6) — 판정이 과민하면 정상 배치마다 거짓 경보가 뜨고, 그러면 에이전트는
경보 자체를 무시하게 된다.

### 실패 경계 (종료 코드 사전 #2707 준수)

| 상황 | 코드 | 근거 |
| --- | --- | --- |
| 지원 안 하는 형식(`.svg` 등) | **2** | 인자 문제이지 런타임 실패가 아니다 |
| 확장자는 맞는데 내용이 그림이 아님 | **2** | 매직바이트 재판정 — 크기를 못 재면 좌표가 무의미 |
| 쪽 번호 범위 초과 | **2** | 형제 명령(`dump-pages`)과 같은 규약 |
| 음수·소수·`3000px` 같은 값 | **2** | 코어가 음수를 0 으로 깎으므로 **조용한 보정 대신** 끊는다 |
| `--width 0` / `--height 0` | **2** | 크기 0 은 그림이 아니다 |
| 알 수 없는 옵션·입력 파일 2개·값 없는 플래그 | **2** | #3349 파싱 규약 |
| 그림 파일 없음 / 문서 없음 | **1** | 인자 형태는 맞다 — 런타임 실패 |
| `--verify` IR 차이 | **3** | #3702 규약 |

실패 경로의 **stdout 은 전부 0바이트**다(실측 §8~§12). 실패 시 산출물도 만들지 않는다.

지원 형식은 `png · jpg · jpeg · bmp · tif · tiff` — BinData 로 넣을 수 있으면서
**원본 픽셀 크기를 헤더만 읽어 잴 수 있는** 형식이다. 크기를 못 재면 배율·좌표가
의미를 잃으므로 삽입을 시작조차 하지 않는다.

## 곁가지로 잡은 코어 결함 — `img_dim` 비대칭

`edit insert-image --verify` 가 **정상 삽입에도 항상 exit 3** 이었다. 진단(실측 §14b):

```
DIFF: PictureSize { path: "/ctrl[2]pic",
       detail: "imgDim: expected=(0, 0) actual=(3000, 1500)" }
```

`insert_picture_native` 가 `img_dim` 을 기본값 `(0,0)` 으로 두는데,

- HWP5 직렬화기는 `img_dim == (0,0)` 이면 `crop.right/bottom` 을 원본 크기 자리에
  기록하고(`serializer/control.rs:1133`, #1929 폴백),
- HWP5 파서는 그 값을 다시 `img_dim` 으로 적재한다(`parser/control/shape.rs:948`).

그래서 **삽입 직후 IR 과 저장본 재파싱 IR 이 언제나 이 한 필드만큼 어긋났다.**
`insert_picture_native` 의 세 분기(본문·표 셀·글상자) Picture 에 `img_dim` 을
`(natural_px × 75)` 로 채워 정합시켰다 — 직렬화기가 이미 쓰던 값과 **같은 값**이다.

렌더 무영향 근거: `compute_image_crop_src`(`renderer/svg.rs:3344`)는 `imgDim` 이
없으면 `crop.right/bottom` 을 같은 자리에 쓰는 폴백을 가진다. 즉 두 경로의 배율
`scale_x/scale_y` 가 동일하다. HWPX 산출은 종전 `imgDim 0/0` 대신 실제 좌표 범위를
얻어 **HWP5 산출과의 비대칭이 해소**된다.

무회귀: `cargo test --release --lib` 3030 passed / 0 failed.

## 드리프트 가드

| 가드 | 대응 |
| --- | --- |
| MCP `inputSchema` 에 `type:object` + `properties` + `required` 배열 | `required: ["path","image"]` (실측 §18) |
| 선언한 입력 속성 전부가 CLI 인자에 배선 | 9속성 전부 — 값 없는 `dryRun` 은 `optionalArgs` presence 규약. 미배선 0건 |
| `--json` 명령은 MCP 도구 필수 | `hwp_insert_image` (매니페스트 도구 26종) |
| capabilities 등재 명령은 `--help` 에도 | `edit` 축에 `insert-image` 절 추가 |
| 선언한 flags 는 실제 수용 | `capabilities_and_mcp_declare_insert_image_axis` 가 선언 조합을 그대로 호출해 확인 |
| 실패 경로 stdout 0바이트 | 인자 오류 7종 · 런타임 실패 2종 전수 |
| `commands[edit].flags` 누락 없이 | `--image --page --x --y --width --height` 추가 + **`--verify` 도 함께**(edit 4종이 모두 받는데 선언만 빠져 있던 기존 드리프트) |

## 검증

- 신규 `insert_image_contract` **19건** green (본류·크기 규약 2·dry-run·넘침 2·
  형식 6종·앵커 전 쪽·실패 9종·`--verify`·HWPX 형식 보존·자기서술·봉투↔선언 대조)
- 무회귀 계약 시험 81건 green — cli_json 26 · mcp_server 22 · profile_router 7 ·
  set_cell 5 · verify 4 · changed_pages 5 · fill_fields 7 · replace_text 5
- `cargo test --release --lib` 3030 passed / 0 failed / 7 ignored
- `cargo clippy -- -D warnings` 경고 0 · rustfmt `--check` 차이 0
- MCP 서버 실호출 왕복 성공(실측 §19) — `tools/call hwp_insert_image` → 474112바이트 산출

시험은 **좌표·쪽 수를 박지 않는다.** 용지 크기는 `--dry-run` 넘침 보고에서, 쪽 수는
`info --json` 에서, 봉투 키 이름은 `capabilities --mcp` 의 `outputFields` 에서 읽어
쓴다 — 샘플이나 선언이 바뀌면 시험이 함께 따라가거나 소리 내어 깨진다.

## 남은 것

- **에이전트 프로필 미등재**: `agent_profiles.rs` 의 `행정서식` 프로필 도구 목록에
  `hwp_insert_image` 를 넣어야 그 역할로 필터링한 에이전트가 쓸 수 있다. 같은 목록을
  동시 작업 중인 다른 축들이 함께 건드리므로 충돌을 피해 후속 분리했다(한 줄 추가).
- **세션 도구 미제공**: 무상태 CLI/MCP 만 있고 `hwp_doc_insert_image` (핸들 편집)는
  없다. `hwp_doc_set_cell` 계열과 같은 자리에 적층할 수 있다.
- **표 셀·글상자 좌표 지정 없음**: 코어는 `cell_path` 로 셀 안 배치를 지원하지만 v1
  CLI 는 본문 용지 기준만 노출한다(도장·서명의 실사용 형태). 셀 지정이 필요해지면
  `--table/--row/--col` 축을 `set-cell` 좌표계 그대로 얹으면 된다.
- **`mydocs/manual/cli_commands.md` 반영**: 명령 레퍼런스 문서는 이 PR 범위 밖으로
  두었다(같은 파일을 여러 축이 동시 수정 중).
