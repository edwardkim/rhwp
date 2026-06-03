# Task #1217 Stage 1 — dependabot.yml 그룹 추가

- **이슈**: #1217 / **브랜치**: `local/task1217`
- **작성일**: 2026-06-01

## 변경

`.github/dependabot.yml` 두 ecosystem 블록에 `groups` 키 추가. 기존 키 전부 보존.

### cargo — resvg-stack
```yaml
    groups:
      resvg-stack:
        patterns:
          - "svg2pdf"
          - "pdf-writer"
          - "usvg"
          - "resvg"
          - "ttf-parser"
```
svg2pdf 가 만든 `pdf_writer::Chunk` 를 `src/renderer/pdf.rs` 가 재배치 → 동일 버전 그래프 공유 필수.

### npm — vite-stack
```yaml
    groups:
      vite-stack:
        patterns:
          - "vite"
          - "vite-plugin-pwa"
```
vite-plugin-pwa 1.x ↔ vite 8 peer 결합.

## 설계 결정

- `applies-to` 미지정 → version-updates 에만 적용. security-updates 는 그룹화 안 함(개별 신속 처리).
- `update-types` 미지정 → major/minor/patch 모두 그룹. 결합 크레이트는 patch 도 동반 안전.
- 그룹 외 crate(serde 등)는 자동으로 개별 PR 유지 — 작은 배치 회전 철학 정합.

## 결과

소스/Cargo.toml/package.json 무변경. 설정 1파일만 수정.
