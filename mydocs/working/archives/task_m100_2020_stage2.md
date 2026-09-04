# Task M100-2020 Stage 2 — 복학원서 접수증 도장/날인선 보정

## 목표

Stage 1 커밋 `3de39988e` 이후 남은 복학원서 1쪽 하단 접수증 차이를 처리한다.

작업지시자가 지적한 증상:

- 기준 PDF 대비 접수증 우측 하단 도장 원 위치가 다르다.
- `복 학 원 서 접 수 증` 위에 있어야 하는 `-------------(인)------------` 날인선이 rhwp 출력에서 보이지 않는다.

## 기준 자료

- 원본: `samples/복학원서.hwp`
- 기준 PDF: `pdf/issue2020/복학원서-2022.pdf`
- 기존 기대 스크린샷: `samples/issue2020/issue2020_expected_bokhak_p1.png`
- 최신 sweep 산출물: `output/issue2020/recheck_20260708/full_after_remaining_fix_p1/bokhak/`

## 조사 순서

1. 원본 HWP의 접수증 영역 문단/shape/control 구조를 `dump`, `dump-pages`, render tree, SVG로 확인한다.
2. `-------------(인)------------`가 원문 텍스트, PUA, shape 선, 또는 조판부호 조합 중 어디에서 오는지 판정한다.
3. 도장 원 위치가 anchor/relative position, z-order, wrap, paragraph flow 중 어디에서 틀어지는지 분리한다.
4. 문서명 기반 예외 없이 구조 가드로 수정한다.
5. `tests/issue_2020.rs` 또는 기존 #937 테스트에 회귀 가드를 추가하고, 복학원서 1쪽 visual sweep으로 확인한다.

## 완료 기준

- 복학원서 1쪽 접수증 영역에서 날인선 `-------------(인)------------`가 출력된다.
- 우측 하단 도장 원 위치가 기준 PDF 방향으로 이동한다.
- 기존 `issue_937` PUA 치환 및 U+F081C filler 미출력 정책을 깨지 않는다.
- `cargo fmt --check`, `git diff --check`, 관련 테스트, 복학원서 visual sweep을 통과한다.
