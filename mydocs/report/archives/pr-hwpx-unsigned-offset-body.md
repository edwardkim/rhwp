## 요약

`#3544` — HWPX 재저장 시 OWPML 스키마상 unsigned 인 `<hp:offset>` 의 `x`/`y` 에
음수가 기록되던 문제를 고칩니다. 저장기의 `write_offset` 한 곳을 고쳐
`samples/hwpx` 81종 재저장 기준 위반 **64건 → 0건**이 됩니다.

Fixes #3544

## 근인 — 클램프 문제가 아니라 인코딩 비대칭

이슈에서 미확인으로 남겨 두신 "음수가 생성되는 지점"을 특정했습니다. 음수는
레이아웃이 잘못 계산한 값이 아니라, **한컴이 의도적으로 쓴 값을 rhwp 가 잘못된
표기로 되돌려 쓴 것**이었습니다.

한컴은 음수 오프셋을 u32 wraparound 십진수로 기록합니다(예: `-2429` →
`"4294964867"`). rhwp 파서도 같은 관례로 복호합니다.

```rust
// src/parser/hwpx/section.rs
b"x" => { let v = parse_u32(&attr); shape_attr.offset_x = v as i32; ... }
```

즉 **복호(파서)는 있는데 대응하는 부호화(저장기)가 없었습니다.** `write_offset` 이
IR 의 signed 값을 그대로 `to_string()` 한 탓에, 파서가 `4294964867` → `-2429` 로
읽어들인 값이 저장 때 `-2429` 문자열로 나가 XSD 를 위반했습니다.

그래서 0 클램프가 아니라 **부호화 복원**으로 고쳤습니다. 클램프였다면 한컴이 기록한
음수 오프셋 정보가 소실되어 그룹 내부 좌표가 틀어집니다. IR 이 `i32` 인 것은 레이아웃
계산상 정당하므로 IR 은 그대로 두고, XML 경계에서만 파서 복호의 역함수를 적용합니다.

## 변경

`src/serializer/hwpx/shape.rs` 의 `write_offset`:

```rust
let x = (sa.offset_x as u32).to_string();
let y = (sa.offset_y as u32).to_string();
```

같은 `hp:offset` 을 쓰는 `src/serializer/hwpx/picture.rs` 는 IR 필드가
`HwpUnit = u32` 라 음수를 방출할 수 없음을 확인했습니다 — 수정 지점은 이 한 곳뿐입니다.

## 검증

### red → green

수정 블록을 되돌린 상태에서 신규 테스트가 실제로 실패함을 먼저 확인했습니다.

| 게이트 | red (수정 전) | green (수정 후) |
|---|---|---|
| `tests/issue_3544_hwpx_unsigned_offset.rs` (실물 코퍼스) | FAILED — 음수 `hp:offset` 16건 방출 | ok |
| `serializer::hwpx::shape::tests::issue3544_…` (단위) | FAILED — `x="-8974" y="-2"` | ok |

### 코퍼스 실측 (samples/hwpx 81종)

| 대상 | 위반 문서 | 위반 건수 |
|---|---:|---:|
| 원본 (한컴 산출) | 0 | **0** |
| 재저장 — 수정 전 | 17 | **64** |
| 재저장 — 수정 후 | 0 | **0** |

수정 전 64건은 이슈에 보고된 건수(64건)와 일치합니다.

### 회귀

- `cargo test --profile release-test --lib` — **3016 passed, 0 failed**
- `rhwp export-hwpx <각 샘플> --verify` — **81/81 통과** (IR 왕복 동일).
  값이 아니라 표기만 바뀌었음을 뒷받침합니다.
- `rustfmt --check` (변경 파일), `cargo clippy --profile release-test --bin rhwp -- -D warnings` — 통과

### 시각 검증

**N/A.** XML 속성의 어휘 표기만 바뀌고 디코드 결과 정수값은 동일합니다(`--verify`
81/81 로 IR 왕복 동일 확인). 렌더 경로에 입력되는 좌표가 변하지 않아 렌더 산출물
비교 대상이 없습니다.

## 테스트 설계 노트

자체 왕복 `--verify` 는 이 버그를 잡지 못합니다 — rhwp 파서가 음수 표기도 관대하게
읽어 IR 이 왕복상 일치해 버리기 때문입니다(수정 전에도 `--verify` 는 exit 0). 그래서
신규 테스트는 IR 이 아니라 **방출된 XML 문자열 자체를 계약으로 고정**하고, 두 가지를
함께 단언해 향후 "0 클램프" 류 회귀도 막습니다.

1. 저장본 `hp:offset` 에 음수 십진수가 0건일 것 (XSD unsigned)
2. 원본의 wraparound 값이 저장본에도 남아 있을 것 (클램프였다면 전멸)

## 남은 범위

이번 수정은 이슈가 제시한 `hp:offset` 계열을 닫습니다. 다른 unsigned 좌표 속성에
같은 인코딩 비대칭이 있는지는 위 코퍼스 검증 범위 밖이며, 발견되면 별도 이슈로
다루는 편이 변경 범위를 좁게 유지한다고 판단했습니다.

처리결과 문서: `mydocs/report/pr-hwpx-unsigned-offset.md`

🤖 Generated with [Claude Code](https://claude.com/claude-code)
