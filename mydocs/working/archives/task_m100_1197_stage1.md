# Stage 1 완료보고서 - Task M100-1197

- 이슈: #1197
- 단계: Stage 1 - 재현 계측 및 RED 회귀 고정
- 브랜치: `local/task1197`
- 기준 커밋: `upstream/devel` `f6ffe9d6`
- 작성일: 2026-06-02

## 1. 원본 재현 자료 확인

저장소에서 다음 패턴으로 원본 재현 문서를 검색했다.

```text
온새미로
Onsaemiro
onsaimiro
2027
```

검색 결과, 이슈 본문의 `[2027] 온새미로 1 본교재.hwpx`와 관련 PDF 정답지는 현재 저장소에 없다.
따라서 Stage 1에서는 원본 파일 기반 SVG 산출 대신, #1197의 레이어 순서 계약을 재현하는 최소
synthetic RenderTree 테스트를 추가했다.

## 2. RED 회귀 테스트 추가

추가 파일:

- `tests/issue_1197_svg_object_zorder.rs`

테스트 구성:

1. root 자식 순서로 z-order 의도 순서를 모델링한다.
   - `Z01_LOW_TABLE`: 낮은 zOrder BehindText 표
   - `Z11_IMAGE`: 전체 페이지 BehindText 이미지
   - `Z12_FINAL_TABLE`: 높은 zOrder 최종 표시 표
   - `Z69_FRONT_SHAPE`: front shape 역할 마커
2. 현재 SVG 렌더러가 `ImageNode.text_wrap=BehindText`만 plane 1로 분류하고,
   `TableNode`는 Flow plane으로 남기는지 확인한다.
3. 기대 순서 `Z01_LOW_TABLE < Z11_IMAGE < Z12_FINAL_TABLE < Z69_FRONT_SHAPE`를 단언한다.

## 3. 검증 결과

### 신규 #1197 RED 테스트

명령:

```sh
cargo test --test issue_1197_svg_object_zorder -- --nocapture
```

결과: 의도한 RED 실패.

핵심 실패 메시지:

```text
low z-order BehindText table must be below the full-page image.
Current SVG order puts image offset 310 before low table offset 332.
```

해석:

- 현재 SVG 출력은 BehindText 이미지를 낮은 zOrder 표보다 먼저 그린다.
- SVG는 뒤에 나오는 요소가 위에 합성되므로, 낮은 zOrder 표가 이미지 위에 남는다.
- 이슈 #1197의 `PROLOGUE`/출처 목록이 전체 페이지 이미지 위에 남는 현상과 같은 계약 위반이다.

### 기존 #1167 회귀 테스트

명령:

```sh
cargo test --test issue_1167_svg_behindtext_zorder
```

결과:

```text
1 passed; 0 failed
```

해석:

- 이미지 단독 BehindText 워터마크 보정은 현재도 유지된다.
- 남은 문제는 Picture/Table/Shape가 같은 z-order 축에서 섞이는 계약이다.

## 4. Stage 1 결론

Stage 1에서 #1197의 최소 RED 조건을 고정했다.
원본 재현 파일은 저장소에 없으므로, Stage 2에서는 synthetic 테스트를 기준으로 RenderNode 레이어
메타데이터 계약을 확정하고, 원본 파일이 확보되면 별도 시각 산출로 보강한다.

## 5. 다음 단계

Stage 2 승인 후 다음 작업을 진행한다.

1. `RenderNode` 공통 optional layer 메타데이터 도입 여부 확정
2. `text_wrap`, `z_order`, 안정 정렬 순서를 RenderNode에 전달하는 방식 구현
3. SVG `node_z_plane()`이 이미지 외 표/도형도 레이어 계약으로 분류하도록 준비
