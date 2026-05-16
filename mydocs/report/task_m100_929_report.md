---
issue: 929
milestone: v1.0.0 (M100)
branch: local/task929
status: 완료 — 머지/푸시 대기
report_date: 2026-05-16
---

# Task #929 최종 결과 보고서

**[Bug] hwp3-sample19.hwp HWP3 파서 파싱 실패 (failed to fill whole buffer)**

## 1. 결과 요약

`samples/hwp3-sample19.hwp` 가 `HWP 3.0 오류: failed to fill whole buffer` 로 파싱 실패하던 결함을 수정. HWP3 spec §10.5 표 39 (탭 컨트롤 = 8 bytes, char_count 단위 4 hchar) 를 미준수하던 `parse_paragraph_list` 의 `9 =>` 분기를 정합 처리로 고침.

## 2. 원인 (확정)

`src/parser/hwp3/mod.rs` 의 본문 char loop 의 `ch == 9` (탭) 분기가 spec §10.5 표 39 를 따르지 않아 2 bytes / 1 hchar 만 소비. 실제 탭 구조는 다음 8 bytes:

| offset | 자료형 | 의미 |
|--------|--------|------|
| 0 | hchar | 특수 문자 코드 (=9) |
| 2 | hunit | 탭 폭 |
| 4 | word | 점끌기 여부 |
| 6 | hchar | 특수 문자 코드 닫기 (=9) |

탭 8 bytes = 4 hchar 차지하므로 `char_count` 에서도 4 카운트.

**파급**: cursor 가 6 bytes 일찍 진행 → 탭 내부 `hunit` 값(예: 0x0011=17) 을 다음 hchar 로 잘못 read → ch=17 (각주/미주) 컨트롤로 해석 → 각주 14B 식별 + `parse_paragraph_list` 재귀 호출 → garbage 위치에서 ParaInfo 읽음 → `LineInfo::read` EOF.

## 3. 수정

`src/parser/hwp3/mod.rs` 의 `9 =>` 분기를 spec 정합 처리로 (기존 코드의 `7|8` 날짜 컨트롤과 동일 패턴):

```rust
9 => {
    // [#929] HWP3 spec §10.5 표 39: 탭 = 8 bytes
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

## 4. 검증

| 항목 | 결과 |
|------|------|
| `issue_929_hwp3_sample19_parses_ok` | ✅ pass |
| `issue_929_existing_hwp3_samples_no_regression` (sample, 10, 11, 13, 14, 16) | ✅ pass |
| `cargo test --release` 전체 | ✅ FAILED 0 |
| `rhwp dump samples/hwp3-sample19.hwp` | ✅ 545 lines, err=0 |
| 기존 hwp3 samples 6 종 `rhwp dump` | ✅ 모두 err=0 |
| `rhwp ir-diff sample19-hwpx vs sample19` --summary | 509 건 (HWP3↔HWPX IR 표현 차로 본 task 범위 밖) |
| `cargo clippy --all-targets -- -D warnings` | ❌ 51 errors (모두 `src/wasm_api/tests.rs:11467` unused Result; task929 변경분 외부 — `local/devel` 기존 결함, 별 이슈 권고) |

## 5. 단계별 진행

| Stage | 커밋 | 내용 |
|-------|------|------|
| 사전 | `ece18fec` | 수행 계획서 + hwp3-sample19 fixture 추가 |
| Stage 1 | `11718e80` | 진단 출력 추가 + 실패 지점 식별 (재귀 진입 시 garbage ParaInfo) |
| Stage 2 | `a372069b` | 원인 확정 (ch=9 spec §10.5 미준수) + 회귀 가드 테스트 (의도된 실패) |
| Stage 3 | `e2de8ad8` | ch=9 spec 정합 처리 + 진단 제거 + 전체 검증 |
| Stage 4 | (생략) | 시각 검증 — 작업지시자 결정으로 생략 (Stage 3 검증으로 충분) |
| Stage 5 | (본 보고서) | 최종 보고서 + orders 갱신 |

## 6. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/parser/hwp3/mod.rs` | `9 =>` 분기 정합 처리 (+10/-4) |
| `tests/issue_929.rs` | 회귀 가드 테스트 신규 |
| `samples/hwp3-sample19{,-hwp5}.hwp`, `samples/hwp3-sample19-hwpx.hwpx` | fixture (task902 와 중복 가능, 동일 바이너리) |
| `pdf/hwp3-sample19-hwp5-2022.pdf` | 한글 2022 권위 PDF |
| `mydocs/plans/task_m100_929{,_impl}.md` | 수행/구현 계획서 |
| `mydocs/working/task_m100_929_stage{1,2,3}.md` | 단계별 보고서 |
| `mydocs/report/task_m100_929_report.md` | 본 보고서 |
| `mydocs/orders/20260516.md` | 오늘할일 갱신 |

## 7. 후속 과제 (별 이슈 후보)

1. **`src/wasm_api/tests.rs:11467` 의 unused Result 51 호출** — `cargo clippy --all-targets -- -D warnings` 차단. `local/devel` 기존 결함이므로 별 이슈 등록 후 일괄 `let _ = …` 처리 권장.
2. **HWP3 ↔ HWPX ir-diff 의 char_offsets / char_shapes 차이** — IR 표현 차이로 본 task 범위 밖이지만 향후 정합도 향상 기회. 데이터가 모이면 별 이슈로.
3. **HWP3 char loop 의 다른 미준수 컨트롤 검토** — `ch=9` 와 동일 패턴 결함이 다른 컨트롤에도 존재할 가능성 (코드상 `ch=7,8` 등은 이미 처리되어 있으나, 다른 컨트롤 spec 정합도 점검 권장).

## 8. 머지/푸시 계획

- 본 task 브랜치: `local/task929` (HEAD=e2de8ad8 + 본 보고서 commit 예정)
- 머지 대상: `local/devel` (로컬), 그 후 작업지시자 판단에 따라 `devel` push.
- Stage 3 commit 메시지에 `Closes #929` 포함 — `devel` push 시 자동 issue close.

---

## 승인 요청

Task #929 모든 단계 완료 보고드립니다. 최종 보고서 + orders 갱신 commit 후 다음 사항 결정 부탁드립니다:

1. **`local/task929` → `local/devel` 머지** 진행 여부
2. **`devel` push** 진행 여부 (push 는 별도 승인 필요 — MEMORY 룰)
3. **후속 별 이슈 등록 여부** (clippy 51 errors / ir-diff 차이)
