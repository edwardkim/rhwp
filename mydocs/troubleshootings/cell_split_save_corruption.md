---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# 셀 나누기 후 저장 시 파일 손상

## 증상

rhwp-studio에서 표 생성 → 셀 나누기 → 저장하면 한컴오피스에서 "파일이 손상되었습니다" 오류가 발생한다.
한컴에서 손상 파일을 열고 다른 이름으로 저장하면 오류가 사라진다.

## 테스트 파일

| 파일 | 설명 | 상태 |
|------|------|------|
| `saved/tb-err-003.hwp` | 표 생성 → 셀 나누기 → 저장 | **손상** |
| `saved/tb-err-003-s.hwp` | 손상 파일을 한컴에서 다른 이름 저장 | 정상 |

## 원인 분석

### 파일 비교 결과

두 파일의 **표 구조(Table, Cell)는 완전히 동일**했다:
- 3행×5열, 11개 셀
- row_sizes, col_count, row_count 모두 정합
- 셀별 col/row/col_span/row_span/width/height 동일

**차이점은 표 바깥의 문단 수**였다:

| | 손상 파일 (5 문단) | 정상 파일 (3 문단) |
|---|---|---|
| 문단[0] | SectionDef+ColumnDef | 동일 |
| 문단[1] | Table(3×5) | 동일 |
| **문단[2]** | **빈 문단 (ctrl_mask=0x00)** | 마지막 문단 (msb=true) |
| **문단[3]** | **고아 문단 (아래 참조)** | - |
| 문단[4] | 마지막 문단 (msb=true) | - |

### 근본 원인: 고아 문단의 control_mask 불일치

문단[3]의 상태:

| 필드 | 값 | 문제 |
|------|------|------|
| `control_mask` | `0x00000800` (TABLE 비트) | 실제 controls 배열은 **빈 배열** |
| `has_para_text` | `true` | 텍스트도 컨트롤도 없는 **빈 문단** |
| `char_count` | `1` | 문단 끝 마커만 존재 |
| `controls` | `[]` | 빈 배열 |

이 문단이 직렬화되면:

```
[손상 파일 - 문단[3]의 직렬화 결과]
PARA_HEADER: cc=1, ctrl_mask=0x800   ← TABLE 제어 레코드가 있을 것으로 선언
PARA_TEXT: [0D 00]                    ← 빈 문단인데 PARA_TEXT 존재 (cc=1에 PARA_TEXT → 손상)
PARA_CHAR_SHAPE
PARA_LINE_SEG
(CTRL_HEADER 없음)                   ← ctrl_mask=0x800인데 TABLE 레코드 부재
```

한컴의 파서는:
1. `ctrl_mask=0x800`을 보고 TABLE 컨트롤 레코드를 기대하지만 존재하지 않음
2. `cc=1`(빈 문단)에 PARA_TEXT가 있으면 레코드 구조 불일치로 판단

두 가지 불일치가 동시에 발생하여 파일 손상으로 판정된다.

### 고아 문단의 발생 경로

편집 과정에서 문단의 `controls` 배열이 변경되었지만 `control_mask` 필드가 갱신되지 않은 채로 남은 경우 발생한다. 예를 들어:
- 기존 표 문단에서 표 컨트롤이 제거되었지만 `control_mask=0x800`이 유지됨
- 문단 분할/복사 시 원본의 `control_mask`가 상속되었지만 `controls`는 비어있음

## 수정 내용

### [FIX-1] control_mask 재계산 (직렬화 시)

**파일**: `src/serializer/body_text.rs`

모델의 `control_mask` 값을 그대로 사용하지 않고, 직렬화 시점에 **실제 controls 배열에서 재계산**한다.

```rust
/// 실제 controls에서 control_mask 비트를 계산한다.
fn compute_control_mask(controls: &[Control]) -> u32 {
    let mut mask: u32 = 0;
    for ctrl in controls {
        let (char_code, _) = control_char_code_and_id(ctrl);
        mask |= 1u32 << char_code;
    }
    mask
}
```

비트 매핑:
- 0x0002 (SectionDef, ColumnDef) → bit 2 = 0x04
- 0x000B (Table, Shape, Picture) → bit 11 = 0x800
- 0x0010 (Header, Footer) → bit 16 = 0x10000
- 기타 컨트롤도 char_code 기반으로 자동 매핑

이로써 `controls=[]`이면 `control_mask=0`이 되어 한컴 파서와 일치한다.

### [FIX-2] has_para_text 보정 (직렬화 시)

**파일**: `src/serializer/body_text.rs`

빈 문단(텍스트 없고 컨트롤 없음, char_count ≤ 1)에 대해 PARA_TEXT 레코드를 쓰지 않도록 보정한다.

```
[수정 전]
if !para.text.is_empty() || !para.controls.is_empty() || para.has_para_text {
    // has_para_text=true이면 빈 문단에도 PARA_TEXT 기록 → 한컴이 거부

[수정 후]
let has_content = !para.text.is_empty() || !para.controls.is_empty();
if has_content || (para.has_para_text && para.char_count > 1) {
    // 실제 콘텐츠가 있거나, char_count > 1인 경우에만 PARA_TEXT 기록
```

이 수정은 FIX-3(엔터 2회 후 저장 손상, `table_paste_file_corruption.md` 참조)의 직렬화 측 방어 계층으로, `split_at()` 이외의 경로에서 발생하는 `has_para_text` 불일치도 방지한다.

## 수정 파일

| 파일 | 변경 |
|------|------|
| `src/serializer/body_text.rs` | `compute_control_mask()` 추가, `serialize_paragraph_with_msb()` 수정 |

## 검증

```
[수정 전 - 문단[3] 재직렬화 결과]
PARA_HEADER: ctrl_mask=0x800, cc=1
PARA_TEXT: [0D 00]           ← 불필요
→ 한컴 "파일 손상" 오류

[수정 후 - 문단[3] 재직렬화 결과]
PARA_HEADER: ctrl_mask=0x000, cc=1
(PARA_TEXT 없음)             ← 빈 문단에 맞게 생략
→ 한컴 정상 오픈
```

- 582개 테스트 전부 통과
- WASM 빌드 성공
- TypeScript 컴파일 성공
- 진단 테스트(`test_diag_tb_err_003`): 재직렬화 후 control_mask/has_para_text 불일치 0건
- 한컴오피스에서 정상 오픈 확인

## 관련 문서

- `mydocs/troubleshootings/table_paste_file_corruption.md` — FIX-3 (엔터 2회 후 저장 손상: `has_para_text` 문제의 최초 발견)
- `mydocs/plans/task_135.md` — 셀 나누기 기능 구현 계획서
