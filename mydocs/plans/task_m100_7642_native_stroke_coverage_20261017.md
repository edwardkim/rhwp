# #7642 Native 합성 획의 장치 단위 소비 검토

Issue: #7642.

## 독립 근거와 원인 범위

- producer `145ea004`의 법령 50쪽: Native PNG raw 99.17636%, 같은 기하의
  RHWP direct PDF를 PDFium으로 출력한 진단 raw 99.99123%다.
- stage2810에서 원 한컴 Print PDF의 복제본에 있는 `2 Tr` 획 너비 25개만
  0.05배·0.5배로 줄여도 96DPI raster는 원본과 픽셀 단위로 같았다.
  변경 없는 직렬화 대조군도 동일하며, 4배에서는 10,631픽셀이 달랐다.
- PDFium의 AGG `RasterizeStroke`와 Skia `SetupStrokePaint`는 장치 단위의
  최소 획을 소비하고 fill/stroke를 나누어 그린다. main 소스 snapshot은
  stage2809에 있으며 설치 binary와 같은 revision이라고 가정하지 않는다.

이는 원문 좌표나 글자 크기를 조정할 근거가 아니다. 같은 원 윤곽선·em 획이
vector와 raster에서 소비되는 차이를 보정한다. 특정 문서·쪽·문자열·글꼴
이름을 조건으로 삼지 않는다.

## 생산에서 실제 소비까지

`SkiaLayerRenderer::render_page_to_canvas_with_options`의 실제 최종 surface에서
raster/vector 여부를 한 번 확정한다. `ReplayCacheContext`에 이 상태를 담고
cache seed에도 포함한다. 정적 picture를 기록할 때 중간 Canvas의 surface
종류를 다시 추측하지 않는다. 기존 current matrix는 기록 Canvas에도 적용되고
cache key에 포함되어 있으므로 각 DPI·회전·장평의 장치 변환을 보존한다.

이 상태를 `SkiaTextReplay`와 합성 굵기 path helper에 전달한다. vector는
원래 0.02em 획을 유지한다. raster에서는 현재 CTM의 역변환으로 장치 한
단위를 원 좌표계에 옮긴 뒤 PDFium의 최소 획 계약을 소비한다. 이는 문단
위치를 고정하는 px 보정이 아니며 DPI가 바뀌면 원 좌표계의 값도 달라진다.
fill과 stroke는 같은 원점·윤곽선에서 순서대로 소비한다.

기존 대형 `renderer.rs`는 surface 상태의 인자 전달만 최소 수정한다.
새 정책과 실제 그림 코드는 작은 기존 helper에 둔다. 이 작업을 이유로
4천 줄의 renderer 모듈 전체를 재구성하지 않는다.

## 반례와 검증

실제 Bold face·보통 글자·명시 outline은 기존 경로를 유지한다. path 누락은
부분 출력 전에 기존 fallback으로 되돌린다. vector PDF의 원 획, 서로 다른
DPI, 장평·회전, 같은 renderer의 raster/vector 순서 전환과 정적 picture
cache hit/miss를 대조한다. 장치 변환이 유효하지 않은 경우 출력을 숨기지 않는다.

현재 145ea 후보의 진행 중인 Native/fresh WASM·Rust 검증을 끝까지 수집하고
새 후보를 공유 target에서 순차 빌드한다. 먼저 작은 실제 영향 범위와 반례의
방향을 확인한 뒤 전쪽 Native/fresh WASM·base 하락 비교를 갱신한다. 후속
보정은 아직 구현·실행하지 않았으며 실제 출력 개선이나 검증 완료를 주장하지 않는다.
새 실물 회귀·baseline은 전쪽 양쪽 90%와 확대 판독을 확인한 뒤 추가한다.
