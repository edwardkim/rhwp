## 무엇을 고치나

Issue #3545 의 **잔여 축**입니다. 선행 PR #3659(merge `952c831a`)가 HWPX
`fieldBegin@dirty` ↔ `properties` 비트 15 왕복을 해소해 **채운 값**은 보존되지만,
초기 상태(`dirty="0"`) 누름틀의 **안내문 본문 run 을 적재 때 물리 삭제**하는 축은
그대로 남아 있었습니다. 그 결과 HWPX 를 **열고 저장하기만 해도** 해당 텍스트가
파일에서 영구 소실됩니다.

한컴은 미기입 누름틀의 안내문을 파일에는 본문 run 으로 유지하고 렌더·인쇄에서만
구분 취급합니다 — 동봉 샘플 `samples/hwpx/form-01.hwpx` 실물이 그 정준형입니다.
XSD·구조 검증은 통과하고, 재적재하면 같은 정규화가 다시 지워 값 API 표면에도 안
나타나며, 빈 필드 안내문 합성 렌더 덕분에 화면도 멀쩡해 보입니다. **저장 전후 파일
대조로만 드러나는 조용한 내용 소실**입니다.

## 전 / 후 (저장본 `Contents/section0.xml`, `samples/hwpx/form-01.hwpx`)

원본 (한컴 정준형):

```xml
<hp:fieldBegin ... type="CLICK_HERE" name="myMsg01" editable="1" dirty="0" ...>
<hp:run charPrIDRef="6"><hp:t>여기에 입력</hp:t></hp:run>
<hp:run charPrIDRef="1"><hp:ctrl><hp:fieldEnd .../></hp:ctrl><hp:t/></hp:run>
```

**전** — rhwp 저장본에서 본문 텍스트가 사라짐 (run 껍데기만 남음):

```xml
<hp:run charPrIDRef="6"><hp:t></hp:t></hp:run>
```

**후** — 원본 형상 복원 (charPrIDRef 까지 동형):

```xml
<hp:run charPrIDRef="6"><hp:t>여기에 입력</hp:t></hp:run>
```

> 시각(렌더) 전/후 이미지는 첨부하지 않았습니다. 이 결함은 **렌더에 나타나지 않는**
> 부류이기 때문입니다 — rhwp 는 빈 필드에 안내문을 합성 렌더하므로 수정 전후 SVG 가
> 동일합니다. 관측 가능한 유일한 표면인 저장본 XML 대조로 대신했습니다.

## 해법 — 삭제는 유지, 잔재를 기록해 저장 시 복원

정규화 계약(편집 IR 은 빈 필드, 값 API 는 빈 값)을 **1비트도 바꾸지 않고**, 삭제
시점에 잔재를 파생 상태로 기록해 HWPX 저장에서만 원본 run 을 되살립니다.

1. `model::control::GuideResidue { text, char_shape_id }` 신설 +
   `Field.guide_residue: Option<GuideResidue>`.
2. `clear_initial_field_texts` 가 삭제할 때 제거 텍스트와 *그 텍스트를 담던 run 의
   `charPrIDRef`* 를 함께 기록합니다 (삭제 수술이 경계를 zero-width 로 접기 **전**
   좌표에서 조회해야 정확).
3. HWPX 저장기가 0-length 필드의 `fieldEnd` 방출 직전, 기록된 char shape 경계까지만
   run 을 끊고 그 안에 `<hp:t>` 를 되돌립니다. IR 위치 축은 건드리지 않으므로
   저장→재적재 고정점이 유지됩니다.

### 이슈의 방향 (i)/(ii) 에 대해

- 이슈가 지적한 **(ii) 저장 시 재구성**의 약점은 "삭제 수술로 원본 char shape 가 이미
  소실돼 서식 충실 재구성이 불가능"이었습니다. 삭제 **시점에** 텍스트와 char shape 를
  같이 기록하면 그 전제가 무너집니다 — 복원본은 원본과 문자열 동형입니다.
- **(i) 적재 보존 → 소비 계층 해석**은 IR 정규화 전제를 바꿔 값 API·렌더·텍스트
  추출·라운드트립 지표·렌더 golden 전반에 영향을 줍니다. 이 PR 은 그 결정을 선점하지
  않습니다. 정규화 계약을 그대로 두므로, (i) 로 가기로 결정되면 이 복원 경로는 그대로
  대체·제거되고 데이터 소실만 그동안 막습니다.

## 잘못 주입하지 않는 게이트

- 원본부터 잔재가 없던 필드는 표식이 없어 아무것도 주입되지 않습니다
  (`samples/issue1893_clickhere_field_roundtrip.hwpx` 의 `id=1549188905` — `Direction`
  파라미터조차 없는 빈 스팬). 계약 테스트로 고정했습니다.
- 값이 채워진 필드(`start != end`)와 수정됨 표식(bit 15)이 선 필드는 건너뜁니다 —
  중복 주입과 사용자가 비운 값의 부활을 막습니다.

## 검증 (red → green 실행 확인)

| 게이트 | 결과 |
| --- | --- |
| red (수정 전) `--test issue_3545_clickhere_dirty_roundtrip` | 3 failed / 5 passed — 저장본 안내문 run **0회** |
| green (수정 후) 같은 타깃 | **8 passed** |
| `cargo test --profile release-test --lib` (변경 소스 소속 타깃) | **3016 passed / 0 failed** |
| 필드·HWPX 왕복 focused 타깃 16종 | **84 passed / 0 failed** |
| `rustfmt --check` (변경 파일) | 통과 |
| `cargo clippy --profile release-test --bin rhwp -- -D warnings` | 통과 |

focused 16종: `edit_field_occurrence_contract`, `edit_fill_fields_contract`,
`fields_json_contract`, `hwpx_form_roundtrip`, `hwpx_roundtrip_baseline`,
`hwpx_roundtrip_integration`, `ir_field_sweep_baseline`, `issue_1391_memo_field_roundtrip`,
`issue_1434_clickhere_guide_hancom_command`, `issue_1893`, `issue_258_clickhere_form_mode`,
`issue_3375_field_guide_print_profile`, `issue_3380_field_value_equals_guide`,
`issue_3545_clickhere_dirty_roundtrip`, `issue_493_hwpx_cell_field_name`,
`issue_838_field_set_value`. `--tests` 전체 스위트는 로컬 제약으로 실행하지 않고 CI 에
위임했습니다.

추가한 계약 테스트 5건: 저장 보존(charPrIDRef 동형까지), 저장→재적재→재저장 고정점
(값 API 는 빈 값 유지), 채운 필드 중복 방지, 실물 행정 서식의 잔재 보존 + 원본 빈 스팬
무주입.

## 남긴 것

- **HWP5 저장 축**: 같은 적재 정규화가 HWP5 원본에도 걸리므로 `export_hwp` 경로에는
  소실이 남습니다. 이 이슈가 HWPX 축으로 좁혀 제기됐고 HWP5 직렬화는 별도 golden
  면적을 건드리므로 후속 건으로 분리했습니다. 표식은 포맷 무관하게 기록되므로 HWP5
  저장기 배선만 추가하면 됩니다.
- 슬롯 위치 추정 실패(mismatch) 퇴화 경로는 복원 대상에서 제외했습니다 — 그 경로는
  run·위치 정합이 이미 무너져 있어 주입이 개선이라 단정할 수 없습니다.

처리결과 문서: `mydocs/report/task_m100_3545_report.md`

🤖 Generated with [Claude Code](https://claude.com/claude-code)
