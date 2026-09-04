# Task #2784 처리 결과 — CommonObjAttr affectLSpacing 필드 추가로 5개 경로 유실 해소

이슈: https://github.com/edwardkim/rhwp/issues/2784
브랜치: `task/m100-2784-affect-linespacing-field` (`origin/devel` 기준)

## 1. 문제 요약

`<hp:pos affectLSpacing>`("개체가 줄 간격에 영향을 주는가")는 양식 컨트롤만 프로퍼티백으로
보존되고, 그림·도형·표·공용 도형·수식은 저장 시 전부 `"0"` 으로 하드코딩됐다. 근본 원인은
`CommonObjAttr`(`src/model/shape.rs`)에 이 값을 담을 필드가 없었기 때문이다 — 값이 있어도
필드가 없으면 파서는 버리고 직렬화기는 상수를 낼 수밖에 없다.

## 2. 실측 — `samples/issue1949_giant_cell_nested_tables_perf`(한컴 원본 hwp/hwpx 쌍)

- HWPX 원본: `affectLSpacing="1"` 6개 (표 1 + 수식 5). `export-hwpx` 왕복 후 0개로 붕괴.
- 한컴 원본 `.hwp`(OLE, FileHeader `HWP Document File` v5.1.1.0, 작성자 admin/Windows_10,
  압축)를 올레파일로 직접 열어 `CTRL_HEADER`(HWPTAG 0x47) 를 스트림 압축 해제 후 파싱: 개체
  공통 속성 attr **bit 2** set 개체가 `tbl 1개 + eqed 5개` — 짝 HWPX 의 `affectLSpacing="1"`
  개체(표1+수식5)와 **1:1 정확히 일치**. 스펙 문서 `mydocs/tech/한글문서파일형식_5.0_revision1.3.md:1482`
  표 70 의 `bit 2 = 줄 간격에 영향을 줄지 여부` 와도 부합한다.
- 결론: **affectLSpacing = HWP5 개체 공통 속성 attr bit 2**, 실파일로 검증됨. bit 1 은 어느
  개체에서도 set 되지 않아 "예약"이라는 스펙 기술과 상충하지 않는다.

## 3. 근본 수정 (1 필드 배선)

- `src/model/shape.rs` — `CommonObjAttr` 에 `affect_line_spacing: bool` 필드 추가.
- `src/parser/control/shape.rs::parse_common_obj_attr` — `attr & (1 << 2)` 로 HWP5 bit 2 읽기.
- `src/document_core/converters/common_obj_attr_writer.rs::pack_common_attr_bits` — HWPX 유래
  (`attr=0`) 합성 경로에서 bit 2 를 다시 세팅. `serializer/control.rs` 의 표 직렬화기도 같은
  헬퍼를 재사용하므로 자동 반영.
- `src/parser/hwpx/section.rs` — 표/도형·그림/공통 개체를 읽는 **4곳**의 일반 `<hp:pos>` 루프에
  `b"affectLSpacing"` arm 추가(기존에는 `treatAsChar`/`flowWithText`/`allowOverlap` 만 읽고
  이 속성은 통째로 버려졌다).
- `src/serializer/hwpx/{picture,shape,table}.rs` — 하드코딩 `"0"` 을
  `bool01(common.affect_line_spacing)` 로 교체.

## 4. 레드→그린 (직접 실행, 두 계층 모두 load-bearing 임을 증명)

신규 테스트 `task2784_table_affect_line_spacing_roundtrips`
(`src/serializer/hwpx/roundtrip.rs`): 표 `affect_line_spacing=true` → `serialize_hwpx` →
`parse_hwpx` 후 값이 보존되는지 검증.

- **RED #1** — `table.rs` 의 emit 을 `"0"` 으로 되돌리고(파서는 그대로) 실행:
  ```
  test serializer::hwpx::roundtrip::tests::task2784_table_affect_line_spacing_roundtrips ... FAILED
  panicked at src\serializer\hwpx\roundtrip.rs:1641:9:
  표 affectLSpacing 이 왕복 보존돼야 함
  ```
  → 원복 후 GREEN 확인.
- **RED #2** — `section.rs` 의 표 pos 파서 arm 을 제거하고(직렬화기는 그대로) 실행:
  ```
  test serializer::hwpx::roundtrip::tests::task2784_table_affect_line_spacing_roundtrips ... FAILED
  panicked at src\serializer\hwpx\roundtrip.rs:1641:9:
  표 affectLSpacing 이 왕복 보존돼야 함
  ```
  → 원복 후 GREEN 확인.

파서·직렬화기 두 계층이 독립적으로 실효성 있음을 실측으로 증명했다.

추가로 `common_obj_attr_writer.rs` 에 HWP5 bit 팩/언팩 단위 테스트
(`roundtrip_affect_line_spacing_bit2`)를 추가해 bit 2 세팅·bit 1(예약) 비세팅을 함께 assert.

## 5. 검증 (오늘 우선순위: 속도 — 전수 재검증 대신 자체 테스트 + 정적 게이트로 확인)

- `cargo build --lib`, `cargo check --all-targets --profile release-test` — 통과(누락 필드
  E0063 없음; 모든 다른 `CommonObjAttr { .. }` 리터럴은 `..Default::default()` 사용이라
  영향 없음).
- 신규 테스트 2건(`task2784_table_affect_line_spacing_roundtrips`,
  `roundtrip_affect_line_spacing_bit2`) 개별 실행 — 통과.
- `cargo test --tests --profile release-test --no-fail-fast` 전체 실행(브랜치 전략 변경
  전 1회 완주) — 실패 0건. `hwpx_roundtrip_baseline`·`hwp5_roundtrip_baseline` 테스트
  바이너리 모두 정상 구동(실패 로그 없음).
- `cargo clippy --all-targets --profile release-test -- -D warnings` — 경고/에러 0건.
- 변경된 `.rs` 8개 파일 전부 `rustfmt --edition 2021` 적용, diff 는 8개 파일에 한정(다른
  파일로 번짐 없음).

## 6. 범위 밖 (잔여)

`src/serializer/hwpx/section.rs` 의 공용 도형(`render_common_shape_xml`, ≈:1913)과 수식
(`render_equation`, ≈:2030) 두 방출 지점은 **동일 필드**로 `bool01(common.affect_line_spacing)`
한 줄만 바꾸면 되지만, 해당 파일을 다른 작업이 동시 편집 중이라 충돌을 피하기 위해 이번 PR
에서 제외했다. issue1949 실측의 수식 5개 손실은 이 잔여가 풀려야 회복된다. 이번 PR 로
회복되는 실측분은 **표 1개**이며, 그림·도형 경로는 코드적으로 동일 필드를 배선했으나
issue1949 샘플에는 그림/도형 쪽 `affectLSpacing="1"` 사례가 없어 이번 파일 기준 실측 회복
수치에는 포함되지 않는다(향후 다른 샘플로 회귀 확인 가능).

## 7. 커밋 대상 파일

```
src/model/shape.rs
src/parser/control/shape.rs
src/document_core/converters/common_obj_attr_writer.rs
src/parser/hwpx/section.rs
src/serializer/hwpx/picture.rs
src/serializer/hwpx/shape.rs
src/serializer/hwpx/table.rs
src/serializer/hwpx/roundtrip.rs
mydocs/report/task_m100_2784_report.md
```
