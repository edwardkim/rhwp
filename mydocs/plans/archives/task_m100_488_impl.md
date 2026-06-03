# Task M100 #488 — 구현계획서

## 1. 변경 대상 파일

**1개 파일만 수정**: `src/renderer/equation/tokenizer.rs`

## 2. 핵심 설계

### 2.1 룰 정의 (휴리스틱 아님)

hwpeq 문법: `rm` / `it` / `bold` 폰트 스타일 키워드는 식별자의 prefix로 등장할 수 있으며, 키워드 길이만큼만 소비되고 나머지는 별도 토큰으로 처리된다.

분리 조건:
1. `read_command()` 진입 시 현재 위치에서 FONT_STYLES 키워드가 prefix로 매치되는지 확인
2. 매치된 경우 + 키워드 직후 문자가 ASCII 영문자/숫자라면 → 키워드 길이만큼만 소비하고 토큰 종료
3. 매치되었으나 직후가 식별자 종료(공백, 기호, EOF)라면 → 일반 키워드로 끝까지 소비 (기존 동작과 동일)

매치 우선순위: `bold` (4자) → `it` (2자) → `rm` (2자). `it`/`rm`는 첫 글자(`i` vs `r`)가 달라 충돌 없음.

### 2.2 회귀 안전성 검증

`grep -oE '"(rm|it|bold)[a-zA-Z]*"' src/renderer/equation/symbols.rs` 결과: `bold`, `it`, `rm` 자체 외에 prefix를 공유하는 명령어 없음. 즉 키워드 분리로 깨지는 기존 명령 매핑은 없다.

`read_command()` 의 호출자(파서)는 Command 토큰의 value 문자열을 키워드 테이블과 매칭하므로, prefix 분리 후에도 `rm` / `it` / `bold` 토큰이 정상적으로 FONT_STYLES와 매칭된다.

### 2.3 의사코드

```rust
fn read_command(&mut self) -> Token {
    let start = self.pos;

    // 폰트 스타일 키워드 prefix 매치 (bold > it > rm 순)
    for kw in ["bold", "it", "rm"] {
        if self.matches_prefix(kw) {
            // 키워드 직후 문자가 식별자 연속 여부 확인
            let next = self.peek(kw.len());
            if next.map_or(false, |c| c.is_ascii_alphanumeric()) {
                self.pos += kw.len();
                return Token::new(TokenType::Command, kw, start);
            }
        }
    }

    // 일반 식별자 읽기 (기존 로직)
    let mut value = String::new();
    while let Some(ch) = self.current() {
        if ch.is_ascii_alphanumeric() {
            value.push(ch);
            self.pos += 1;
        } else { break; }
    }
    Token::new(TokenType::Command, value, start)
}
```

`matches_prefix` 는 `self.chars[self.pos..]` 가 키워드로 시작하는지 검사하는 보조 함수(인라인 처리도 가능).

## 3. 단위 테스트 (Stage 1)

`tests` 모듈에 추가:

```rust
#[test]
fn test_font_style_prefix_rm_uppercase() {
    let tokens = tokenize("rmK ^{+}");
    assert_eq!(values(&tokens), vec!["rm", "K", "^", "{", "+", "}"]);
}

#[test]
fn test_font_style_prefix_rm_compound() {
    let tokens = tokenize("rmCa ^{2+}");
    assert_eq!(values(&tokens), vec!["rm", "Ca", "^", "{", "2", "+", "}"]);
}

#[test]
fn test_font_style_prefix_rm_lowercase() {
    let tokens = tokenize("1`rmmol");
    assert_eq!(values(&tokens), vec!["1", "`", "rm", "mol"]);
}

#[test]
fn test_font_style_prefix_it() {
    let tokens = tokenize("LEFT ( itaq RIGHT )");
    assert_eq!(values(&tokens), vec!["LEFT", "(", "it", "aq", "RIGHT", ")"]);
}

#[test]
fn test_font_style_prefix_it_single_letter() {
    let tokens = tokenize("LEFT ( itl RIGHT )");
    assert_eq!(values(&tokens), vec!["LEFT", "(", "it", "l", "RIGHT", ")"]);
}

#[test]
fn test_font_style_prefix_bold() {
    let tokens = tokenize("boldX");
    assert_eq!(values(&tokens), vec!["bold", "X"]);
}

