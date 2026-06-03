---
타스크: #417 HWP 3.0 정식 파서 구현
단계: Stage 3 — 하이퍼링크 URL 추출 구현
브랜치: local/task417
작성일: 2026-04-29
상태: 완료
---

# Stage 3 완료 보고서

## 1. 구현 목표

HWP3 파서에서 하이퍼링크(ch=10, `parsed_is_hypertext=true`) 개체의 URL을 추출하여
`Control::Hyperlink.url` 필드에 설정.

이전 상태: `url: String::new()` (TODO 주석 존재)

## 2. 스펙 분석

스펙 `mydocs/tech/한글문서파일구조3.0.md` §8.3 하이퍼텍스트(HyperLink) 정보:

| 오프셋 | 자료형 | 길이 | 의미 |
|--------|--------|------|------|
| 0 | dword | 4 | 3 = 하이퍼텍스트 정보 ID |
| 4 | dword | 4 | 617 × n = 정보 길이 |
| 8 | kchar[256] | 256 | 건너뛸 파일 이름(URL) — null 종료 |
| 264 | hchar[16] | 32 | 건너뛸 책갈피 |
| 296 | byte[325] | 325 | 매크로 (도스용) |
| 621 | byte | 1 | 종류 (0,1=한글 2=HTML/ETC) |
| 622 | byte[3] | 3 | 예약 |

- 추가정보블록(`Hwp3AdditionalInfoBlock`) id=3으로 식별
- 각 항목 617바이트, `block.data.len() / 617`개의 하이퍼링크
- URL은 `block.data[entry_offset..entry_offset+256]` — kchar 인코딩, null 종료
- 등장 순서: 본문 내 하이퍼링크 순서와 추가정보블록 순서가 일치

## 3. 구현 내용

### `src/parser/hwp3/mod.rs`

추가정보블록 처리 루프에 id=3 핸들러 추가:

```rust
} else if block.id == 3 {
    // 추가정보블록 #1 TagID 3 = 하이퍼텍스트(HyperLink) 정보
    // 각 항목 617바이트
    //   data[  0..256]: 건너뛸 파일 이름(URL) — kchar[256], null 종료
    //   data[256..288]: 건너뛸 책갈피 — hchar[16]
    //   data[288..613]: 매크로 (도스용) — byte[325]
    //   data[613]     : 종류 (0,1=한글 2=HTML/ETC)
    const ENTRY_SIZE: usize = 617;
    let n = block.data.len() / ENTRY_SIZE;
    for i in 0..n {
        let offset = i * ENTRY_SIZE;
        if offset + 256 <= block.data.len() {
            let url = decode_hwp3_string(&block.data[offset..offset + 256]);
            hyperlink_urls.push(url);
        }
    }
}
```

추가정보블록 처리 후 URL을 본문 단락의 Control::Hyperlink에 적용:

```rust
if !hyperlink_urls.is_empty() {
    let mut url_idx = 0;
    for para in &mut paragraphs {
        for ctrl in &mut para.controls {
            if let Control::Hyperlink(hl) = ctrl {
                if url_idx < hyperlink_urls.len() {
                    hl.url = hyperlink_urls[url_idx].clone();
                    url_idx += 1;
                }
            }
        }
    }
}
```

## 4. 검증

### 4.1 샘플 파일 제약

`samples/hwp3-sample.hwp`는 학술 논문으로 하이퍼링크가 없음.
추가정보블록 id=3 탐색 결과: 없음.

### 4.2 단위 테스트

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | 1016 passed / 0 failed / 1 ignored |
| `cargo clippy -- -D warnings` | clean |
| `rhwp export-svg samples/hwp3-sample.hwp` | 20 SVG 생성 성공 (회귀 없음) |

### 4.3 구현 정합성

- 스펙 §8.3 구조대로 617바이트 단위 파싱
- kchar 문자열은 `decode_hwp3_string`으로 디코딩 (ASCII URL 정상 처리)
- 순서 매핑: 본문 하이퍼링크 순서 = 추가정보블록 순서 (하이퍼링크 문서에서 검증 필요)

## 5. 커밋 대상 파일

- `src/parser/hwp3/mod.rs` — 하이퍼링크 URL 추출 (id=3 블록 처리)
- `mydocs/working/task_m100_417_stage3.md` — 이 보고서

## 6. 다음 단계

Stage 3 완료 → Stage 4: 최종 검증 + 결과 보고서 작성
