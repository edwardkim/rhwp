# task_m100_4396 Stage 1 — HWPX 필드 파라미터가 HWP5 왕복에서 Command 하나로 축소

- **이슈**: [#4396](https://github.com/edwardkim/rhwp/issues/4396)
- **브랜치**: `fix/issue-4396-hwpx-field-parameters`
- **분기 기준**: `upstream/devel` `9f5911e86` (0 behind)
- **상태**: 게이트 통과, PR 게시
- **기록일**: 2026-08-10 KST

## 1. 결함

HWPX 필드의 `<hp:parameters>` 가 HWP5 왕복 후 `Command` 파라미터 하나만 남는다.

```
원본  <hp:parameters cnt="3" ...>
        <hp:integerParam name="Prop">9</hp:integerParam>
        <hp:stringParam name="Command">Clickhere:set:66:...</hp:stringParam>
        <hp:stringParam name="Direction">이곳을 마우스로 누르고 내용을 입력하세요.</hp:stringParam>
      </hp:parameters>
왕복후 <hp:parameters cnt="1" ...>
        <hp:stringParam name="Command">Clickhere:set:66:...</hp:stringParam>
      </hp:parameters>
```

## 2. 고친 것

파서가 `<hp:parameters>` 를 raw 텍스트로만 잡던 것을 OWPML `hp:ParameterList`
(`ParaList XML schema.xml:2764`) 구조 그대로 트리로 읽도록 했다 — 다섯 변형
(`booleanParam`/`integerParam`/`floatParam`/`stringParam`/재귀 `listParam`)과 필수 `cnt`.
`Field::parameters` 와 `Parameter` 모델을 추가하고 직렬화기가 그 트리를 되쓴다.

## 3. 리뷰가 되돌린 것 — 없는 슬롯을 발명했다

첫 수정은 손실을 막으려고 HWP5 CTRL_DATA 에 `item_id=0x4010` 을 **발명해** 파라미터를
담았다. 두 가지가 틀렸다.

1. **스펙에 없다.** `pdf/hwpspec-2024.pdf` §4.2.8(HWPTAG_CTRL_DATA, 표 61) /
   §4.2.10.11 / §4.2.10.15 를 확인한 결과, 규정된 것은 이름 항목(`item_id=0x4000`,
   hwplib 실측)뿐이고 그 위의 스키마가 문서화돼 있지 않다.
2. **충돌한다.** `src/document_core/converters/hwpx_to_hwp.rs:1822` 가 표 레이아웃
   CTRL_DATA 에 `0x4000_u16 + idx` 를 순차 할당한다. `0x4010` 은 그 범위 안이다.

되돌렸다. **HWP5 축은 손실을 막지 못한 채로 두고, 대신 경고를 남긴다** —
`field_parameter_loss_warning`(`src/serializer/control.rs:182`, `:232`), 근거는
`src/model/control.rs:670` 부근 doc comment.

"규정하면 따르고, 규정하지 않으면 발명하지 않는다"가 이 저장소의 판단 규칙이다.

## 4. 검증

RED 확인에 함정이 있었다. 커밋된 테스트는 새 API(`Field::parameters`/`Parameter`)를 쓰므로
수정 전 소스에서는 **컴파일이 안 된다** — 그건 결함의 증거가 아니라 API 부재다. 그래서
수정 전에도 존재하는 `raw_parameters_xml` 만 쓰는 임시 테스트를 따로 만들어 소스만 되돌리고
돌렸다.

```
test field_parameters_not_collapsed_to_command_only_after_hwp5_roundtrip ... FAILED
필드[0]: HWP5 왕복 후 Prop 파라미터가 사라짐(Command 하나로 축소됨)
  원본  cnt="3" (Prop, Command, Direction)
  왕복후 cnt="1" (Command)
```

이슈에 실린 예시와 정확히 일치한다. 이후 임시 파일을 지우고 트리를 복원했다.

- `cargo fmt --all -- --check` exit 0
- `cargo clippy --all-targets -- -D warnings` exit 0
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` exit 0 —
  `test result: ok` 블록 **503개, FAILED 0건**

## 5. 이 작업에서 고치지 않은 것

전부 별건으로 열었다.

- **[#4436](https://github.com/edwardkim/rhwp/issues/4436)** — `parse_field_parameters` 의
  중첩 `listParam` 재귀에 깊이 상한이 없다. 기존 raw 캡처에도 없던 위험을 트리 빌더가
  그대로 물려받았다.
- **[#4437](https://github.com/edwardkim/rhwp/issues/4437)** — `booleanParam` 렌더가
  원본 lexical 표기(`false`/`true`)를 `0`/`1` 로 정규화한다. 코퍼스에 `Fiexde=1` 과
  `RefHyperLink=false` 가 섞여 있다.
- **[#4438](https://github.com/edwardkim/rhwp/issues/4438)** — 위 3절의
  `0x4000 + idx` 순차 할당이 근거 기록 없이 이미 운영 중이다.

검증 공백도 남긴다.

- `floatParam` 은 코퍼스 3,418건 / 요소 2,572개에서 **0건 관측**. 합성 유닛 테스트만 있고
  실제 한컴 출력과의 숫자 포맷 정합은 미검증.
- `FieldType::Memo` 의 `Number` 파라미터(이슈 원문이 든 예시)도 **0건 관측**.
- `ParameterList::parse_xml`(`src/model/control.rs:394`)은 HWP5 확장을 되돌린 뒤
  프로덕션 호출부가 없다. 모델 자체 왕복 계약을 증명하는 유닛 테스트에서만 불린다.
- `generated_field_parameters`(`src/serializer/hwpx/section.rs:1600`)의
  `raw_parameters_xml` 없는 분기는 현재 어떤 실제 호출 경로로도 도달하지 않는다.
  API 로 필드를 새로 만드는 기능이나 HWP5 슬롯이 생기면 그때 처음 실사용된다.

## 6. 미처리

GitHub Actions, 작업지시자 승인, merge.
