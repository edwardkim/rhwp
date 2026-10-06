# 최종 보고서 — Task M100 #2145: 개요번호 `^n`/`^N` 레벨 경로 자동코드 구현

- 이슈: #2145 / 브랜치: `fix/2145-outline-levelpath` / 작성일: 2026-07-10
- 계획서: `mydocs/plans/task_m100_2145.md`, `task_m100_2145_impl.md`

## 증상 → 원인

비공개 실문서(로컬 `1.hwp`, 웹 취약점 점검 보고서 5쪽)의 개요 문단이 rhwp 렌더에서
번호 대신 **리터럴 `^N`** 으로 출력. 한글 COM oracle(HPrint 1-up PDF, 5쪽 일치)은
`1.` / `1.1.`~`1.4.` / `2.` / `3.` 표시.

원인: `expand_numbering_format`(`src/renderer/layout/utils.rs`)이 `^1`~`^7`만
치환하고, OWPML ParaHeadType 스펙의 자동 코드 `^n`(레벨 경로, `1.1.1`)과
`^N`(레벨 경로+후행 마침표, `1.1.1.`)이 미구현 — 미치환 문자는 리터럴 통과.

## 수정

- `expand_numbering_format`에 `current_level`(0-based) 파라미터 추가,
  수준별 번호 포맷을 `format_level` 클로저로 추출, `^n`/`^N` 분기 구현
  (수준 1~현재 수준을 각 수준의 `number_format`으로 포맷해 `.` 연결,
  `^N`은 후행 `.` 부가). 호출부 `apply_paragraph_numbering` 1곳 갱신.
- 변경 파일: `utils.rs` +52/−22줄 내외, `paragraph_layout.rs` 호출부,
  `tests.rs` 단위 테스트.

## 검증 자산

**대외비 정책에 따라 fixture/PDF/통합 테스트는 저장소 미포함** — 회귀는 단위
테스트로 고정하고, 검증 자산은 로컬 보관한다.

| 자산 | 위치 (로컬, gitignore) |
|------|------|
| 재현 fixture (합성 HWPX, 개요 4문단 level 0→1→1→0, 전 수준 `^N`) | `output/poc/issue_1hwp_outline/local_assets/issue2145_outline_levelpath.hwpx` |
| 권위 PDF (한글 2022 COM HPrint 1-up) | `output/poc/issue_1hwp_outline/local_assets/issue2145_outline_levelpath-2022.pdf` |
| 통합 테스트 원본 (fixture 의존, 미커밋) | `output/poc/issue_1hwp_outline/local_assets/issue_2145_outline_levelpath.rs` |
| 단위 테스트 (**커밋됨**, 회귀 가드) | `src/renderer/layout/tests.rs` `test_expand_numbering_format_level_path{,_mixed_format}` |

fixture는 `issue1549_empty_host_float_clamp.hwpx` 구조에서 본문·numbering만 교체한
합성본으로, **한글이 직접 열어 재조판한 출력**을 권위 PDF로 확보 —
한글도 `1. / 1.1. / 1.2. / 2.`를 표시하여 rhwp 수정 후 렌더와 일치.
(비공개 원문서 및 시각 대조 산출물도 로컬 `output/poc/issue_1hwp_outline/`.)

## 게이트

- `cargo test --release` 전체: 실패 0
- `cargo clippy --release`: warning/error 0
- `hwpx_roundtrip_baseline` / `visual_roundtrip_baseline`: 통과
- 코퍼스 영향: 추적 샘플 8건이 `^n`/`^N` numbering 정의를 보유하나 **본문에서
  Outline 문단을 사용하는 샘플은 0건** (diag 전수 확인) — 기존 샘플 시각 변화 없음.

## 판정

한글 oracle과 픽셀 수준 대조 완료(비공개 문서 페이지 2~4 + 합성 fixture),
번호·카운터 전진(`2.`, `3.`)·하위 수준 경로(`1.1.`~`1.4.`) 모두 일치. 수정 완료.
