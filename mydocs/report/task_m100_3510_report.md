---
kind: report
status: active
canonical: mydocs/report/task_m100_3510_report.md
last_verified: 2026-07-29
---

# #3510 처리 기록 — HWP3 파서 char_count 끝 마커 규약 불일치

## 원인

`para.char_count` 는 HWP5(`src/parser/body_text.rs`)와 HWPX(`src/parser/hwpx/section.rs`)
에서 문단 끝 마커(0x000D)를 포함해 저장한다. HWP3(`src/parser/hwp3/mod.rs`)만 이 마커를
빼고 계산했다:

```rust
// 이전
para.char_count = utf16_pos;
```

그 결과 HWP3 → HWPX 변환은 **내용이 완전히 같은데도** 사실상 모든 문단·표 셀에서
`char_count` 가 정확히 1씩 어긋났고, `export-hwpx --verify` 게이트가 정상 변환을
exit 3(IR 차이 감지)으로 거부했다.

## 수정

`src/parser/hwp3/mod.rs` 안의 두 문단-텍스트 생성 지점(원본 파싱 경로, 제목차례 장식
inject 경로) 모두 HWP5/HWPX 와 같은 +1 규약으로 맞췄다:

```rust
para.char_count = utf16_pos + 1; // +1 for 끝 마커 (HWP5/HWPX 규약과 정합, #3510)
```

## 검증 — 전/후

`samples/hwp3-sample.hwp` → HWPX 라운드트립, `ir-diff --json`:

| | 수정 전 | 수정 후 |
|---|---|---|
| `diffCount` | 298 | 16 |
| `cc`(char_count) 카테고리 | 298건, 전부 `\|A-B\|==1` | 11건, 전부 `\|A-B\|!=1` (아래 참고) |
| `export-hwpx --verify` exit code | 3 | 3 (아래 참고) |

exit 3 은 수정 후에도 남는다 — 그러나 원인이 바뀌었다. 남은 16건은 #3510 과
무관한 **다른** 결함이다:

- `ctrl[0] type: A=pgnp vs B=secd`, `ctrl[1] type: A=foot vs B=cold` — 구역 시작
  문단의 secd/cold 컨트롤 순서·개수가 라운드트립에서 바뀜(#3367 로 이미 별도 추적).
- 그 여파로 해당 문단의 `char_count`·`char_offsets`·`pos` 도 함께 어긋남(연쇄 결과).

이 잔여 결함이 이 표본에서 `--verify` exit 3 을 계속 유발하지만, **패턴이 완전히
다르다** — 수정 전에는 "예외 없이 모든 문단에서 정확히 1" 이었고, 수정 후에는
"구역 시작 문단 근처 소수(1개 문단, 연쇄 5줄)에서만, 값도 제각각" 이다. 즉 #3510 이
설명하던 결함 클래스는 사라졌다.

## 회귀 확인

- `samples/field-01.hwp`(HWP5) 라운드트립: `cc` 차이 0건, 그대로 유지.
- `cargo test --release --lib`: 2998 passed, 0 failed, 7 ignored.
- `cargo test --release --test hwp3_charcount_convention`: 4 passed.
- `cargo clippy --release --bin rhwp -- -D warnings`: 0 warnings.
- `cargo fmt --check`: 변경 파일 기준 clean(레포 전역 CRLF 노이즈 제외).

## 테스트

`tests/hwp3_charcount_convention.rs` 신설:

- `hwp3_roundtrip_char_count_is_not_off_by_exactly_one` — 본론. `cc` 라인이 하나라도
  `|A-B|==1` 이면 실패(끝 마커 규약 회귀 고정).
- `hwp3_roundtrip_char_count_diff_count_dropped_sharply` — 무회귀 가드. 수정 전
  298 이던 diffCount 가 50 미만으로 유지되는지 확인(#3367 류 잔여 결함은 허용).
- `hwp5_roundtrip_char_count_unaffected` — HWP5 경로에 새 회귀가 없는지 확인.

## 남은 일 (이 PR 범위 밖)

- `ctrl type: pgnp/foot vs secd/cold` — 구역 시작 문단 컨트롤 순서/개수 뒤집힘은
  #3367 로 이미 등록되어 있어 그쪽에서 처리한다.
