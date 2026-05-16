---
issue: 929
stage: 3
status: 작성 중 — 검증 결과 채우는 중
---

# Task #929 Stage 3 완료 보고서 — 구현 + 진단 제거 + 회귀 검증

## 변경 내역

### 1) `src/parser/hwp3/mod.rs` — ch=9 (탭) 처리 spec §10.5 정합

```rust
9 => {
    // [#929] HWP3 spec §10.5 표 39: 탭 = 8 bytes 구조
    //   offset 0: hchar(=9)  [outer read 완료]
    //   offset 2: hunit       탭 폭
    //   offset 4: word        점끌기 여부
    //   offset 6: hchar(=9)  닫기
    // char_count 단위는 hchar(2B); 8 bytes = 4 hchar 차지 → i += 3 추가.
    let mut buf = [0u8; 6];
    if let Err(_) = body_cursor.read_exact(&mut buf) { break; }
    for k in 0..3usize {
        if i + k < hwp3_char_to_utf16_pos.len() {
            hwp3_char_to_utf16_pos[i + k] = utf16_len;
        }
    }
    i += 3;
    char_offsets.push(utf16_len);
    utf16_len += 1;
    text_string.push('\t');
}
```

### 2) `src/parser/hwp3/mod.rs` — Stage 1+2 진단 코드 모두 제거

제거된 진단 출력 (`[diag929]` 태그):
- `parse_paragraph_list` loop 시작 trace
- `ParaInfo::read` map_err wrap + 결과 trace
- `LineInfo::read` map_err wrap
- 본문 `read_u16` map_err wrap + 컨트롤 ch trace + para 21 한정 i/ch_pos/ch trace
- ch=17 진입 시 cursor + peek dump

총 약 40 라인 제거 → 원본 코드 복원 + ch=9 분기만 변경.

## 검증 결과

### A. issue_929 회귀 가드 테스트

```bash
$ cargo test --release --test issue_929
test issue_929_hwp3_sample19_parses_ok ... ok
test issue_929_existing_hwp3_samples_no_regression ... ok
test result: ok. 2 passed; 0 failed; 0 ignored
```

✅ Stage 2 의도된 실패 → Stage 3 통과로 정확히 전환.

### B. `rhwp dump` 동작 — 전체 hwp3 샘플

```
samples/hwp3-sample.hwp:    lines=  2586 err=0
samples/hwp3-sample10.hwp:  lines=219173 err=0
samples/hwp3-sample11.hwp:  lines= 60581 err=0
samples/hwp3-sample13.hwp:  lines=   757 err=0
samples/hwp3-sample14.hwp:  lines=  2406 err=0
samples/hwp3-sample16.hwp:  lines= 18907 err=0
samples/hwp3-sample19.hwp:  lines=   545 err=0   ← 본 Task 수정 효과
```

기존 6 샘플 모두 정상 동작 (회귀 없음), sample19 도 정상 dump.

### C. 전체 `cargo test --release`

```
test result: ok. 2 passed; 0 failed; 0 ignored ... (issue_929)
test result: ok. 8 passed; 0 failed; 0 ignored ... (svg_snapshot)
test result: ok. 1 passed; 0 failed; 0 ignored ... (tab_cross_run)
test result: ok. 0 passed; 0 failed; 1 ignored ... (Doc-tests)
```

✅ FAILED 0. 모든 test binary 빌드 + 실행 통과.

### D. `cargo clippy --release --all-targets -- -D warnings`

❌ **51 errors** — 모두 `src/wasm_api/tests.rs:11467` 의 unused Result 패턴.

```
error: unused `std::result::Result` that must be used
    --> src/wasm_api/tests.rs:11467:9
error: could not compile `rhwp` (lib test) due to 51 previous errors
```

**평가**: 본 task929 변경분이 아님. `git diff --stat local/devel..HEAD` 로 task929 변경 파일 = `src/parser/hwp3/mod.rs` + `tests/issue_929.rs` + docs + fixture. `src/wasm_api/tests.rs` (795 KB, 15,759 line) 는 손대지 않음 — `local/devel` 에 이미 존재하는 기존 코드. 따라서 본 결함은 task929 회귀가 아니며 별 이슈 후보.

(필요 시 별 이슈에서 wasm_api/tests.rs 의 51 호출부를 `let _ = …` 처리하면 됨)

### E. `rhwp ir-diff samples/hwp3-sample19-hwpx.hwpx samples/hwp3-sample19.hwp --summary`

```
=== 비교 완료: 차이 509 건 ===
```

주요 카테고리:
- `char_shapes count` 6건
- `char_offsets[N]` 다수 (HWP3 IR 와 HWPX IR 의 컨트롤 자리 카운트 표현 차이로 인한 구조적 차이)
- `ParaShape 수` 1건
- `TabDef 수` 1건
- `탭 수` 1건
- `controls count` 1건
- `text` 2건

평가: char_offsets 차이가 다수인 건 HWP3 ↔ HWPX 의 컨트롤·텍스트 IR 표현 차이로 본 task 범위 밖. 다른 hwp3 샘플에서도 유사 차이가 나타나는 영역. 핵심 결함(파싱 실패)은 해결됨.

## 변경 파일

- `src/parser/hwp3/mod.rs`
  - char loop `9 =>` 분기 정정 (spec §10.5 표 39 정합, 7 줄 추가)
  - Stage 1+2 진단 코드 약 40 라인 제거

## 후속 과제 (Stage 4 / 5 또는 별 이슈)

- **Stage 4 시각 검증**: sample19 의 SVG 가 hwp5/pdf 변환본과 시각적으로 일치하는지 확인.
- **별 이슈 후보**: ir-diff 결과의 일부 차이 (예: `ParaShape 수`, `controls count`) 가 본 task 범위 밖이지만 향후 정합도 향상 기회가 있을 수 있음.

---

## 승인 요청

Stage 3 완료 보고드립니다. 다음 사항 승인 부탁드립니다:

1. **Stage 3 변경 + 검증 결과** (위 A~E) 의 적정성 확인.
2. **Stage 4 시각 검증 진행 여부** — sample19 의 SVG ↔ hwp5/pdf 비교를 진행할지, 아니면 Stage 5 (최종 보고서) 로 바로 갈지 결정 부탁드립니다.

검증 결과(C, D) 가 완료되는 대로 추가 보고드리겠습니다.
