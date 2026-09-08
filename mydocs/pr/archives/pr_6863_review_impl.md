---
kind: implementation
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-08
---

# PR #6863 메인터너 보정

## 1단계: 원인과 회귀 고정

- 원 contributor head: `1e82075a8ba4d79ac12fb11255a8563f87ce2281`.
- 로컬 적용 head: `2e17c6a80f7050d92f9b4a28476f3e6dec6e5c5b`.
- 사용자 승인: 머지 보류 사유를 메인터너 보정으로 처리. 원격 push/PR/merge는 별도다.
- 원인: 글상자 내장 표 셀의 Shape 경로가 모든 인라인 도형에 첫 줄과 같은 영역을 전달한다.
  일반 표 셀은 문단이 등록한 개별 좌표를 소비하지만 이 경로에는 해당 처리가 없다.
- 공개 합성 입력의 같은 줄/서로 다른 저장 줄에서 두 도형의 글자가 모두 `(0, 0)`에 겹쳤다.
- 기존 `tests/cases/issue_6735_textbox_table_cell_shape.rs`에 위치 분리 회귀를 추가한다.
  기존 inline/floating 내용 보존과 3단계 cell path 계약도 유지한다.

## 보정 범위와 검증 계획

1. 문단 조판이 등록한 인라인 Shape 좌표를 우선 사용한다.
2. 빈 셀 문단의 미등록 도형은 기존 줄 소유권 해석기로 대상 줄을 찾고, 같은 줄 TAC 폭을
   합산해 문단 정렬과 앞 개체의 가로 진행을 반영한다. 부동 도형 경로는 변경하지 않는다.
3. 새 위치 회귀와 원 PR focused test, 독립 SVG/PNG 재현을 검증한다.
4. 기존 유사 문서/PDF를 재사용해 보정 전후를 비교한다. 임시 log/SVG/JSON은 커밋하지 않는다.
5. 집중 검증 결과 이후 PR 직전 전체 검증은 승인 단계에서 수행한다. 기존 원 head CI를
   새 보정 code head의 전체 회귀나 Clippy 성공으로 기록하지 않는다.

## 결과

수정 전 focused nextest: 4건 중 기존 2건 통과, 새 위치 회귀 2건 실패(exit 100).
두 label 좌표는 두 경우 모두 `(0, 0)`이다. 첫 실행은 자동 suite 재배정 전 번호를 사용해
0건(exit 4)이었으며, 새 manifest의 suite 009로 재실행한 위 결과만 회귀 근거로 삼는다.

## 2단계: 메인터너 보정과 집중 검증

- 1단계 고정 commit: `6bb9b5298`.
- `table_cell_content.rs`: 문단에서 등록한 좌표를 동일 cell context로 조회한다.
  미등록 TAC는 `control_line_seg_index`로 소유 줄을 해석하고 같은 줄 개체 폭을 합산한다.
  정렬은 개별 도형이 아닌 전체 줄 개체 묶음에 한 번 적용한다. floating 분기는 유지한다.
- 추가 회귀: 같은 줄, 다른 저장 줄, 가운데/오른쪽 정렬, 도형 앞 텍스트 4건.
  원 기존 내용 보존 2건과 함께 6/6 통과했다.
- 원 PR의 네 module을 manifest로 다시 선택해 **18/18 통과**, 필터 밖 677 skipped,
  exit 0. 빌드 37.39초, 테스트 0.068초. 소스 변경 뒤 첫 빌드는 5분 48초였다.
- 독립 probe: 같은 줄 `FIRST=(0,0), SECOND=(80,0)`, 다른 줄
  `FIRST=(0,0), SECOND=(0,40)`. 두 assertion 모두 exit 0이며 Chrome PNG도 직접 확인했다.
- `cargo fmt --all -- --check`, `git diff --check`, suite manifest `--check` 통과.
- 보정 바이너리 SHA-256:
  `ef407f4e48c3aed60a814fc73c36c8d532480641cc77c48e7f681320ad6608dc`.
- 유사 문서 4종, 40쪽 전체 SVG는 보정 전과 byte-identical이다. 신규 전체 회귀,
  세 Clippy, Native Skia, WASM 검증은 수행하지 않았으며 PR 직전 승인 검증으로 남긴다.
- 원격 push, PR 생성, comment, merge는 수행하지 않았다.
