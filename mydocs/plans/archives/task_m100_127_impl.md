# 구현 계획서: Task M100 #127

**이슈**: [#127 Dependabot 보류 PR 코드 수정 대응](https://github.com/edwardkim/rhwp/issues/127)  
**수행 계획서**: `mydocs/plans/task_m100_127.md`  
**작성일**: 2026-04-13

---

## 단계 구성

| 단계 | 내용 | 대상 파일 |
|------|------|-----------|
| Stage 1 | quick-xml 0.39 대응: `unescape()` → `decode()` 교체 | `src/parser/hwpx/section.rs` |
| Stage 2 | TypeScript 6.0 대응: `baseUrl` 제거 | `rhwp-studio/tsconfig.json` |
| Stage 3 | PR #116/#123 close 및 보류 이슈 등록 | GitHub |

---

## Stage 1: quick-xml 0.39 — `unescape()` → `decode()`

### 변경 목표

`quick-xml` 0.39에서 `BytesText::unescape()` API 제거.  
`BytesText::decode()` 로 대체한다.  
반환 타입이 `Cow<str>` 이므로 기존 `&str` 패턴과 호환.

### 변경 위치 (6곳)

| 줄 | 기존 코드 | 변경 후 |
|----|-----------|---------|
| 505 | `t.unescape().unwrap_or_default()` | `t.decode().unwrap_or_default()` |
| 2380 | `t.unescape().unwrap_or_default().to_string()` | `t.decode().unwrap_or_default().to_string()` |
| 2512 | `t.unescape().unwrap_or_default()` | `t.decode().unwrap_or_default()` |
| 2594 | `t.unescape().unwrap_or_default()` | `t.decode().unwrap_or_default()` |
| 2662 | `if let Ok(s) = txt.unescape()` | `if let Ok(s) = txt.decode()` |
| 2756 | `if let Ok(s) = t.unescape()` | `if let Ok(s) = t.decode()` |

> **참고**: `decode()`는 `Result<Cow<str>>` 반환 (vs `unescape()`도 동일). 패턴 변경 없이 메서드명만 교체.

### 완료 기준

```bash
cargo test   # 전체 테스트 통과
```

---

## Stage 2: TypeScript 6.0 — `baseUrl` 제거

### 변경 목표

TypeScript 6.0에서 `compilerOptions.baseUrl`이 deprecated error로 처리됨.  
`moduleResolution: "bundler"` 환경에서는 `paths`만으로 경로 alias 동작 가능.

### 변경 내용

`rhwp-studio/tsconfig.json`에서 `"baseUrl": "."` 라인 제거.  
`paths`의 상대 경로 기준은 `tsconfig.json` 위치가 되므로 동작 변경 없음.

### 완료 기준

```bash
cd rhwp-studio && npx tsc --noEmit   # 빌드 에러 없음
```

---

## Stage 3: PR #116/#123 Close

### 처리 내용

1. GitHub에서 PR #116 (usvg 0.47) close — 코멘트: svg2pdf upstream 대기
2. GitHub에서 PR #123 (pdf-writer 0.14) close — 코멘트: svg2pdf 0.13 호환성 문제

### 후속 이슈 등록 (선택)

- `svg2pdf`가 usvg 0.47 지원 버전 출시 시 재처리 예정 (Issue #127 코멘트로 추적)
