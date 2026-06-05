# 최종 결과 보고서 — Task M100 #1298

## 개요

| 항목 | 내용 |
|------|------|
| 이슈 | #1298 — HWPX 시리얼라이저: 빈 필드(0-length field range) fieldBegin/fieldEnd 인터리빙 오류 |
| 브랜치 | `local/task1298` |
| 마일스톤 | M100 |
| 커밋 | `1f25588f` (수정+테스트), `273dbc1f` (보고서) |

---

## 배경

PR #1289에서 HWPX 시리얼라이저의 Bookmark/Field dispatcher를 연결했다. 리뷰에서 "빈 필드(0-length field range)에서 fieldBegin/fieldEnd 인터리빙을 더 정교하게 보완할 여지가 있음"이 지적됐고, 이번 이슈 #1298에서 처리했다.

---

## 수정 내용

**파일**: `src/serializer/hwpx/section.rs` — `render_run_content` 함수

### 결함 1: `end_char_idx == 0` → fieldEnd가 텍스트 뒤로 밀림

기존 post-char fieldEnd 방출 조건 `fr.end_char_idx == next_idx`는 `next_idx = idx + 1 ≥ 1`이므로 `end_char_idx == 0`을 루프 내에서 영원히 잡지 못했다. 결과적으로 fieldEnd가 post-loop에서 모든 텍스트 뒤에 방출됐다.

### 결함 2: 0-length mid-text 필드 (`start == end == N > 0`) → fieldEnd/fieldBegin 순서 역전

기존 post-char 검사가 idx=N-1 시점(문자 N-1 처리 후)에 fieldEnd를 방출하고, fieldBegin은 idx=N 시점(슬롯 시스템)에 방출됐다. 결과: `...text[N-1] fieldEnd fieldBegin text[N]...` — 순서 역전.

### 수정

**변경 1**: 슬롯 방출(fieldBegin) 직후, `text_buf.push(c)` **직전**에 0-length 필드 전용 pre-char 검사 추가.

```rust
// 0-length 필드(start == end == idx): fieldBegin 방출 직후, 문자 push 전에 fieldEnd 방출.
for (i, fr) in para.field_ranges.iter().enumerate() {
    if fr.start_char_idx == fr.end_char_idx
        && fr.end_char_idx == idx
        && !field_end_emitted[i]
    {
        flush_text_fragment(...);
        // emit fieldEnd
        field_end_emitted[i] = true;
    }
}
```

**변경 2**: 기존 post-char 검사에 `fr.start_char_idx < fr.end_char_idx` guard 추가 — 0-length 필드가 idx-1 시점에 역순 방출되는 것 차단.

---

## 수정 전후 비교

| 케이스 | 수정 전 | 수정 후 |
|--------|---------|---------|
| `start=0, end=0` | `<fieldBegin/>` text `<fieldEnd/>` | `<fieldBegin/><fieldEnd/>` text |
| `start=N, end=N` (N>0) | text[N-1] `<fieldEnd/>` `<fieldBegin/>` text[N] | text[0..N] `<fieldBegin/><fieldEnd/>` text[N..] |
| `start=0, end=5` (정상) | `<fieldBegin/>` text `<fieldEnd/>` | `<fieldBegin/>` text `<fieldEnd/>` (불변) |

---

## 테스트

신규 테스트 2개 추가, 기존 테스트 12개 전부 회귀 없음.

| 테스트 | 내용 |
|--------|------|
| `task1298_zero_length_field_at_para_start` | start=0, end=0: fieldBegin < fieldEnd < text 순서 검증 |
| `task1298_zero_length_field_mid_text` | start=3, end=3: ABC < fieldBegin < fieldEnd < DE 순서 검증 |

```
test result: ok. 14 passed; 0 failed
```

`cargo clippy --lib -- -D warnings`: 경고 0건

---

## 범위 외 사항

빈 문단(`text == ""`) + 0-length 필드 조합에서는 루프 자체가 실행되지 않아 post-loop에서 fieldEnd → fieldBegin 역순 방출이 발생하는 기존 결함이 남아 있다. 이 케이스는 실사용에서 발생 빈도가 매우 낮고 별도 분석이 필요하므로 후속 이슈로 처리한다.
