# Task M100 #1407 — 1단계 완료 보고서 (슬롯/fieldEnd 우선순위 정정)

- 브랜치: `local/task1407`
- 작성일: 2026-06-14
- 수정 파일: `src/serializer/hwpx/section.rs`

## 1. 구현계획서 대비 정정 — 실제 결함 지점은 post-char fieldEnd

구현계획서는 "pre-char fieldEnd-자리 가드"를 제안했으나, 단위 추적(`RHWP_DBG_1407`)으로
**실제 가로채기 지점이 한 idx 앞**임을 확인했다:

문단 0.14 메인 루프 추적 (수정 전):
```
idx=2 c='훈' char_pos=18 expected=18 → 문자 push 후 expected=19,
       post-char fieldEnd(end_char_idx=3==next_idx=3) 방출 — 그러나 expected 미증가(19)
idx=3 c=' ' char_pos=27 expected=19 → 27 ≥ 19+8=27 → slot[2]=NewNumber 방출 (가로채기!)
```

즉 **post-char fieldEnd 방출이 `expected_utf16_pos`를 +8 진행하지 않아**, 다음 idx 에서
텍스트-끝 슬롯(newNum)이 그 8유닛 갭을 차지했다. pre-char 가드는 이미 emitted 된
fieldEnd 라 작동하지 않았다.

## 2. 수정 (1줄 본질)

`render_runs` post-char fieldEnd 방출(`section.rs:540` 부근)에 `expected += 8` 추가:

```rust
emit_field_end(&mut splitter.content, para, fr.control_idx);
// [#1407] fieldEnd 는 8유닛 슬롯을 소비한다. expected 를 +8 진행하지 않으면
// 다음 idx 에서 텍스트-끝 슬롯(newNum 등)이 이 8유닛 갭을 가로챈다.
expected_utf16_pos = expected_utf16_pos.saturating_add(8);
field_end_emitted[i] = true;
```

이제 idx=3 에서 expected=27, char_pos=27 → 27 ≥ 27+8? No → newNum 미방출. NewNumber 는
루프 후 남은 슬롯 일괄 방출(line 559 부근)로 텍스트 끝에 배치 → 원본 XML 동형.

## 3. 검증

- 143E 문단 0.14 ir-diff: **차이 0** (수정 전 char_offsets[3] 27→35).
- 143E 전체 문서 ir-diff: **차이 0**.
- RT XML 확인: `…<hp:t> 기자(…)</hp:t><hp:ctrl><hp:newNum/></hp:ctrl>` — newNum 이
  텍스트 뒤(끝)로 정상 복원.
- 단위 테스트 `task1407_field_end_not_stolen_by_newnum_slot`: 방출 XML 에서
  fieldBegin < fieldEnd < " DEF" < newNum 순서 봉인. **passed**.
- `serializer::hwpx::section` 39 passed (기존 #1298/#1378/#1321 회귀 없음).
- `cargo test --test hwpx_roundtrip_baseline` 4 passed — **B=0 유지**.

## 4. 다음 단계

- 2단계: 전수 배치(`--batch`) 신규 실패 0 확인 + **143E RT 페이지 수 1→2 재측정**
  (증상 ②) → 슬롯 정정으로 해소되는지 귀속 확정. CI급.
- 3단계: 문서 + 보고서.
