---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_3891.md
last_verified: 2026-08-03
---

# Task #3891 Stage 3 보고 — 전체 검증

| 게이트 | 결과 |
|---|---|
| `cargo test --profile release-test --tests` 전체 | **exit 0** |
| `cargo clippy --workspace --all-targets -- -D warnings` | 통과 |
| `cargo fmt --check` | 통과 |
| 신설 가드 2건 | ok |

외부 툴체인 없이 `cargo test` 만으로 실행된다(검증 기준 4). Swift·.NET 설치 불요.

## 부수 확인 — 공개 API 대칭 회복

정정 전후로 두 래퍼의 공개 표면을 비교했다.

| 래퍼 | 정정 전 | 정정 후 |
|---|---|---|
| C# | 2개 (ExportText·ExportMarkdown) | **3개** (+ ReadText) |
| Swift | 3개 (exportText·exportMarkdown·readText) | 3개 |

가드가 잡은 것은 단순한 선언 누락이 아니라 **기능 격차**였다 — C# 사용자는 파일로
내보내지 않고 텍스트만 읽는 경로를 쓸 수 없었다.
