# Task M100-1205 Stage 1 완료 보고서 — RED 테스트 고정

## 1. 작업 범위

구현계획서 Stage 1에 따라 #1205 결함을 소스 수정 전 RED 테스트로 고정했다.

변경 파일:

- `src/renderer/layout/integration_tests.rs`

## 2. 추가한 테스트

테스트명:

```text
task_1205_para_border_none_sides_do_not_render_vertical_edges
```

검증 의도:

- 문단 `borderFill`의 side 구성 `[left, right, top, bottom]`이 `[NONE, NONE, SOLID, SOLID]`인 경우를 합성한다.
- 렌더 트리에서 stroke 있는 4면 `RectangleNode`가 없어야 한다.
- 렌더 트리에서 좌우 수직 `LineNode`가 없어야 한다.
- top/bottom 가로선은 존재해야 한다.

## 3. RED 결과

실행 명령:

```text
cargo test --lib task_1205 -- --nocapture
```

결과:

```text
FAILED
```

핵심 실패:

```text
assertion `left == right` failed: left/right NONE 문단 border 는 좌우 수직선을 렌더하면 안 됨
  left: 2
 right: 0
```

즉 현재 구현은 `left/right = NONE`인 문단 border에서도 좌우 수직선 2개를 생성한다.

## 4. 판단

Stage 1 RED 테스트는 #1205 이슈 본문의 원인 가설과 일치한다.

현재 결함은 HWPX parser의 side별 `NONE` 매핑 누락이 아니라, 문단 border group 렌더링 단계에서 side별 visibility를 반영하지 않는 문제로 고정됐다.

## 5. 다음 단계

작업지시자 승인 후 Stage 2로 진행한다.

Stage 2 범위:

- `src/renderer/layout.rs` 문단 border group 렌더링 경로 수정
- visible side만 `LineNode` 생성
- 4면 동일 visible stroke일 때만 기존 `RectangleNode` stroke 경로 유지
