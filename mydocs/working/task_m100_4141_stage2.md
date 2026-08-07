# Stage 2 — task_m100_4141 수정과 계약 고정

- **이슈**: [#4141](https://github.com/edwardkim/rhwp/issues/4141)
- **계획서**: [`mydocs/plans/task_m100_4141.md`](../plans/task_m100_4141.md)
- **선행 단계**: [stage1](task_m100_4141_stage1.md)
- **브랜치**: `task_m100_4141` (분기 기준 `upstream/devel` `0fdac31ba`)
- **작업 시각**: 2026-08-07 KST
- **프로덕션 코드 변경**: `src/model/style.rs` 1곳 (`impl Default for CharShape`)

## 1. TDD 순서 — 빨강을 먼저 확인했다

계약 테스트를 먼저 쓰고 **수정 없이** 돌려 5건 전부 실패함을 확인한 뒤 수정했다.

### 빨강 (수정 전)

```text
test result: FAILED. 0 passed; 5 failed; finished in 0.82s
```

실패 메시지가 Stage 1 실측치와 정확히 일치했다 — 테스트가 진짜 결함을 보고 있다는 증거다:

```text
samples/SO-SUEOP.hwp: CHAR_SHAPE 2512개 중 2512개의 상대크기가 100 이 아니다
  (첫 위반 id=0 한글=0). ...
samples/hwp3-sample.hwp: CHAR_SHAPE 747개 중 747개 ...
samples/hwp3-sample10.hwp: CHAR_SHAPE 28193개 중 28193개 ...
tests/fixtures/hml/exambank_math_equations_min.hml: <RELSIZE> 2개 중 2개가
  유효범위 10~250 밖이다 (첫 위반: `<RELSIZE Hangul="0" ... />`)
samples/SO-SUEOP.hwp: <hh:relSz> 2512개 중 2512개가 유효범위 10~250 밖이다
```

### 초록 (수정 후)

```text
test result: ok. 5 passed; 0 failed; finished in 0.61s
```

## 2. 수정 — `src/model/style.rs` 한 곳

`#[derive(Debug, Clone, Default)]` 에서 `Default` 를 빼고 수동 impl 을 썼다.
**`relative_sizes: [100; 7]` 한 줄만 파생값과 다르고** 나머지 30개 필드는 파생값을 그대로 나열했다.

이 한 곳이 Stage 1 §2 의 **6개 누출 경로를 전부** 해소한다 — 전부
`CharShape::default()` 또는 `..Default::default()` 를 통과하기 때문이다. 실제로 라이터가 셋인
3축(HWP5 바이트 / HWPX XML / HML XML)이 **같은 커밋 하나로** 동시에 초록이 됐다.

주석에 네 가지를 남겼다: ① 왜 파생 Default 로는 안 되는가(OWPML `positiveInteger` [10,250],
한컴의 `크기 × 상대크기%` 해석, 6개 누출 경로) ② HWP5 파서가 이미 100 을 폴백하므로
파생값 0 은 그 폴백과도 불일치였다는 점 ③ 왜 `ratios`·`base_size` 는 그대로 두는가(렌더러가
소비 → 별도 이슈) ④ 왜 필드를 전부 나열하는가(새 필드 추가 시 컴파일 에러로 표류 차단).

### 함께 바꾸지 않은 것

`ratios`·`base_size`(렌더러가 소비), `shade_color`·`shadow_color`·`underline_color`(sentinel),
`char_offsets`·`spacings`(0 이 스펙상 유효값). 이 비대칭을
`src/model/style.rs` 의 `char_shape_default_matches_spec_only_for_relative_sizes` 가 고정한다 —
다음 사람이 "왜 ratios 는 안 고쳤나"를 코드에서 읽는다.

## 3. 추가한 테스트

### 통합 계약 — `tests/issue_4141_hwp3_relative_size_contract.rs` (신규, 5건)

`tests/issue_3676_hwp3_convert_hancom_openable.rs` 를 확장하지 않고 새 파일로 만들었다.
실패 부류가 다르고(개봉 거부 vs 열리는데 백지), 스트림이 다르고(BodyText vs DocInfo),
표본 범위가 다르다(단일 vs 전수).

| 테스트 | 축 | 대상 |
| --- | --- | --- |
| `hwp3_convert_emits_valid_relative_sizes_for_every_sample` | HWP5 바이트 | HWP3 표본 **전수** (오프셋 28..35) |
| `so_sueop_convert_relative_sizes_are_all_100` | HWP5 바이트 | `samples/SO-SUEOP.hwp` 이름 고정 |
| `public_document_core_export_also_emits_valid_relative_sizes` | HWP5 바이트 | `export_hwp_with_adapter` 경로 |
| `hwp3_export_hwpx_emits_valid_rel_sz` | HWPX XML | `Contents/header.xml` 의 `<hh:relSz>` |
| `hml_roundtrip_without_relsize_child_emits_valid_relsize` | HML XML | `RELSIZE` 자식 없는 fixture 왕복 |

재사용: `Record::read_all`(`src/parser/record.rs:34`) + `CfbReader::read_doc_info`(압축 해제 포함).
`walk_records` 를 손으로 다시 쓰지 않았다 — `tests/issue_3507_sectiondef_ctrl_data.rs:52-62` 골격을 따랐다.

전수 스윕에 `assert!(swept >= 10)` 하한을 뒀다 — 표본이 전부 건너뛰어져 조용히 통과하는 것을 막는다.

### 유닛 (3건)

- `src/model/style.rs::char_shape_default_matches_spec_only_for_relative_sizes`
- `src/parser/hwpx/header.rs::char_pr_without_rel_sz_child_defaults_to_100_percent`
  — `<hh:relSz>` 자식 부재 시 100. 명시된 `ratio=95` 는 덮이지 않음도 함께 단언
- `src/parser/hwpx/header.rs::char_pr_id_gap_filler_gets_valid_relative_size`
  — id 0·3 만 있는 header.xml 의 갭 채움 자리(`resize_with`)도 유효범위 안

## 4. 문서 정합

Default 변경으로 **사실과 어긋나게 된 서술**을 고쳤다. 결론은 전부 유지된다 — 사라진 것은
부수 논거 하나뿐이다.

| 위치 | 처리 |
| --- | --- |
| `src/document_core/queries/hidden_text.rs` `effective_pt` 독 코멘트 | "덤으로 default `[0;7]` 오독 사고가 불가능해진다" 논거 소멸 → 이력 절로 명시. **판정 규칙은 불변** |
| 같은 파일 `default_relative_sizes_can_never_cause_a_zero_size_misjudgment` 주석 | 이 테스트가 지키는 것은 기본값이 아니라 **입력 내성**임을 명확히. 0 은 여전히 외부 파일로 들어온다(수정 이전 rhwp 산출물 전부) |
| `mydocs/report/task_sec_hidden/README.md` | 같은 문장 → 갱신 각주 추가 |
| `decision_log.md` D-11 / `invariants.md` INV-21 | **수정 불필요** — 기본값을 근거로 들지 않는다. "렌더러가 안 곱하므로 판정기도 안 곱한다"는 논지는 그대로 유효 |

## 5. 검증 게이트 결과

### 초점 테스트

| 대상 | 결과 |
| --- | --- |
| `issue_4141_hwp3_relative_size_contract` (신규) | **5 passed** (0.61s) |
| `issue_3676_hwp3_convert_hancom_openable` | 5 passed |
| `hml_serializer` | 31 passed |
| `hidden_text_contract` | 24 passed |
| `hwp5_roundtrip_baseline` | 3 passed (38.71s) |
| `hwpx_roundtrip_baseline` | 4 passed |
| `--lib model::style` | 6 passed |
| `--lib parser::hwpx::header` | 51 passed (신규 2건 포함) |

### IR field sweep — 논증이 아니라 실측으로 확인

계획서 §7 은 3 lane 전부 무변동을 **논증**으로 예측했다. 덤프를 실제로 떠서 확인했다.

```bash
RHWP_IR_SWEEP_DUMP=<dump> cargo test --profile release-test \
  --test ir_field_sweep_baseline -- --nocapture
```

```text
test ir_field_sweep_does_not_regress ... ok
test baseline_samples_exist ... ok
test result: ok. 2 passed; 0 failed; finished in 42.27s
```

덤프 대조 — 줄 수 598 = 598, **줄바꿈 정규화 후 SHA-256 일치**:

```text
baseline 68e716d35d63efef...
current  68e716d35d63efef...
diff -u --strip-trailing-cr → 빈 출력
```

(정규화 없는 `diff` 는 전 행 차이로 보이는데, 저장소 checkout 이 CRLF 이고 덤프가 LF 라서다.
내용 차분은 0 이다.)

무변동의 이유는 lane 별로 다르다:

- `hwp5` — `hwp5_out_of_scope` 가 HWP3 를 제외하고, HWP5 는 `raw_data` 우선이라
  `serialize_char_shape` 가 호출되지 않는다
- `hwp5rb` — `src/diagnostics/ir_field_sweep.rs:1045-1049` 가 `raw_stream_dirty`/`raw_stream` 만
  건드리고 **`CharShape.raw_data` 는 지우지 않는다**
- `hwpx` — Default 변경이 왕복 양쪽에 대칭 작용한다 (0→"0"→0 이 100→"100"→100 이 될 뿐)

### 포맷

`cargo fmt --check` 의 `Diff in` 위반은 이번에 만진 3파일뿐이었고 rustfmt 로 해소했다.
`Incorrect newline style` 경고는 저장소 전역 선행 상태다(Windows CRLF checkout) — 이번 변경과
무관하며, staged blob 은 CR 0바이트로 LF 정규화됨을 확인했다.

### clippy

```bash
CARGO_INCREMENTAL=0 cargo clippy --profile release-test --lib --tests -- -D warnings
```

**exit 0, 경고 0.**

전체 `cargo test --profile release-test --tests` 와 `cargo clippy --all-targets` 는
`docs_and_git_workflow.md:181-184` 에 따라 **작업지시자 승인 후** 실행한다.

renderer lane(`local_validation.md:74`)은 실행하지 않는다 — `relative_sizes` 는 렌더 경로 참조가
0건이고 렌더러가 소비하는 `ratios`·`base_size` 는 건드리지 않았다. 그 사실을
`char_shape_default_matches_spec_only_for_relative_sizes` 가 코드로 고정한다.

## 6. 남은 것 (Stage 3)

- 한컴 판정 번들 (`output/issue_4141/`, 비커밋) 과 바이트 사전검증
- 최종 보고서
- `ratios` 후속 이슈 등록 (Stage 1 §7 근거)
