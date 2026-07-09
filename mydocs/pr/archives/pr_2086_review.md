# PR #2086 리뷰 — #2017 HWPX curSz=0 sentinel 재직렬화 보존

- 작성 시각: 2026-07-09 16:30 KST
- PR: https://github.com/edwardkim/rhwp/pull/2086
- 작성자: `planet6897`
- base / head: `devel` / `fix/2017-cursz-fidelity`
- 문서 작성 시점 참고 head: `b2ba1fcdf62e01437a9a1874664d3a2306fecd96`
- 문서 작성 시점 참고 merge state: `BEHIND`
- reviewer assign: `jangster77` 요청 완료
- 처리 경로: `codex/planet6897-prs-review-20260709` 에서 여러 PR 누적 체리픽 검토

## 변경 범위

- `src/model/shape.rs`: `curSz` dimension별 zero sentinel 원본 여부 플래그 추가.
- `src/parser/hwpx/section.rs`: HWPX parse materialize 시 zero sentinel 여부 기록.
- `src/serializer/hwpx/picture.rs`, `src/serializer/hwpx/shape.rs`: HWPX serialize 시 zero sentinel 복원.
- `mydocs/report/task_m100_2017_report.md`: contributor 보고서.

## 체리픽 검토

- 누적 체리픽 순서: 5/11.
- 적용 커밋: `2937eb6f9` (`Issue #2017: HWPX 재직렬화 시 curSz=0 sentinel 복원 ...`).
- 충돌: 없음.
- 선행 PR 의존: 없음.

## 검증

- GitHub Actions: 원 PR head 기준 `Build & Test`, `CodeQL`, `Canvas visual diff` 등 성공 확인.
- `git diff --check upstream/devel...HEAD`: 통합 브랜치 fixup 이후 통과.
- `cargo fmt --check`: 통과.
- 이번 PR은 renderer 시각 출력 변경이 아니라 HWPX serializer fidelity 변경이다. PR diff에 원본 HWPX fixture가 없어 MCP PDF 산출 대상은 없다.
- contributor 보고서상 검증값: 원본 `curSz 0x0` 12건, `44752x0` 2건 복원, 구조 roundtrip diff 0.

## 판단

- 체리픽 가능 여부: 가능.
- blocking finding: 없음.
- 다만 원 PR diff 안에 재현 fixture나 직접 roundtrip test가 포함되지 않아, 장기적으로는 #2017 계열 fixture 보강이 바람직하다. 이번 누적 체리픽 자체의 충돌/빌드 리스크는 낮다.
