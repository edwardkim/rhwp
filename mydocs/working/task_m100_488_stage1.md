# Task M100 #488 — Stage 1 완료 보고서

## 작업 내용

`src/renderer/equation/tokenizer.rs` 수정: hwpeq 폰트 스타일 키워드(`bold`/`it`/`rm`)가 식별자 prefix로 등장하는 경우 토크나이저가 키워드 길이만큼만 소비하고 나머지를 별개 토큰으로 분리하도록 변경.

### 구현 핵심

- `matches_at(kw)` 헬퍼 추가: 현재 위치에 키워드가 prefix로 있는지 검사
- `read_command()` 진입 시 `bold` → `it` → `rm` 순으로 prefix 매치 시도
- 키워드 직후가 ASCII 영문자/숫자(즉 식별자가 이어짐)인 경우에만 분리; 직후가 공백·기호·EOF면 기존 동작 유지

### 파일 변경

| 파일 | 변경 |
|------|------|
| `src/renderer/equation/tokenizer.rs` | +86 −0 (헬퍼 1개, `read_command` 로직 보강, 단위 테스트 8건) |

## 검증 결과

### 신규 단위 테스트 8건

```
test_font_style_prefix_rm_uppercase ........ ok   (rmK ^{+})
test_font_style_prefix_rm_compound ......... ok   (rmCa ^{2+})
test_font_style_prefix_rm_lowercase ........ ok   (1`rmmol)
test_font_style_prefix_it_compound ......... ok   (LEFT ( itaq RIGHT ))
test_font_style_prefix_it_single_letter .... ok   (LEFT ( itl RIGHT ))
test_font_style_prefix_bold ................ ok   (boldX)
test_font_style_keyword_alone_unchanged .... ok   (rm K / it{x} / rm)
test_existing_commands_unchanged ........... ok   (OVER MATRIX SQRT alpha beta)
```

### 회귀 검증

- 토크나이저 모듈: 20 passed (기존 12 + 신규 8)
- 수식 모듈 전체: 60 passed
- 라이브러리 전체: **1086 passed, 0 failed, 1 ignored**

## Stage 2 진행 가능 여부

✅ Stage 1 완료. Stage 2 (exam_science.hwp 시각 검증) 진행 가능.

## 승인 요청

위 결과로 Stage 2 진행을 승인 요청드립니다.
