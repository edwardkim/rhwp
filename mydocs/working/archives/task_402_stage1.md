# Task #402 Stage 1: 진단 로깅 + 가설 확정

## 작업 내용

`src/renderer/layout.rs::layout_shape_item` 진입부에 임시 `eprintln!` 로깅을 추가하여 TAC Picture가 처리될 때의 `y_offset`, `para_start_y[para_index]`, 결정된 `pic_y` 값을 출력했다.

```rust
// (임시) Stage 1 진단 로깅 — Stage 1 종료 후 제거
eprintln!(
    "[T402] layout_shape_item TAC pic: pi={} ci={} y_offset={:.2} existing_before={:?} pic_y={:.2} pic_h={:.2}",
    para_index, control_index, y_offset, __t402_existing_before, pic_y, pic_h
);
```

샘플 7쪽(0-indexed=6) 재생성 시 출력:

```
[T402] layout_shape_item TAC pic: pi=51 ci=0 y_offset=383.55 existing_before=Some(94.49) pic_y=94.49 pic_h=289.05
[T402] layout_shape_item TAC pic: pi=57 ci=1 y_offset=919.40 existing_before=Some(578.09) pic_y=578.09 pic_h=369.43
```

## 분석

| 케이스 | control_index | 같은 문단의 선행 TAC | y_offset | existing | 현재 pic_y | 정상? |
|--------|--------------:|---------------------|---------:|---------:|----------:|:-----:|
| pi=51 ci=0 | 0 | 없음 | 383.55 | 94.49 | 94.49 | ✅ |
| pi=57 ci=1 | 1 | Table(ci=0, tac) | 919.40 | 578.09 | 578.09 | ❌ |

### pi=51 (정상)
- `control_index=0`이고 같은 문단에 선행 TAC 컨트롤이 없음.
- `FullParagraph` PageItem이 먼저 처리되며 `layout_paragraph` (line 1820~) 내에서 `para_start_y[51] = 94.49` (paragraph 시작 y)로 설정됨.
- `FullParagraph` 내부 처리에서 line 높이만큼 y_offset이 진행되어 383.55가 됨.
- 이후 `Shape` PageItem 처리 시 `pic_y = para_start_y[51] = 94.49`로 paragraph 시작 위치에 그림이 그려짐 (FullParagraph가 사전 예약한 line 영역에 정상 배치).

### pi=57 (버그)
- `control_index=1`이고 같은 문단에 선행 TAC 컨트롤(Table at ci=0)이 있음.
- pi=57은 텍스트가 없는 TAC 전용 문단이라 `FullParagraph` PageItem이 없고, 대신 `Table(57,0)` → `Shape(57,1)`만 스케줄됨.
- `Table(57,0)` 처리 시 `layout_table_item`이 `para_start_y[57] = 578.09`로 설정하고 표 본체를 렌더하여 y_offset을 919.40으로 진행시킴.
- 이후 `Shape(57,1)` 처리 시 `para_start_y.entry(57).or_insert(...)`는 이미 존재하므로 갱신되지 않고, `pic_y = 578.09` 사용 → 표가 그려진 영역에 그림이 겹쳐 그려짐 (정상이라면 919.40).

## 가설 최종 확정

**두 케이스를 일관되게 처리할 조건:**
- 같은 문단(`para_index`)에서 현재 컨트롤(`control_index`)보다 **앞선 인덱스에 다른 TAC 컨트롤이 존재**하는 경우, 이 그림은 후속 line에 위치하므로 `pic_y`는 paragraph 시작 y가 아닌 **현재 진행된 y_offset**을 사용해야 한다.
- 선행 TAC 컨트롤이 없으면 기존 동작(paragraph 시작 y 사용) 유지.

### 단순 비교(`y_offset > existing_y + 1.0`)만으로는 부족한 이유
- pi=51 ci=0의 경우에도 `y_offset(383.55) > existing(94.49) + 1.0` 조건이 성립하지만, 정답 `pic_y`는 94.49임.
- → 선행 TAC 존재 여부가 핵심 판별 조건.

## 영향 범위 추정

- 영향: 같은 paragraph 안에 TAC 컨트롤(표/그림/도형) 2개 이상이 있고, 각자 다른 line에 배치되는 케이스에서 두 번째 이후의 그림.
- 비영향:
  - 같은 paragraph 안에 TAC 그림이 1개만 있는 일반 케이스 (가장 흔함). `control_index=0`이므로 선행 TAC가 없어 동작 변화 없음.
  - non-TAC anchor 그림 (위치 모드 별도 분기, 이번 수정 범위 밖).

## 산출물 / 상태

- 진단 로그는 Stage 1 종료 시점에 제거 완료. 빌드 통과 확인.
- Stage 2에서 위 가설에 따라 `layout_shape_item`에 선행 TAC 검사 + para_start_y 갱신 로직 추가 예정.
