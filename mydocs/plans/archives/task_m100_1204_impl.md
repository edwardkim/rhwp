# 구현계획서 — Task M100 #1204

**이슈**: [#1204](https://github.com/edwardkim/rhwp/issues/1204) HWPX 수식 스크립트 토큰 처리
**브랜치**: `local/task1204`
**작성일**: 2026-06-01

---

## 단계 분할 (4단계)

### Stage 1 — 토크나이저: root/sqrt(A), prime(C) glued 분리
`src/renderer/equation/tokenizer.rs` `read_command`:
- `root`/`sqrt`(+대문자): 뒤가 **숫자**면 키워드만 소비 후 분리 (over/atop digit-guard 패턴).
- `prime`(+대문자): 뒤가 **alnum**이면 분리 (bold/it/rm 패턴).

산출물: 빌드 + 토크나이저 단위 테스트.

### Stage 2 — 파서: 글꼴명령 body 의 decoration/구조 명령 정상 처리 (B)
`src/renderer/equation/parser.rs` `parse_single_or_group`:
- Command 분기에서 symbol/function 외에는 `self.parse_command(&val)` 재귀로 변경.

산출물: 빌드 + 파서 단위 테스트.

### Stage 3 — 회귀 테스트
- 토크나이저: `root3`→[root,3], `sqrt5`→[sqrt,5], `primeF`→[prime,F].
- 파서/AST: `rm bar {F prime F}` → FontStyle(Roman, Decoration(Bar, ...)), `root3` → Sqrt(3).
- 렌더: 위 스크립트 SVG 에 "root"/"bar"/"prime" literal leak 없음.

산출물: `cargo test` 통과 (기존 수식 테스트 회귀 0 포함).

### Stage 4 — 시각 검증
1. `export-svg -p 19` → 문24)·25)·30) √·overline 렌더 확인 (SVG 텍스트 grep).
2. rsvg 래스터 → 한글 2022 PDF 20쪽 시각 대조.
3. `rustfmt`(변경 파일) → 최종 보고서.

---

## 영향 범위

- 수정 파일: `src/renderer/equation/tokenizer.rs`, `parser.rs` (+테스트).
- 모델/레이아웃/타 모듈 무변경.
