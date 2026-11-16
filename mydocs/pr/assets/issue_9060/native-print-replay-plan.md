# #9075 Native Print 재생 보정 단계

## 원인과 독립 근거

[잔여 차이 분석](native-residual-diagnosis/README.md)의 같은 조판 분리 대조에서 글자 기하
재생과 PDF에 맞는 맑은 고딕 6.68 공급을 함께 적용하면 2·5·12쪽 모두 raw 100%, 미대응 0개다.
제품의 이전 인쇄 세대 경로는 Native `draw_str`의 장치 글자 격자를 소비하고 있었다.
독립 Print의 glyph 윤곽·전진 폭과 실제 layer 원점을 유지해야 한다. 글자 폭·문단·표 위치를
보정하거나 문서·쪽·글꼴명으로 분기하지 않는다. 사양이 특정 힌팅 방식을 강제한다고 주장하지 않는다.

## 공통 소비 경로

`PageLayerTree`의 Print profile → Native `ReplayCacheContext`의 실제 raster 글자 정책 →
`SkiaTextReplay.geometric_text` → `fonts::preserve_geometry`와 `draw_positioned_cluster`.
Native의 Print raster는 인쇄 세대에 관계없이 기존 단색 SFNT 윤곽 경로를 소비하게 한다.
공유 layer의 세대 설정과 Canvas/WASM 정책은 바꾸지 않는다. Screen·벡터 출력은 기존 경로를
유지한다. 여러 scalar의 shaping과 bitmap·컬러·가변 글꼴의 기존 재생 분기도 유지한다.
실제 Native 정책을 cache 신원에도 포함한다.

## 검사와 완료 경계

1. 특정 사설 글꼴 없이, 저장소가 공급하는 공개 TTF와 같은 글자의 상대 이동으로 이전 인쇄
   세대의 실제 Native baseline 손실을 수정 전 FAIL로 확인한다. 같은 renderer에서 두 세대의
   Print와 Screen을 대조하며 기대값을 현재 구현에서 추출하지 않는다.
2. Native 공통 재생 경로를 보정하고 같은 검사를 PASS로 확인한다. 기존 Print 기하·Screen·
   cache 전환·프로그램 선택 검사도 수행한다.
3. 같은 86036 원본·기존 PDF의 전 14쪽을 실제 제품 binary로 재출력한다. 이때 로컬에만 있는
   PDF 일치 글꼴 프로그램을 공급하고 Native/fresh WASM의 전쪽 TSV와 주장 영역 5·6쪽을
   직접 확인한다. 진단 PNG를 제품 결과로 복사하지 않는다.
4. #8389의 기존 Print 보존 사례와 정본 보호 대상·편람 gate를 실제 수정 binary로 대조한다.
   후퇴가 있으면 원인부터 수정하며 기대값·TSV를 낮추지 않는다. 집중 회귀·전쪽 검증만으로
   전체 로컬 필수 검증을 대신하지 않는다. 모든 필수 검증 완료 전에는 PR을 제출하지 않는다.

위 내용은 보정 시작 전에 작성한 계획이다. 제품 Native 전 14쪽 100%·최소 글꼴 집중 6건·
Native 필수 3종 통과와 내용 보존은 [보정 단계 결과](native-print-stage/README.md)에 기록했다.
정본 보호의 모든 페이지와 전체 PR 필수 검증은 계속 진행하며, 완료로 판정하지 않았다.
