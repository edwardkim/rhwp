# 0.9.0 승격 검증의 Full Renderer Sweep 실패 조사

## 확인한 범위

2026-11-08 KST, 기준 devel `78615c52a7c7d7a6b25fad57c3bf1041fb2dfcf2`.
[수동 승격 실행](https://github.com/edwardkim/rhwp/actions/runs/42434345354)은
`BlogForm_BookReview.hwp`를 HTTP 404로 읽지 못해 실제 첫 비교 전에 실패했다.
Oracle의 assets 누락을 보정하는 #8766과 원인이 다르다.

## 1. 파일 전달 실패: 로컬 보정·검증 완료

`rhwp-studio/vite.config.ts`의 `/samples` middleware는 저장소 `samples/`에 파일이
없으면 바로 404를 반환한다. 같은 URL을 제공하는 Vite `public/samples/`에 있는 문서도
가려진다. 기존 #2185의 [실행 기록](archives/task_m100_2185_stage4.md)에도 같은 실패가
있었으며 당시에는 읽을 수 있는 여섯 표본만 검사했다. 이번 승격은 전체 표본을 요구한다.

- 원래 11개 표본 중 5개 HTTP 404를 실제 재현했다.
- 저장소 파일이 없고 public에 실제 regular file이 있을 때만 Vite의 public 처리로 넘기는
  보정으로 11개 표본의 응답과 원본 바이트가 모두 같았다.
- 저장소 샘플 우선순위, 없는 문서의 404, 상위 경로 접근 403도 실제 HTTP로 확인했다.
- 새 비시각 HTTP 회귀는 기존 `npm test` 자동 수집에 포함했다. 별도 CI 호출을 추가하지 않았다.
- Studio 2,778건 통과·기존 skip 2건, TypeScript와 production build 통과.
- 첫 전체 테스트에서는 오래된 로컬 WASM에 `insertMemo`가 없어 1건 실패했다.
  정확한 기준 devel의 [WASM Build](https://github.com/edwardkim/rhwp/actions/runs/42434338951)
  artifact `13599253771`을 적용한 뒤 전체 테스트가 통과했다. 실제 브라우저가 받은 WASM의
  SHA-256은 `385adc68f5a817ad8aa557f4572a7366f4ea04a7efde3bbe9af9fae1f6d8c378`로 일치했다.

원본 문서·기대값·workflow·비교 임계치는 변경하지 않았다. HTTP 보정은 로컬 상태다.

## 2. 입력 복구 뒤 드러난 Legacy/Layer 본문 clip 차이

같은 WASM으로 11개 표본의 첫 쪽을 실제 실행했다. 10개는 기존 기준을 통과했으나
BlogForm 1쪽은 `570 / 440,960 = 0.12926%`로 현행 `0.05%` 기준을 넘었다.
이 값은 **Legacy/Layer RGBA 차이율**이며 독립 한컴 PDF와의 실루엣 일치율이 아니다.

직접 이미지를 대조하면 차이는 오른쪽 표 테두리의 한 픽셀 열(x=506, y=196..765)에
있으며 max channel delta는 41이다. 원문 누락이나 페이지 수 차이로 판정하지 않았다.

실제 Canvas `rect` 호출을 추적한 결과:

| 경로 | 본문 clip x | 폭 |
| --- | ---: | ---: |
| Legacy | 30.000000000000004 | 480.7333333333334 |
| Layer | 30.000000000000004 | 476.7333333333334 |

- `src/renderer/web_canvas.rs`의 Legacy Body replay는 비표식 모드에도 저장 clip에
  4px를 더한다. 현재 분기의 커밋은 `448853ae787`이며 최초 코드까지 이전 이력을 추적했다.
- `src/paint/builder.rs`는 표식이 없으면 저장된 본문 clip을 유지한다. 이후 Layer replay가
  그 결과를 그대로 소비한다. 표시용 표식·텍스트 그림자 확장은 별도 계약으로 남아 있다.
- 진단 중 브라우저에서 Legacy의 해당 clip에 추가된 4px만 제거하면, 같은 문서·같은
  호출에서 차이가 **570 → 0픽셀**이 된다. 이는 원인 분리 실험이며 제품 보정 완료나
  독립 한컴 정답 검증으로 승격하지 않는다. 제품 Rust는 아직 수정하지 않았다.

원 해결 #6590 → PR #6591의 본문 폭/표 우단 처리와 독립 출력 근거도 확인했다.
[기존 검토](../pr/archives/pr_6591_review.md)는 `pdf/BlogForm_BookReview-2020.pdf`를
사용했다. 비슷한 이름의 다른 PDF와 혼동하지 않고 입력 SHA·생성 버전·독립 출력의 우단을
대조해야 한다.

## 필요한 다음 처리

1. 저장 본문 clip·표 외곽선 점유와 비인쇄 표식 확장의 계약을 독립 출력과 대조한다.
2. 검증된 공통 경계를 Legacy와 Layer가 소비하도록 최소 보정하고, 상수 확장으로
   화면을 맞추지 않는다. 기본 화면/표식 표시의 반례를 함께 확인한다.
3. 실제 원 사례의 Native/fresh WASM 전쪽·확대 판독, 기존 전체 Canvas 비교와
   변경 범위의 필수 Rust·Studio 검증을 완료한다. 기대값·임계치 완화로 통과시키지 않는다.
4. 검증된 보정을 별도 devel PR로 통합한 뒤 최종 devel SHA를 고정해 승격 workflow를 실행한다.

#8766의 승인된 CI·self-review·병합은 계속한다. 위 추가 실패를 알고 있는 상태에서
비용이 큰 13개 승격 workflow를 먼저 반복하지 않는다. main 승격·태그·실제 게시도 미완료다.
