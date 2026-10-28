---
kind: report
status: active
last_verified: 2026-08-01
---

# Task #3545 처리결과 — 초기 상태 누름틀 안내문의 파일 수준 보존

- Issue: [#3545](https://github.com/edwardkim/rhwp/issues/3545) —
  「[HWPX] 안내문과 같은 본문 텍스트를 가진 누름틀이 로드만 해도 텍스트를 잃고,
  저장하면 파일에서 영구 소실된다」
- 브랜치 `task/3545-hwpx-pressframe-text-loss` (기준 `upstream/devel` f80b910aa)
- 선행 축: [#3659](https://github.com/edwardkim/rhwp/pull/3659) (merge `952c831a`) 가
  HWPX `fieldBegin@dirty` ↔ `Field.properties` 비트 15 왕복을 해소 — **채운 값**은
  이제 보존된다. 이 작업은 이슈에 남겨 둔 **잔여 축**, 즉 초기 상태(`dirty="0"`)
  안내문 잔재의 물리 삭제를 다룬다.

## 문제

적재 정규화 `clear_initial_field_texts`(`src/document_core/commands/document.rs`)는
`properties` 비트 15 == 0 인 ClickHere 필드의 본문 텍스트가 안내문과 같으면 이를
`para.text` 에서 **물리 삭제**한다. 함수 주석은 이 텍스트를 "메모 추가 등의 동작 시
삽입되는" 이례 상태로 전제하지만, 동봉 샘플 실물은 그 반대를 보여준다.

`samples/hwpx/form-01.hwpx` 원본:

```xml
<hp:fieldBegin ... type="CLICK_HERE" name="myMsg01" editable="1" dirty="0" ...>
<hp:run charPrIDRef="6"><hp:t>여기에 입력</hp:t></hp:run>
<hp:ctrl><hp:fieldEnd .../></hp:ctrl>
```

즉 한컴은 미기입 누름틀의 안내문을 **파일에는 본문 run 으로 유지**하고 렌더·인쇄에서만
구분 취급한다. 이것이 정준형이다. rhwp 는 적재 때 이 run 을 지우고 저장기가 되살리지
않으므로, **열고 저장하기만 해도 그 텍스트가 파일에서 영구 소실**된다. XSD·구조 검증은
통과하고, 재적재하면 같은 정규화가 다시 지워 값 API 표면에도 안 나타나며, 빈 필드
안내문 합성 렌더 덕에 화면도 멀쩡해 보인다 — 저장 전후 파일 대조로만 드러나는 부류다.

## 해법 — 삭제는 유지, 잔재를 기록해 저장 시 복원

정규화 계약(편집 IR 은 빈 필드, 값 API 는 빈 값)을 **그대로 두고**, 삭제 시점에 잔재를
파생 상태로 기록해 HWPX 저장에서만 원본 run 을 되살린다.

1. **IR 표식** — `model::control::GuideResidue { text, char_shape_id }` 신설,
   `Field.guide_residue: Option<GuideResidue>` 추가 (`src/model/control.rs`).
2. **기록** — `clear_initial_field_texts` 가 삭제할 때 제거 텍스트 원문과 *그 텍스트를
   담던 run 의 `charPrIDRef`* 를 함께 남긴다. char shape 는 삭제 수술이 경계를
   zero-width 로 접기 **전** 좌표에서 조회해야 정확하다.
3. **복원** — HWPX 저장기 `render_runs` 가 0-length 필드의 `fieldEnd` 를 방출하기 직전,
   기록된 char shape 경계까지만 run 을 끊고 그 안에 `<hp:t>` 를 되돌린다
   (`emit_guide_residue` / `emit_field_end_at`, `src/serializer/hwpx/section.rs`).
   IR 위치 축(`expected_utf16_pos`)은 건드리지 않는다 — 방출 XML 에만 텍스트가 돌아온다.

### 왜 이 형태인가

- 이슈가 제시한 **(ii) 저장 시 재구성**의 결정적 약점은 "삭제 수술로 원본 char shape 가
  이미 소실돼 서식 충실 재구성이 불가능"이었다. 삭제 **시점에** 텍스트와 char shape 를
  같이 기록하면 이 전제가 무너진다 — 복원본은 원본과 문자열 동형이다.
- **(i) 적재 시 보존 → 소비 계층 해석**은 IR 정규화 전제를 바꾸므로 값 API·렌더·텍스트
  추출·라운드트립 지표·렌더 golden 전반에 영향을 준다. 이 PR 은 그 큰 결정을 선점하지
  않는다: 정규화 계약을 1비트도 바꾸지 않으므로 (i) 로 가더라도 그대로 대체된다.
- 삭제 자체가 이미 zero-width char run 을 원본 서식의 근거로 남겨 두고 있었으므로
  (#1893 주석), 복원은 그 근거를 소비하는 자연스러운 역연산이다.

### 잘못 주입하지 않는 게이트

- 원본부터 잔재가 없던 필드는 표식이 없어 아무것도 주입되지 않는다
  (`samples/issue1893_clickhere_field_roundtrip.hwpx` 의 `id=1549188905` — `Direction`
  파라미터조차 없는 빈 스팬).
- 값이 채워진 필드(`start != end`)와 수정됨 표식이 선 필드(`bit 15`)는 건너뛴다 —
  중복 주입·사용자가 비운 값의 부활을 막는다.

## 검증

red → green 은 실제 실행으로 확인했다.

| 게이트 | 결과 |
| --- | --- |
| red (수정 전) `--test issue_3545_clickhere_dirty_roundtrip` | **3 failed / 5 passed** — 저장본 안내문 run 0회 |
| green (수정 후) 같은 타깃 | **8 passed** |
| `cargo test --profile release-test --lib` (변경 소스 소속 타깃) | **3016 passed / 0 failed** (7 ignored) |
| 필드·HWPX 왕복 focused 타깃 16종 | **84 passed / 0 failed** |
| `rustfmt --check` (변경 파일) | 통과 (CRLF 오탐 제거 후 exit 0) |
| `cargo clippy --profile release-test --bin rhwp -- -D warnings` | 통과 |

focused 16종: `edit_field_occurrence_contract`, `edit_fill_fields_contract`,
`fields_json_contract`, `hwpx_form_roundtrip`, `hwpx_roundtrip_baseline`,
`hwpx_roundtrip_integration`, `ir_field_sweep_baseline`, `issue_1391_memo_field_roundtrip`,
`issue_1434_clickhere_guide_hancom_command`, `issue_1893`, `issue_258_clickhere_form_mode`,
`issue_3375_field_guide_print_profile`, `issue_3380_field_value_equals_guide`,
`issue_3545_clickhere_dirty_roundtrip`, `issue_493_hwpx_cell_field_name`,
`issue_838_field_set_value`.

> `cargo test --profile release-test --tests` 전체 스위트는 이 세션에서 실행하지
> 않았다(로컬 디스크·시간 제약). 변경 소스 소속 타깃(`--lib`)과 필드·HWPX 왕복 면적을
> focused 로 덮었고, 전체 회귀는 PR CI 에 위임한다.

red 실패 메시지(대표):

```text
initial_guide_body_run_survives_hwpx_save
  assertion `left == right` failed: 초기 상태 누름틀의 안내문 본문 run 이 저장에서 소실/중복됐다
  left: 0   right: 1
```

계약 테스트 5건은 `tests/issue_3545_clickhere_dirty_roundtrip.rs` 에 추가했다 —
저장 보존, 저장→재적재→재저장 고정점(값 API 는 빈 값 유지), 채운 필드 중복 방지,
실물 행정 서식의 잔재 보존 + 원본 빈 스팬 무주입.

## 남긴 것 (범위 밖)

- **HWP5 저장 축**: 같은 적재 정규화가 HWP5 원본에도 적용되므로 `export_hwp` 경로에도
  같은 소실이 남아 있다. 이 이슈는 HWPX 축으로 좁혀 제기됐고 HWP5 직렬화는 별도
  golden 면적을 건드리므로 후속 건으로 분리한다. 표식(`guide_residue`)은 포맷 무관하게
  기록되므로 HWP5 저장기 배선만 추가하면 된다.
- **mismatch 경로**: 슬롯 위치 추정 실패로 `fieldEnd` 를 말미 일괄 방출하는 퇴화 경로는
  복원 대상에서 제외했다 — 그 경로는 run/위치 정합이 이미 무너져 있어 주입이 개선이라
  단정할 수 없다.
- 이슈의 **(i) 렌더 전용 억제** 전환 판단은 그대로 열려 있다.

## 교훈

- "정규화가 파일을 조용히 줄인다"는 부류는 값 API·렌더 어느 표면에도 안 나타난다.
  **저장 전후 파일 대조**가 유일한 관측면이라, 계약 테스트도 IR 이 아니라 저장본 XML 을
  직접 봐야 한다.
- 파괴적 정규화를 유지해야 한다면, **파괴 시점이 원본 형상을 기록할 유일한 기회**다.
  나중에 재구성하려 하면 이미 근거가 없다.
