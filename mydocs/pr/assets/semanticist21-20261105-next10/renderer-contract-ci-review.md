# PR #8640의 Render Diff 계약 보완

첫 제출 head `4c425b3f77800f5f16c3fd099f7ad8fe045edb0a`의 [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/42011334625/job/127191010890)는 실제 이미지 비교 전에 `e2e:renderer-contract`에서 실패했다. 이 로컬 계약 검사를 제출 전에 실행하지 않은 누락을 확인했다. 같은 Node 검사로 수정 전 실패를 재현했고, 임의 재실행이나 검사 생략으로 처리하지 않았다.

## 원 계약과 독립 근거

실패 입력과 기대값은 [PR #3638](https://github.com/edwardkim/rhwp/pull/3638)의 `c7be19d7d07ea470e0690908db190aeffe19bc3d`에서 왔다. [기존 검토](../../archives/pr_3638_review.md)는 이 검사가 CanvasKit backend replay와 fallback을 보호하며 한컴 Print 충실도 증거는 아니라고 구분한다. 이후 원 파일의 마지막 변경은 [PR #8296](https://github.com/edwardkim/rhwp/pull/8296)의 `3ddf77af61b10a9ace10224588ad68841e79b2f1`이며, 원문자의 소유·PUA 글꼴 보호는 이번에 변경하지 않았다.

이번 #8369의 공통 `char_outline_stroke → paint/json::outlineStroke → CanvasKit`은 독립 한컴 Print에 따라 투명한 내부와 1/600 inch 획을 사용한다. 옛 합성 입력에는 `outlineType`만 있고 공통 획 정보가 없었으며, 기대값은 흰색 채움과 글자 크기 비례 폭 0.8이었다. 독립 출력은 [전쪽 TSV](validation/post8622/summary.json)의 문자 외곽선 1쪽과 PDF 명시 TextPaint 7쪽에서 Native/fresh WASM 모두 100.00%이며, [600 DPI 확대와 실제 판독](validation/pre8622/direct-reading.md)을 완료했다. 합성 계약의 기대값을 변경하기 전 이 원 해결 PR·현재 공통 소비·독립 출력 관계를 확인했다.

## 보정과 남는 보호 범위

제품 코드와 입력·PDF·golden·문턱은 변경하지 않았다. 기존 합성 양성 입력에 생산된 `outlineStroke`를 명시하고, FakePaint에 실제 소비하는 cap/join/PathEffect API를 제공했다. 그림자 위치·장평·배경 검사는 유지하며 외곽선 내부를 흰색으로 덮지 않고 독립 pen으로 그리는지 확인한다. 점선의 물리 반복 구간을 크기 비례로 재계산하지 않는지와 PathEffect 해제를 추가 확인한다. 공통 획 정보 누락·무효 폭은 unsupported로 남아 fallback 대상이 되는 음성 대조군도 보존한다. 다른 페이지의 절대 좌표 핀이나 관계 검사는 바꾸지 않았다.

## 실제 로컬 재검증

[명령·소스 동일성·결과 원본](validation/renderer-contract-recovery.json)을 보존했다.

- 수정 전 같은 Node 계약 FAIL / 수정 후 PASS.
- CanvasKit font coverage PASS.
- 실제 고정 Chromium 1660786 Canvas 비교 3/3 PASS. KTX 0.01862%, 나머지 2건 0% 차이.
- direct PDF compatibility gate 3/3 PASS. 별도 Canvas↔SVG-derived PDF report-only 경고 4개는 성공한 direct gate와 구분하며, 그 경고를 해결했다고 표시하지 않는다.
- 실제 CanvasKit readiness 8/8 PASS·누락 0·runtime 오류 0.

최적화 WASM `11012c56c9463f4eca27426ab118a2c73a54d90418597315f3df7c0211d2b78d`와 최종 제품 source 584c9503ba의 실제 산출물을 사용했다. 이번 변경은 e2e 합성 검사와 이 보완 기록뿐이며 이미 완료한 Rust·Native·전쪽 시각 검증의 제품 subtree는 같다. 로컬 전체 Cargo 검증을 반복하지 않는다. 첫 head의 CI·Native·Lint·Frontend·A–E·CodeQL·Adapter·Proptest는 성공했고 Render Diff만 실패했다. 해당 실행이 모두 종료된 뒤 보정 commit을 한 번 push하며, 새 head의 실제 GitHub 필수 검증과 self-review를 확인하기 전에는 merge하지 않는다.
