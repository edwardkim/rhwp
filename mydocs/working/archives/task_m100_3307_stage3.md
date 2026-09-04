---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_3307.md
last_verified: 2026-08-01
---

# Task #3307 Stage 3 보고 — 전체 게이트 검증

## 게이트 결과 (전부 통과)

| 게이트 | 결과 |
|---|---|
| `cargo test --profile release-test --tests` 전체 | **exit 0** |
| `cargo clippy --all-targets -- -D warnings` | **exit 0** |
| Native Skia 3종 (58+2+4) | 통과 |
| wasm Docker 재빌드 | 성공 (`pkg` 갱신) |
| samples 쪽수 A/B (수정 전/후 바이너리, 666건) | **차이 0건** — 번호 fallback 이 조판 흐름 무변 |
| **이중 baseline (4.3.1 신판 첫 적용)** — IR sweep | 신규 발산 **0** (fixture 왕복 청정) |
| **이중 baseline** — overflow-cell 원장 | 신규 발생 **0** (fixture 쪽 밖 소실 없음 — 행 미추가, 규약대로) |
| fmt | 통과 (Stage 2) |

## 정답지 이미지 스왑 (p7)

- **한컴 2020 vs rhwp 수정 후**: 1.~6. 전 항목 번호 정합, 표 구조·배치 일치.
- **수정 전 vs 후**: 1.~4. 자동번호 복원이 유일 차이.
- 자산: `mydocs/report/assets/task3307_p7_{hancom_vs_rhwp,before_vs_after}.png`
  — [이슈 코멘트](https://github.com/edwardkim/rhwp/issues/3307#issuecomment-5148855317)에
  embed 게시(이미지 호스팅 위해 `task3307` 브랜치 선push — PR 생성과 별개).
- 판정 세트: `/mnt/e/hwp/swap3307/` (한컴 대조쌍·수정 전후쌍·개별 3장).

## 작업 사고 2건 기록 (재발 방지)

1. **상주 stash 함정** — clean 트리에서 `git stash`/`pop` 실행 → stash 가 아무것도
   저장하지 않았고 pop 이 상주 stash 를 꺼내 충돌. `git reset --hard HEAD` 복구,
   상주 4개 보존 확인. baseline 바이너리는 임시 라인 제거 방식으로 대체.
2. **red-check 원복 실수** — 미커밋 수정 상태에서 `git checkout` 원복 → 수정 자체
   소실, 재적용. red-check 는 수정 커밋 후 수행하거나 파일 백업으로 원복할 것.

## 부수 관찰

- #3308 예비 재현(같은 문서 p7): 증상 2(용지규격 문구)는 현행 텍스트 층 미재현 —
  #3127(v0.8.0) 수정 가능성. 증상 1(직인 위치)은 시각 관찰됨. 별도 착수 결정
  (작업지시자, 2026-08-01).

## 남은 단계

Stage 4 — 작업지시자 시각 판정 → 최종 보고 → PR(별도 승인) → merge → 리포터
완결 회신.
