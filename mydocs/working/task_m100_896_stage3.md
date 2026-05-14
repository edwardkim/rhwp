# Task #896 Stage 3+4 통합 보고서 — 차이 2 진단 + Fix (WMF font encoding)

**Stage**: 3+4 / 5 (차이 2: WMF 그림 안 텍스트 겹침)
**상태**: ✅ 완료

## 1. 진단

### 1.1 WMF SVG 변환 결과 분석

sample16 paragraph 394 의 picture (WMF `bin_id=3`, "주전산센터 목표시스템 구성(안)") 의 SVG 변환 결과 (base64 decode):

```xml
<text dominant-baseline="auto" fill="#FFFFFF"
  font-family="'±¼¸²Ã¼','굴림체'"
  font-size="117" font-weight="700" id="elem28" text-anchor="start"
  x="174" y="255">
  <tspan>주</tspan><tspan dx="1">전</tspan>...
</text>
```

→ **font-family primary 가 `±¼¸²Ã¼`** (깨진 글자). secondary 가 `굴림체` (정확).

### 1.2 ROOT CAUSE 추적

`src/wmf/parser/objects/graphics/font.rs:165~188` 의 facename 처리:

```rust
let as_latin1 = bytes_into_utf8(&bytes[..len], ANSI_CHARSET)?;  // Latin-1 변환
let as_charset = bytes_into_utf8(&bytes[..len], charset)?;       // charset 변환
(as_latin1, as_charset)  // primary=as_latin1, secondary=as_charset
```

WMF spec: facename 은 Latin-1 ANSI character 만 허용. 그러나 **한컴 WMF (HANGUL_CHARSET=0x81) 는 CP949 byte 그대로 facename 에 넣음** — spec 위반이나 한컴의 일반적 패턴.

`as_latin1`: "굴림체" 의 CP949 bytes 를 Latin-1 으로 해석 → `±¼¸²Ã¼` (깨진 글자)
`as_charset` (CP949): "굴림체" (정확)

primary 로 `as_latin1` (깨진 글자) 사용 → SVG renderer 가 못 찾음 → fallback `굴림체` 시도 → 시스템 미설치 시 다시 fallback 없음 → 깨진 글리프 표시.

### 1.3 시각 검증 (rsvg-convert PNG)

PDF 비교용 PNG 추출:

- 한컴 PDF: 그림 안 텍스트 정상 한글 표시
- rhwp (이전): **모자이크 깨진 글리프** (font 못 찾음)

## 2. Fix

### 2.1 Fix 1 — font.rs: charset 기반 primary 선택

`src/wmf/parser/objects/graphics/font.rs`:

```rust
// charset 이 ANSI 가 아니면 as_charset (CP949 등) 을 primary, as_latin1 fallback
if charset != crate::wmf::parser::CharacterSet::ANSI_CHARSET {
    (as_charset, as_latin1)
} else {
    (as_latin1, as_charset)
}
```

### 2.2 Fix 2 — util.rs: 한국어 시스템 fallback chain 추가

`src/wmf/converter/svg/util.rs:414`:

```rust
let has_korean = font_family.iter().any(|f| {
    f.chars().any(|c| ('\u{AC00}'..='\u{D7A3}').contains(&c))
});
if has_korean {
    for fallback in [
        "Apple SD Gothic Neo",
        "Malgun Gothic",
        "Nanum Gothic",
        "Noto Sans CJK KR",
        "sans-serif",
    ] {
        font_family.push(fallback.to_string());
    }
}
```

facename 또는 fallback_facename 에 한글 char (U+AC00~U+D7A3) 있으면 한국어 시스템 폰트 chain 추가.

## 3. 결과

### 3.1 SVG font-family (fix 후)

이전:
```
font-family="'±¼¸²Ã¼','굴림체'"
```

이후:
```
font-family="'굴림체','±¼¸²Ã¼','Apple SD Gothic Neo','Malgun Gothic','Nanum Gothic','Noto Sans CJK KR','sans-serif'"
```

### 3.2 시각 정합

| 항목 | 이전 | 이후 |
|------|------|------|
| 그림 안 텍스트 표시 | 모자이크 (깨진 글리프) | **정상 한글 텍스트** ✅ |
| 시스템 환경 호환 | 굴림체 설치 필요 | **macOS/Windows/Linux 정합** |

### 3.3 회귀 점검

- `cargo test --release --lib`: **1250 passed**, 0 failed
- HWP3 sample 6종 + HWPX sample 페이지 수 회귀 없음
- ANSI WMF (영문 only) 의 경우 영향 없음 — Fix 1 의 `charset != ANSI_CHARSET` 조건으로 분기

## 4. 영향 범위

- 모든 한국어 WMF (HANGUL_CHARSET) 의 font-family 정합
- 다른 multi-byte charset (SHIFTJIS_CHARSET=0x80, GB2312_CHARSET=0x86 등) 도 자동 정합
- ANSI WMF (서구권) 영향 없음

## 5. 커밋

- `(다음 commit)` — Task #896 Stage 3+4: WMF font name 인코딩 + fallback chain

## 6. Stage 산출물

- 본 보고서: `mydocs/working/task_m100_896_stage3.md`
- Fix 1: `src/wmf/parser/objects/graphics/font.rs` (+11 lines)
- Fix 2: `src/wmf/converter/svg/util.rs` (+18 lines, -2 lines)
