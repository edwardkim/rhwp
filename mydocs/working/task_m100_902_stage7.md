# Task #902 Stage 7 보고서 — 폰트 metric 정합

**Stage**: 7 / 9 (v2)
**상태**: 완료 (7-B: font-family 체인 조정)

## 1. 변경 영역

`src/wmf/converter/svg/util.rs` — `Font::set_props` 의 font-family 체인 구성

## 2. ROOT CAUSE

WMF 의 facename (예: "굴림체") 가 시스템 미설치 → fontconfig 가 Verdana 등 비-한국어 폰트로 fallback → 깨진 글자.

기존 fallback_facename 은 한컴 WMF 의 CP949 byte 를 Latin-1 으로 잘못 디코드한 garbled string ("±¼¸²Ã¼" 등) — font matching 무의미.

## 3. 변경 내용

### 3.1 Garbled fallback_facename 필터

```rust
fn is_garbled_latin1_korean(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| ('\u{0080}'..='\u{00FF}').contains(&c))
}
```

extended Latin (U+0080~U+00FF) 만 으로 구성된 string 검출 → font-family chain 제외.

### 3.2 Korean 폰트 체인 정합 (sans-serif vs serif 분기)

```rust
let is_serif = facename.contains("바탕") || facename.contains("궁서")
    || facename.contains("Batang") || facename.contains("Gungsuh");
let fallback_chain: &[&str] = if is_serif {
    &[
        "Batang", "바탕", "Nanum Myeongjo", "AppleMyungjo",
        "Noto Serif KR", "Noto Serif CJK KR", "serif",
    ]
} else {
    &[
        "Apple SD Gothic Neo", "Malgun Gothic", "맑은 고딕",
        "Nanum Gothic", "Noto Sans KR", "Noto Sans CJK KR",
        "sans-serif",
    ]
};
```

- 한글 facename 검출 시 적절한 한국어 fallback chain 자동 추가
- sans/serif 구분
- 한글 폰트명 (`맑은 고딕`) + 영문명 (`Malgun Gothic`) 모두 포함 — 시스템 별 검색 호환성

## 4. font-family 출력 비교

**Stage 6 (Before)**:
```
'굴림체','±¼¸²Ã¼','Apple SD Gothic Neo','Malgun Gothic','Nanum Gothic','Noto Sans CJK KR','sans-serif'
```

**Stage 7 (After)**:
```
'굴림체','Apple SD Gothic Neo','Malgun Gothic','맑은 고딕','Nanum Gothic','Noto Sans KR','Noto Sans CJK KR','sans-serif'
```

- Garbled '±¼¸²Ã¼' 제거
- '맑은 고딕' 한글 명칭 추가 (fontconfig 검색 호환성)
- 'Noto Sans KR' 최신 명칭 추가

## 5. 검증 결과

### 5.1 빌드 + 회귀

```
cargo build --release           — Finished
cargo test --release --all-targets — 1412 passed / 0 failed
```

### 5.2 SVG/PNG 회귀

- sample16 page 18 PNG (rsvg-convert): **byte-identical** to Stage 6 (macOS 에서는 Apple SD Gothic Neo 동일 선택)
- SVG 파일 크기: 4412621 → 4431825 (한글 명칭 추가로 미세 증가)
- 다른 OS / fontconfig 환경에서는 더 robust 한 한국어 fallback

## 6. 선택지 평가 (v2 plan 의 7-A/B/C/D)

| 옵션 | 본 stage 적용 |
|------|--------------|
| 7-A: CSS 체인 우선순위 조정 | ✓ (부분) — Apple SD Gothic Neo 우선 유지 |
| **7-B: 오픈 한국어 폰트 substitute** | ✓ **(채택)** — Nanum Gothic, Noto Sans KR 추가 |
| 7-C: 폰트 임베딩 | 미적용 — 파일 크기 폭증 vs 효과 trade-off, 향후 follow-up |
| 7-D: per-tspan textLength | 미적용 — 미세 왜곡 위험 |

## 7. 산출물

- 소스 수정: `src/wmf/converter/svg/util.rs`
- 본 보고서: `mydocs/working/task_m100_902_stage7.md`

## 8. 다음 단계

Stage 8: 광범위 회귀 검증 (다중 sample + golden SVG)
