# Task #896 최종 결과 보고서 — sample16 페이지 18 추가 시각 정합

**이슈**: [edwardkim/rhwp#896](https://github.com/edwardkim/rhwp/issues/896)
**브랜치**: `local/task896` (base: `local/task894 @ ce8d3ce`)
**선행 task**: #894 (PR #897 MERGEABLE, 메인테이너 머지 대기), #877 (PR #890 MERGEABLE)
**기간**: 2026-05-14

## 1. 개요

Task #894 의 Stage 4 진단 중 발견된 두 가지 차이를 본 task 에서 통합 처리:
1. **차이 1**: paragraph 397/398/399 의 ◦ 글머리 표시 차이 (paragraph_layout/fixup)
2. **차이 2**: paragraph 394 picture (WMF) 안의 한글 텍스트 겹침/깨짐 (WMF converter)

## 2. 최종 결과

### 2.1 성과 요약

| Stage | 항목 | 결과 |
|-------|------|------|
| 1+2 | 차이 1 진단 정정 + Fix (apply_bullet_fixup_single dash skip) | ✅ paragraph 398/399 ◦ 제거 (한컴 정합) |
| 3+4 | 차이 2 진단 + Fix (WMF font encoding + Korean fallback) | ✅ 그림 안 한글 텍스트 정상 표시 |
| 5 | 통합 검증 | ✅ cargo test 1398 passed |

### 2.2 시각 정합 변화

| 항목 | 이전 | 이후 |
|------|------|------|
| paragraph 398/399 ◦ 글머리 | rhwp ◦ 표시, 한컴 미표시 | **한컴 정합 (미표시)** |
| WMF 그림 안 한글 텍스트 | 모자이크 깨진 글리프 | **정상 한글 표시** |

### 2.3 차이 1 진단 오류 정정

**이전 Task #894 Stage 4 의 진단**:
> paragraph 396 ○ x=117.81 vs paragraph 397~399 ◦ x=107.30 (-10.5 px)

**정정**: paragraph 396/397/398/399 의 ◦ **모두 동일 SVG x=107.30** — 차이 없음. 이전 비교의 x=117.81 은 paragraph 395 의 ○ (다른 paragraph 의 첫 글머리).

**진짜 root cause**: paragraph 398/399 의 ◦ 자체가 `fixup_hwp3_outline_bullets` (Task #877) 로 잘못 추가됨. paragraph 398 raw text 가 공백+dash 시작 ("   - 하드웨어...") 인데 한컴 변환기는 dash 시작 paragraph 에 ◦ 추가 안 함 (sub-item dash marker 이미 표시).

PDF 정합 분석:
```
paragraph 396/397/400: <text top=... left=132/133>◦</text>  (◦ char 있음)
paragraph 398/399:     (◦ char 없음 — 한컴 viewer 도 미표시)
```

## 3. Stage 별 결과

### 3.1 Stage 1+2 — 차이 1 진단 정정 + Fix

#### Fix (`691c6c3`)

`src/parser/hwp3/mod.rs` 의 `apply_bullet_fixup_single` 에 dash skip 조건 추가:

```rust
// 첫 non-space char 가 '-' (sub-item dash) 면 skip.
let first_non_space = para.text.chars().find(|c| *c != ' ').unwrap_or(' ');
if first_non_space == '-' { return; }
```

`apply_textbox_bullet_fixup` (nested) 의 동일 정책 적용.

#### 결과

| paragraph | 이전 | 이후 |
|-----------|------|------|
| 397 | ` ◦  공사 주요업무...` | 유지 (한컴 정합) |
| 398 | ` ◦    - 하드웨어...` | `    - 하드웨어...` (한컴 정합) |
| 399 | ` ◦    - ORACLE...` | `    - ORACLE...` (한컴 정합) |

상세: [`mydocs/working/task_m100_896_stage1.md`](../working/task_m100_896_stage1.md)

### 3.2 Stage 3+4 — 차이 2 WMF Fix

#### Fix 1 (`cfd6e5f`, font.rs) — charset 기반 primary 선택

`src/wmf/parser/objects/graphics/font.rs`:

```rust
if charset != crate::wmf::parser::CharacterSet::ANSI_CHARSET {
    (as_charset, as_latin1)
} else {
    (as_latin1, as_charset)
}
```

WMF spec: facename 은 Latin-1 ANSI only. 그러나 한컴 WMF (HANGUL_CHARSET) 는 CP949 그대로 → Latin-1 변환 시 `±¼¸²Ã¼` 깨진 글자. charset 기반 변환 결과를 primary 로 사용.

#### Fix 2 (`cfd6e5f`, util.rs) — 한국어 시스템 fallback chain

`src/wmf/converter/svg/util.rs`:

```rust
let has_korean = font_family.iter().any(|f| {
    f.chars().any(|c| ('\u{AC00}'..='\u{D7A3}').contains(&c))
});
if has_korean {
    for fallback in ["Apple SD Gothic Neo", "Malgun Gothic", "Nanum Gothic", "Noto Sans CJK KR", "sans-serif"] {
        font_family.push(fallback.to_string());
    }
}
```

facename 에 한글 char 있으면 시스템 한국어 폰트 chain 자동 추가. macOS/Windows/Linux 정합.

#### 결과

| 항목 | 이전 | 이후 |
|------|------|------|
| SVG font-family primary | `'±¼¸²Ã¼'` (깨진) | `'굴림체'` (정확) |
| 시스템 fallback chain | 없음 | Apple SD/Malgun/Nanum/Noto CJK + sans-serif |
| 시각 (rsvg-convert PNG) | 모자이크 | 정상 한글 |

상세: [`mydocs/working/task_m100_896_stage3.md`](../working/task_m100_896_stage3.md)

## 4. 검증

### 4.1 cargo test

```
cargo test --release --all-targets: 1398 passed, 0 failed
```

### 4.2 HWP3/HWPX sample 페이지 수 회귀

| 샘플 | 페이지 수 | 회귀 |
|------|---------|------|
| hwp3-sample.hwp ~ sample14/16 | 동일 | 없음 |
| 모든 HWPX 샘플 | 동일 | 없음 |

### 4.3 영향 범위

- 모든 HWP3 sample 의 paragraph 의 dash 시작 본문 (sub-item) 에서 ◦ 잘못 추가 차단
- 모든 한국어 WMF (HANGUL_CHARSET) 의 font name 인코딩 정정
- 다른 multi-byte charset (SHIFTJIS / GB2312 등) 자동 정합
- ANSI WMF (영문) 영향 없음

## 5. 변경 파일

### 5.1 소스

- `src/parser/hwp3/mod.rs` (+8) — Stage 1+2 dash skip
- `src/wmf/parser/objects/graphics/font.rs` (+11) — Stage 3+4 charset primary
- `src/wmf/converter/svg/util.rs` (+18, -2) — Stage 3+4 Korean fallback chain

### 5.2 문서

- `mydocs/plans/task_m100_896.md` — 수행 계획서
- `mydocs/plans/task_m100_896_impl.md` — 구현 계획서
- `mydocs/working/task_m100_896_stage1.md` — Stage 1+2 (차이 1)
- `mydocs/working/task_m100_896_stage3.md` — Stage 3+4 (차이 2)
- `mydocs/report/task_m100_896_report.md` — 본 최종 보고서

## 6. 결론

본 task #896 의 두 차이 모두 완전 해소:
- ✅ **차이 1**: paragraph 398/399 의 ◦ 잘못 추가 → 한컴 정합 fix
- ✅ **차이 2**: WMF 그림 안 한글 텍스트 깨짐 → font encoding + fallback fix

cargo test 1398 passed + sample 회귀 없음. 한컴 viewer (PDF) 정합.
