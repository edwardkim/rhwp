---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 59 — 최종 devel 재리베이스 PR 게이트

## 목적

Stage 58 검사 중 전진한 `upstream/devel`을 다시 반영한 정확한 HEAD에서 모든
회귀 게이트를 순차 재실행하고, 증적과 PR 초안까지 준비한다. 원격 push와 PR
생성은 사용자의 별도 승인 전에는 수행하지 않는다.

## 재리베이스 기준

- Stage 58 검사 HEAD: `d5fd69d72c621df291d0da22dca48b75e82a59fd`
- 새 upstream 기준: `e64c853124a74109e44d5e42499a2825d05c85a2`
- 재리베이스 직후 HEAD: `2558786d746cdc115f6c8a9817949b327e37bce2`
- 결과: 80커밋 재적용, 충돌 없음, ahead 80 / behind 0 / clean

신규 upstream 9커밋은 레시피와 PR review archive 문서 변경이며 renderer 소스와
충돌하지 않았다. 그래도 리베이스 전 통과 결과를 승계하지 않고 새 checkout에서
전부 다시 검증한다.

## 순차 검증 계획

모든 Cargo 명령은 `CARGO_INCREMENTAL=0`과
`CARGO_TARGET_DIR=target/task-3820-stage59-final-pr-gate`를 사용하며, 같은 checkout과
target에서 병렬 실행하지 않는다.

1. release-test 빌드와 issue2007·issue3637·boxed-PUA focused 회귀
2. issue2007 p11-p13 144dpi PDF 직접 대조
3. release 빌드와 lib 회귀
4. `cargo test --profile release-test --tests` 전체 integration
5. Native Skia 공식 회귀 3종
6. fmt, diff-check, Clippy, rustdoc
7. TypeScript, Studio test, WASM build
8. #536·#4158·#4224 브라우저 E2E
9. Markdown 링크, 대용량·LFS, branch·merge-base 상태 확인

## 완료 조건

- focused·전체 Rust·overflow-cell 회귀 실패 0
- issue2007 17쪽과 p11-p13 exact title owner·page cut 유지
- Native Skia, fmt, diff-check, Clippy, rustdoc 통과
- TypeScript, Studio, WASM, 브라우저 E2E 통과
- 재현 가능한 시각 증적과 최종 SHA를 보관하고 PR 제목·본문 초안을 준비

## 게이트 중단 판정

새 HEAD `4f1c692d2`에서 release-test build, boxed-PUA, issue2007 15건,
issue3637 3건 focused 회귀와 `cargo build --release`는 통과했다. 그러나
작업지시자의 PDF 직접 감사에서 issue2007 물리 p14 하단 문장 소실이 추가로
확인됐다. 따라서 이 단계는 PR 준비 완료로 판정하지 않고, 잘못된 출력 상태에서
전체 회귀를 계속 소비하지 않았다.

- 기준 PDF p14: 금융위원회 항목 ⑦의 `관계자에게 내보여야` / `한다.`를 모두 표시
- rhwp p14: 동일 source owner는 render tree에 존재하나 조상 셀 clip이 두 줄을 일부·전부 잘라냄
- 후속: [Stage 60](task_m100_3820_stage60_issue2007_p14_ancestor_clip.md)에서 원인·회귀·코드를 별도 처리
