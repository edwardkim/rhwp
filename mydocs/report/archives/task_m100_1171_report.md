# 최종 결과보고서 — Task #1171: 사각형 글상자 안 picture 클릭 hit-test / 속성 / 삽입

- **이슈**: [#1171](https://github.com/edwardkim/rhwp/issues/1171) (M100 / v1.0.0)
- **브랜치**: `local/task1171` (base: `local/devel`)
- **기간**: 2026-06-02 ~ 2026-06-03
- **결과**: 완료 (작업지시자 실환경 + 한컴 정합 확인)

## 1. 문제

`samples/tac-img-02.hwp` 6/7쪽의 "사각형(Shape, InFrontOfText) → 글상자(text_box) → paragraph
→ picture" 이중 중첩 picture 가 rhwp-studio 에서 클릭/선택/속성편집 되지 않았다. 작업 중
글상자 위 **이미지 삽입** 실패(별개 결함)도 발견되어 함께 처리했다.

## 2. 접근 — cellPath 일반화 (작업지시자 확정)

글상자를 "cell_index=0 sentinel 단일 셀 컨테이너"로 취급하여, 기존 표 셀 picture 의 cellPath
메커니즘을 글상자까지 확장했다(신규 평행 메커니즘 없음). text/equation/table 은 이미 이 방식을
쓰고 있었고 **picture 만 누락**되어 있었다.

## 3. 변경 요약 (단계별)

| 단계 | 영역 | 변경 |
|------|------|------|
| Stage 1 | `shape_layout.rs` | 글상자 picture 의 `layout_picture` 에 `CellContext`(sentinel) 전달 (None→Some) |
| Stage 1 | `rendering.rs` `collect_controls` | 사각형(Rectangle) 노드 조기 `return` 제거 → Table 처럼 자식 재귀하여 글상자 내부 picture 수집 (★ 진단 중 발견) |
| Stage 2 | `object_ops.rs` getter | `resolve_cell_by_path` → `resolve_paragraph_by_path`(표/글상자 모두) |
| Stage 2 | `object_ops.rs` `resolve_cell_paragraph_mut` | Shape arm 추가 (immutable 짝과 대칭화) → setter 글상자 지원 |
| Stage 3 | `input-handler-mouse.ts` | 글상자 내부 클릭이 텍스트 편집으로 단락되기 전 picture 선제 hit-test (picture 우선) |
| Stage 3 | `input-handler-picture.ts` `findPictureAtClick` | 클릭이 컨테이너 Shape + nested picture 둘 다 hit 시 picture 우선 반환 (★ E2E 중 발견) |
| Stage 4 | `insert.ts` | **무변경** — 기존 depth-1 cellPath 재구성이 글상자에 그대로 동작(E2E 확정) |
| Stage 6 | `input-handler-table.ts` `finishImagePlacement` | 글상자 위 이미지 드롭은 cellPath 없이 본문 para(parentParaIndex)에 floating 삽입 → 한컴처럼 본문 sibling (★ 작업지시자 실환경 발견) |

> 계획 대비 편차 3건(★)은 모두 진단/검증 과정에서 발견되어 문서화했다(stage1/stage3/stage6 보고서).
> 핵심 통찰: 셀과 "다른 방식"이 아니라, 셀이 이미 하던 cellPath 재귀/resolver 를 글상자에 동일
> 적용 — 통합. (작업지시자 질의 "셀이랑 완전 다른 방식?" → 아니오, 동일 메커니즘 확장.)

## 4. 검증

- **백엔드**: `cargo test` 전체 **1948 passed / 0 failed**. 신규 round-trip 테스트
  `tests/issue_1171_textbox_picture_cellpath.rs`(cellPath sentinel 노출 + by_path get/set
  width 15040→20040). 표 셀 by_path 회귀 0.
- **프런트엔드**: `npx tsc --noEmit` 변경 파일 에러 0. E2E(headless, WASM 재빌드):
  - `e2e/textbox-picture-1171.test.mjs`: cellPath 노출 + findPictureAtClick=image + 속성 round-trip.
  - `e2e/textbox-picture-insert-1171.test.mjs`: TDD red→green, 글상자 위 드롭 → 본문 floating
    sibling(treat_as_char=false, shape 독립).
- **한컴 정합(데이터 모델 + z-order)**: 삽입 이미지 = 본문 floating Picture(글상자 sibling,
  control 1 vs Shape control 0), `wrap=Square` → InFrontOfText 글상자 **뒤**에 렌더. 작업지시자
  실환경 + 한컴 직접 비교 **정합 확인**.
- **빌드 메모**: studio 프로덕션 `npm run build` 는 무관한 기존 환경 결함(`canvaskit-wasm`
  미설치)으로 막힘 — 변경 파일 tsc 0 에러, dev 서버 정상 동작으로 대체 검증.

## 5. 커밋 (local/task1171)

```
20fd6e16 Stage6 글상자 위 이미지 드롭 → 본문 sibling 삽입
70bd6389 Stage3 보강 findPictureAtClick picture 우선 + E2E
00d1ab77 Stage3 프런트엔드 picture 우선 hit-test
0bc8e327 Stage2 백엔드 by_path picture getter/setter 글상자 지원
62b58e80 Stage1 글상자 안 picture cellPath 노출
d013b721 수행/구현 계획서 작성
```

## 6. 범위 밖 (후속 권고)

- 글상자 안 **중첩 Shape(도형)** 의 get(`get_cell_shape_properties_by_path_native` 는 아직 표 전용
  resolver) — setter 는 Stage 2 로 부수 지원됨. get 정합은 후속.
- 다단계 중첩(표→글상자→표→picture) 의 insert.ts cellPath 견고화(`ref.cellPath` 직접 사용 + 키 변환).
- 글상자 **내부 콘텐츠로서** 이미지 삽입(글상자 텍스트 편집 상태) 경로.

## 7. 다음

`local/task1171` → `local/devel` merge 후보. 작업지시자 승인 시 진행.
