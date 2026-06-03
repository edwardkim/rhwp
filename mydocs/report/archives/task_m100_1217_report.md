# 최종 보고서 — Task #1217: Dependabot 결합 의존성 그룹화

- **이슈**: #1217 (M100 / v1.0.0)
- **브랜치**: `local/task1217`
- **성격**: 의존성 인프라 개선 (설정 1파일, 소스 코드 무변경)
- **작성일**: 2026-06-01

## 1. 문제

PR #1215(pdf-writer 0.12→0.15)가 CI Build & Test / Analyze(rust) **FAILURE**.

`src/renderer/pdf.rs` 가 svg2pdf + pdf-writer 를 함께 사용(svg2pdf 가 생성한 `pdf_writer::Chunk`
를 재배치). 두 크레이트는 동일 pdf-writer 타입을 공유해야 하나 svg2pdf 0.13(최신)은 pdf-writer
**0.12** 에 고정 → pdf-writer 단독 0.15 bump 시 그래프에 0.12/0.15 공존 →
`error[E0308]: mismatched types — multiple different versions of crate pdf_writer` (로컬 재현 확증).

Dependabot 기본 동작이 결합 크레이트를 개별 PR 로 올려 동일 함정 반복(usvg/resvg/ttf-parser 동일).

## 2. 해결

`.github/dependabot.yml` 에 결합 의존성 그룹 추가:

| 생태계 | 그룹 | 패턴 |
|--------|------|------|
| cargo | `resvg-stack` | svg2pdf, pdf-writer, usvg, resvg, ttf-parser |
| npm | `vite-stack` | vite, vite-plugin-pwa |

- 결합 크레이트는 1 PR 로 동반 bump → svg2pdf 가 새 pdf-writer 지원 시에만 그룹 통과.
- security-updates 는 그룹화 안 함(개별 신속 처리). 그룹 외 crate(serde 등)는 개별 PR 유지
  → 작은 배치 회전 철학(`feedback_small_batch_release_strategy`) 정합.
- 기존 키(target-branch=devel, weekly, limit 10) 전부 보존.

## 3. 검증

| 항목 | 결과 |
|------|------|
| YAML 문법 (`yaml.safe_load`) | ✅ version 2 |
| 기존 키 보존 (cargo/npm) | ✅ target-branch/weekly/limit |
| 그룹 구조·이름 규칙 | ✅ resvg-stack(5) / vite-stack(2) |
| 변경 범위 | ✅ `.github/dependabot.yml` 1파일만 |
| 빌드·테스트 영향 | ✅ 무영향 (소스/Cargo.toml/package.json diff 없음, 자명) |

## 4. 단계 요약

- Stage 1: dependabot.yml groups 추가 (`task_m100_1217_stage1.md`)
- Stage 2: YAML/구조/스펙 검증 (`task_m100_1217_stage2.md`)
- Stage 3: 최종 보고서 + orders + 커밋 (본 문서)

## 5. 사후 관찰 항목

그룹 적용의 실효(단독 #1215 류 재발 방지)는 **다음 weekly Dependabot 실행** 시
resvg-stack/vite-stack 이 단일 그룹 PR 로 묶이는지로 확인. 이번 타스크 검증 범위 밖 —
머지 후 GitHub 동작 관찰 사항.

## 6. 범위 밖 (별개 처리)

- #1216(vite)/#1214(puppeteer-core) 머지, #1215 close — 별개 Dependabot PR 처리.
- pdf-writer 직접 의존 제거(svg2pdf re-export 전환) — 코드 변경, 별도 이슈 후보.
