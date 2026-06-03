# 최종 결과 보고서: Task M100 #127

**이슈**: [#127 Dependabot 보류 PR 코드 수정 대응](https://github.com/edwardkim/rhwp/issues/127)  
**브랜치**: `local/task127`  
**완료일**: 2026-04-13

---

## 처리 결과 요약

| PR | 내용 | 결과 |
|----|------|------|
| #124 (quick-xml 0.37 → 0.39) | `unescape()` → `decode()` 교체 | ✅ merge 대상 |
| #119 (typescript 5.x → 6.0) | `baseUrl` 제거, `paths` 재구성 | ✅ merge 대상 |
| #116 (usvg 0.45 → 0.47) | svg2pdf upstream 미지원 | ✅ close 완료 |
| #123 (pdf-writer 0.12 → 0.14) | svg2pdf 0.13 호환성 문제 | ✅ close 완료 |

---

## Stage 1: quick-xml 0.39 대응

**변경 파일**: `Cargo.toml`, `src/parser/hwpx/section.rs`

- `Cargo.toml`: `quick-xml = "0.37"` → `"0.39"`
- `section.rs` 6곳: `BytesText::unescape()` → `decode()` 교체
  - 줄 505, 2380, 2512, 2594: `t.decode().unwrap_or_default()`
  - 줄 2662, 2756: `if let Ok(s) = *.decode()`
- `cargo test`: **785 passed, 0 failed**

**특이 사항**: quick-xml 0.37의 `BytesText`에는 `decode()` 없음. 버전 업그레이드가 선행 필요.

---

## Stage 2: TypeScript 6.0 대응

**변경 파일**: `rhwp-studio/package.json`, `rhwp-studio/tsconfig.json`

- `package.json`: `typescript: "^5.7.0"` → `"^6.0.2"`
- `tsconfig.json` 변경:
  - `baseUrl: "."` 제거 (TS6.0 deprecated error)
  - `paths["@/*"]`: `"src/*"` → `"./src/*"` (baseUrl 없이 상대 경로 명시)
  - `types: ["chrome"]` 추가 (TS6.0에서 @types 자동 포함 동작 변경)
- `tsc --noEmit`: **에러 없음**

**특이 사항**:
- TS6.0에서 `baseUrl` 제거 시 `@types/chrome`이 자동 포함되지 않는 문제 발생 → `types` 명시로 해결
- `paths`의 비상대 경로는 `./` 접두어로 `baseUrl` 의존 해소

---

## Stage 3: PR #116/#123 Close

- **PR #116** (usvg 0.47): svg2pdf 0.13이 usvg 0.45 고정 → close
- **PR #123** (pdf-writer 0.14): svg2pdf 0.13이 pdf-writer 0.12 고정 → close
- 두 PR 모두 svg2pdf upstream 신버전 출시 시 재처리 예정 (Issue #127 코멘트로 추적)
