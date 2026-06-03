# Task #537 Stage 2 완료 보고서

**제목**: lazy_base trailing-ls 보정 적용 (A'안)
**브랜치**: `local/task537`
**이슈**: https://github.com/edwardkim/regex/issues/537

---

## 1. 코드 변경

`src/renderer/layout.rs:1494-1521` (lazy_base 산출 분기)

```rust
} else {
    // [Task #537] trailing-ls 보정:
    // paragraph_layout 의 마지막 줄은 trailing line_spacing 을
    // 제외하여 y 를 advance 한다 (Task #479, lh_sum + (n-1)*ls 정책).
    // 그 결과 sequential y_offset 은 IR vpos 누적보다
    // prev_pi 의 last seg ls 만큼 부족해진다.
    // 이 부족분을 y_delta_hu 에 더해야 lazy_base 가
    // IR 절대 좌표와 일치한다 (drift 가 base 에 동결되는 것을 방지).
    let trailing_ls_hu = paragraphs.get(prev_pi)
        .and_then(|p| p.line_segs.last())
        .map(|s| s.line_spacing.max(0))
        .unwrap_or(0);
    let y_delta_hu = ((y_offset - col_area.y) / self.dpi * 7200.0).round() as i32
        + trailing_ls_hu;
    let lazy_base = prev_vpos_end - y_delta_hu;
    ...
}
```

LOC: +14 / -2.

## 2. 검증 결과

### 2.1 단위 테스트
```
test renderer::layout::integration_tests::tests::test_537_first_answer_after_tac_table_line_spacing ... ok
```
Stage 1 에서 작성한 TDD 테스트 Green 으로 전환.

### 2.2 전체 테스트 회귀
```
test result: ok. 1117 passed; 0 failed; 1 ignored
```
1116 (이전) + 1 (Task #537 신규) = 1117. 회귀 0건.

### 2.3 본 task 직접 대상 11곳 정량 검증

페이지별 ①→② gap 측정 (수정 전/후, 단위 px):

| 페이지 | 문제 | 라인수 | 수정 전 | 수정 후 | IR vpos 기대 | 상태 |
|--------|------|--------|---------|---------|--------------|------|
| P2 | 3번 | 3 | 63.09 | **72.64** | 72.64 | ✅ |
| P3 | 6번 | 3 | 63.09 (col 1) | **72.64** | 72.64 | ✅ |
| P5 | 9번 | 1+2 | 14.67 | **24.21** | 24.21 | ✅ |
| P6 | 12번 | 2 | 38.88 | **48.43** | 48.43 | ✅ |
| P8 | 15번 | 2 | 38.88 | **48.43** | 48.43 | ✅ |
| P9 | 17번 | 2 | 38.88 | **48.43** | 48.43 | ✅ |
| P9 | 18번 | 2 | 38.88 | **48.43** | 48.43 | ✅ |
| P12 | 23번 | 2 | 38.88 | **48.43** | 48.43 | ✅ |
| P12 | 24번 | 2 | 38.88 | **48.43** | 48.43 | ✅ |
| P13 | 27번 | 2 | 38.88 | **48.43** | 48.43 | ✅ |
| P14 | 29번 | 2 | 38.88 | **48.43** | 48.43 | ✅ |

모든 ①→② gap 이 ②→③ gap 과 동일해졌으며 IR vpos delta 와 일치.

### 2.4 VPOS_CORR 디버그 비교

수정 전:
```
VPOS_CORR pi=39 prev_pi=38 base=716 ...    ← drift 동결
VPOS_CORR pi=40 prev_pi=39 base=716 ...
VPOS_CORR pi=41 prev_pi=40 base=716 ...
VPOS_CORR pi=42 prev_pi=41 base=716 ...
```

수정 후:
```
VPOS_CORR pi=39 prev_pi=38 base=0 ...       ← IR 절대 좌표 직접 사용
VPOS_CORR pi=40 prev_pi=39 base=0 ...
VPOS_CORR pi=41 prev_pi=40 base=0 ...
VPOS_CORR pi=42 prev_pi=41 base=0 ...
```

전 문서 base=0 카운트: 139 → **149** (+10) — drift 해소.

### 2.5 잔존 base=716 분석 (회귀 아님)

수정 후에도 base=716 이 일부 paragraph 에 남아있음 (pi=147~149, pi=214~220 등):
- 동일 케이스가 **수정 전에도 base=716** 이었음 (회귀 아님)
- 발생 위치: prev_pi 의 line_segs.last() 의 segment_width 가 0 등 특수 조건. 본 task 의 trailing-ls 보정이 적용 안 되거나 다른 메커니즘.
- 이들은 작업지시자가 명시한 11곳 외 위치이며, 사용자 보고에 포함되지 않음 → 본 task 범위 밖. 발견된 잔존 케이스는 별도 issue 검토 후보.

### 2.6 Clippy

```
error: this call to `unwrap()` will always panic
  --> src/document_core/commands/table_ops.rs:1007:17
  --> src/document_core/commands/object_ops.rs:298:21
```

**기존 결함** (Stage 1 시작 전 commit `local/task537@226b6446` 에서 이미 존재).
본 task 코드(`layout.rs`) 변경 후 추가되지 않았음 (git stash 검증).
별도 issue 처리 권장.

## 3. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/layout.rs` | lazy_base 산출 trailing-ls 보정 (+14 / -2) |
| `mydocs/working/task_m100_537_stage2.md` | 본 보고서 |

## 4. 다음 단계 (Stage 3)

광범위 회귀 검증:
- `samples/synam-001.hwp` (TAC 표 다수)
- `samples/복학원서.hwp` (BehindText/Square wrap, lazy_base 의 다른 사용처)
- 추가 표/그림 포함 샘플 5~8 종
- 한컴 2010 / 2020 / 한컴독스 PDF 200dpi 시각 비교

회귀 발견 시 가드 조건 추가 검토. 회귀 없으면 최종 보고서 작성 후 merge 승인 요청.

## 5. 승인 요청

Stage 2 완료. Stage 3 진행 승인 요청.