#[test]
fn test_font_style_keyword_alone_unchanged() {
    // 키워드 직후가 공백/기호: 분리하지 않고 그대로 키워드
    let tokens = tokenize("rm K");
    assert_eq!(values(&tokens), vec!["rm", "K"]);
    let tokens = tokenize("it{x}");
    assert_eq!(values(&tokens), vec!["it", "{", "x", "}"]);
}

#[test]
fn test_existing_commands_unchanged() {
    // 기존 명령은 회귀 없음
    let tokens = tokenize("OVER MATRIX SQRT alpha beta");
    assert_eq!(values(&tokens), vec!["OVER", "MATRIX", "SQRT", "alpha", "beta"]);
}
```

## 4. 단계 구성

### Stage 1 — 토크나이저 수정 + 단위 테스트

**작업**:
1. `src/renderer/equation/tokenizer.rs:80 read_command()` 수정 (FONT_STYLES prefix 분리)
2. 위 단위 테스트 8건 추가
3. `cargo test --lib renderer::equation` 통과
4. `cargo build --release` 빌드 확인 (워크스페이스 전체 컴파일)

**커밋**: `Task #488: 수식 토크나이저 폰트 스타일 키워드 prefix 분리` + 단계별 보고서 `mydocs/working/task_m100_488_stage1.md`

**완료 기준**:
- 단위 테스트 모두 통과
- 기존 토크나이저 테스트 회귀 없음 (`cargo test --lib renderer::equation::tokenizer`)
- 단계별 보고서 작성 → 승인 요청

### Stage 2 — exam_science.hwp 시각 검증

**작업**:
1. `./target/release/rhwp export-svg samples/exam_science.hwp -o output/svg/task488_after/`
2. 페이지 1 SVG에서 `rmK`/`rmCa`/`rmmol`/`itl`/`itaq` 등 raw prefix 잔존 검사
   - `grep -oE '>(rm|it|bold)[A-Za-z]+<' output/svg/task488_after/exam_science_001.svg` → 0건이어야 함
3. 정답 PDF (`samples/pdf/hwp2022/exam_science.pdf`) 페이지 1과 비교 (수식 외관)
4. 페이지 2~4도 spot 확인 (전 페이지에 적용되는 토크나이저 변경이므로)

**커밋**: 단계별 보고서 `mydocs/working/task_m100_488_stage2.md`

**완료 기준**:
- 페이지 1 SVG에 raw prefix 0건
- 정답 PDF와 화학 수식이 시각적으로 일치 (K⁺, X⁻, Ca²⁺, O²⁻, mol, KOH(aq), H₂O(l) 등)
- 단계별 보고서 → 승인 요청

### Stage 3 — 회귀 검증 + 최종 보고

**작업**:
1. `samples/` 직속 HWP/HWPX 파일 목록 확인
2. 수정 전(devel HEAD) vs 수정 후(local/task488 HEAD)의 SVG 차이를 핵심 샘플 3~5개에 대해 비교
   - 비교 방식: 동일 파일을 두 브랜치에서 `export-svg` 후 `diff` (또는 시각적 spot check)
   - 차이가 모두 `rm`/`it`/`bold` prefix 정정으로 설명되는지 확인
3. 차이 없는 샘플(수식 자체가 없거나, prefix 형태가 없는 경우)은 회귀 없음으로 기록
4. 최종 보고서 `mydocs/report/task_m100_488_report.md` 작성
5. `mydocs/orders/{오늘날짜}.md` 갱신 (해당 오늘할일 파일이 있으면 #488 항목 추가/업데이트)

**커밋**: 최종 보고서 + orders 갱신

**완료 기준**:
- 회귀 검증 결과 기록
- 최종 보고서 → 승인 요청
- 승인 후 `local/task488` → `local/devel` merge

## 5. 예상 작업 시간

| 단계 | 작업 | 추정 |
|------|------|------|
| Stage 1 | 코드 수정 + 테스트 + 빌드 | 15분 |
| Stage 2 | SVG 검증 + PDF 비교 | 10분 |
| Stage 3 | 회귀 검증 + 보고서 | 20분 |
| 합계 |  | 45분 |

## 6. 롤백 계획

토크나이저 수정 1개 파일이며, FONT_STYLES prefix 분리 로직만 추가됨. 회귀 발견 시:
- `git revert` 단일 커밋으로 즉시 복구 가능
- 또는 prefix 매치 로직만 비활성화하는 추가 commit
