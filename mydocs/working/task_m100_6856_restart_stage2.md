# #6856 재착수 Stage 2 — 구조 식별과 조판부호 소비

- Issue: #6856. 기준 `b5eee9c50`, 계획 승인 기록 `3f0a67b30`.
- 계획: [재착수 구현계획](../plans/task_m100_6856_restart_impl.md).
- 상태: 첫 구현 절편의 집중 검증 완료. 메인테이너가 조판부호 식별 구현의 판정 통과를 승인함.
  #6856 전체 완료 승인은 아니다.

## 메인테이너 판정

2026-09-08 메인테이너: **“조판부호 식별 구현 통과입니다.”**

이번 승인 범위는 구조 기반 사각형/글상자 조판부호 식별 절편이다.
실행 중인 Studio WASM 교체·브라우저 재검증, 파서 손상/소유권 경계 및 전체 검증의
완료나 다음 단계 실행 승인으로 확대하지 않는다.

## 구현

1. `RectangleShape::control_kind()`가 `RectangleControlKind::{Rectangle, TextBox}`를 반환한다.
   파서가 보존한 `drawing.text_box` 존재만 사용하며 글자·그림·기타 컨트롤·배경·선은 판정에 사용하지 않는다.
   기하 Rectangle과 구조적 TextBox는 별개이므로 기존 `match shape`는 유지한다.
2. 레이아웃이 이 식별값을 원본 사각형 노드의 `ControlCode`에 전달한다.
   사각형 컨트롤의 내부 TextBox 레이아웃 노드는 중복 부호를 만들지 않으며 그 안의 별도 컨트롤 부호는 유지한다.
3. SVG·HTML·web canvas·Studio용 paint가 공통 부호 질의를 사용한다.
   paint의 자식 없는 도형 빠른 경로에서도 조판부호 ON이면 `[사각형]`을 출력한다.
4. `ControlCode`는 렌더 단계 내부 메타데이터다. 일반 render tree JSON 스키마에 직렬화하지 않는다.
   조판부호 OFF에서는 새 마커가 생성되지 않는다. 도형의 선·배경·배치·저장은 수정하지 않았다.

## 18가지 검증과 범위

- 내부 영역 없는 도형 × 배경 이미지 유무 = 2가지 일반 사각형.
- 내부 영역 있음 × 글자/그림/기타 컨트롤 유무 8가지 × 배경 이미지 유무 = 16가지 글상자.
- 원본 파서가 이미 구분한 정상 IR의 **명시적 식별과 소비**가 이번 제품 변경이다.
- HWP 파서의 0문단 처리나 레코드 소유권을 이번 절편에서 바꾸지 않았다.
  손상 입력을 정상 빈 구조로 복구하는 코드를 추가하지 않았다. 기존 파서가 구조를 잃는 모든 경우를
  해결했다고 주장하지 않으며, 미확정/손상 입력의 구조 보존은 별도 경계 조사로 남는다.

## 검증 이력

- baseline 라이브러리에서 동일한 공개 API 입력으로 조판부호 시험 실행:
  SVG/HTML/Studio paint 3개는 `[사각형]`이 0개여서 실패, 부호 OFF 보호 검사는 통과.
- 초기 OFF 시험은 JSON 기능 목록의 `paint.controlLabel`까지 잘못 세었다.
  실제 연산 `"type":"controlLabel"`을 검사하도록 시험을 정정한 뒤 baseline을 재실행했다.
  이 정정은 제품 회귀 수정으로 계산하지 않는다.
- 첫 구현에서 SVG/HTML은 통과했으나 paint의 자식 없는 leaf 빠른 경로에서 부호가 누락됐다.
  해당 경로를 수정한 후 paint도 통과했다.
- 새 시험은 `tests/cases/issue_6856_rectangle_structure.rs` 5개와
  `tests/cases/issue_6856_rectangle_control_labels.rs` 4개다.
  18조합, 0문단 영역, 캡션 분리, 중첩 부호, HWPX 영역 없음/빈 영역/그림 전용 영역,
  SVG/HTML/paint 부호와 부호 OFF를 포함한다.
- 집중 시험은 fixed target의 `cargo build --locked --lib --profile release-test` 결과에
  `rustc --test --extern rhwp=... -L dependency=...`로 연결해 실행한다.
  이 빠른 실행은 generated integration suite 기반 PR 전체 게이트를 대체하지 않는다.

## 실제 A4 샘플

최종 소스에서 release-test 라이브러리 빌드와 새 집중 시험 9/9, 기존
`issue_6852_group_rectangle_stroke` 회귀 6/6이 통과했다. 변경 파일 rustfmt 검사와
`git diff --check`도 통과했다. 총 15개 시험 중 하나가 18조합을 순회한다.
18개 별도 파일이나 18개 별도 테스트 함수가 있다는 뜻은 아니다.

한컴 생성 `a4-two-rectangles-and-textbox.hwp`와 `.hwpx`를 각각 새 코드로 읽어 출력했다.
두 형식 모두 SVG와 paint에 **일반 사각형 1개, 글상자 2개**다.
메인테이너가 사각형이라고 지칭한 위쪽 두 개체 중 하나에 빈 내부 문단 영역이 있기 때문에,
이번 승인된 구조 계약에서는 그 개체도 글상자로 식별된다. 글자의 존재로 판정한 결과가 아니다.

- SVG: `output/6856/identification/a4-hwp-control-codes.svg`
- SVG: `output/6856/identification/a4-hwpx-control-codes.svg`
- paint: 같은 폴더의 `a4-hwp-paint.json`, `a4-hwpx-paint.json`
- 원본 HWP/HWPX는 다시 저장하거나 변경하지 않았다.

## 남은 절차

- 최종 소스 집중 검사 및 #6852 기존 회귀 확인 완료(위 결과).
- 조판부호 식별 구현의 메인테이너 판정은 통과했다. 실행 중인 Studio용 Docker WASM
  빌드·교체 및 교체 후 브라우저 재검증은 미수행이다.
- 파서의 손상/소유권 경계, 편람 대표 자료 및 원래 계획의 추가 수용 조건은 미완료.
- review worktree generated suite, 전체 Rust lint/native·WASM·workspace, 전체 integration,
  Native Skia 게이트는 PR 준비 단계에서 별도 승인 후 수행한다. 현재 PR 준비 완료 상태가 아니다.
- 이 보고서의 최초 작성 시점에는 계획 기록만 커밋한 상태였다. 메인테이너의 다음 절차 승인 후
  제품 변경·집중 시험·본 보고서를 `157717f6d`로 커밋했다. push/PR은 하지 않았다.
  후속 경계 검증은 [Stage 3](task_m100_6856_restart_stage3.md)에 기록한다.
