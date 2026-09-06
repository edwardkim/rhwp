---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_6662_stage11_overlay_shape_owner.md
last_verified: 2026-09-06
---

# 열린 이슈 재검증 11단계: 분할 셀의 겹침 묶음 도형

Issue: #6712. 시작 HEAD: `02f7153b9`.

## 분석과 계획

- 한국어 가정통신문 2쪽의 마지막 문단에 붙은 묶음 도형이 렌더에 없다.
  그림은 본문 높이에 기여하지 않는 InFrontOfText이며, 분할 셀은 가시적인
  non-inline flow unit이 없으면 Shape를 건너뛴다.
- 높이 기여 여부와 그림의 페이지 소유는 다른 계약이다. 문단 시작 글줄을 소유한
  조각에는 앞/뒤 겹침 도형도 방출해야 한다. 다음 조각에서는 다시 방출하지 않는다.
- 원본 Group 내부의 BinData ID를 추출해 마지막 페이지의 누락/중복을 먼저 검사한다.
  수정은 일반 셀/partial 셀의 동일한 소유 규칙에 적용하고, 기존 TextBox의 이어지는
  내부 문단 렌더는 보존한다.
- focused, overlap/overflow/off-canvas, 기준 PDF 대조로 판단한다. 세로 간격 및
  전체 테두리는 별도 조건이며 이 단계 통과를 #6712 전체 해결로 해석하지 않는다.

중간 산출물은 `/tmp`에만 둔다. baseline 완화와 PR/push는 하지 않는다.

## 수정 전 검사

- 처음 작성한 테스트의 `Picture.bin_data_id` 필드 경로 오류를
  `Picture.image_attr.bin_data_id`로 수정했다. 최초 실행은 컴파일 실패(exit 101)이며
  기능 검증으로 세지 않는다.
- 정정 후 red: 1 test failed, 0.046초, exit 100. 한국어 마지막 페이지에서 원본 묶음의
  `bin 1`이 없고 실제 그림 ID는 `[7, 8, 9, 10, 11]`이었다.
- 높이를 예약하지 않는 앞/뒤 겹침 도형의 방출 조건에 첫 소스 글줄 소유를 반영한다.
  `start_line == 0 && end_line > start_line`인 경우만 허용하고, 일반 flow 도형이나
  TextBox의 기존 continuation 처리는 바꾸지 않는다.

## 수정 후 검증

- #6712, #5862, #5863, #2007 및 전체 overlap/overflow/off-canvas 파티션 대상:
  **83 passed, 997 filtered/skipped, 85.502초, exit 0**. baseline은 수정하지 않았다.
- 이 검사는 원본 묶음의 그림이 마지막 페이지 렌더 트리에 한 번씩 존재하는지 검증한다.
  상위 SVG clip을 통과해 raster에서도 보이는지는 별도 확인한다.
- CLI 재빌드: release-test `--bin rhwp`, 2분 29초, exit 0.
  binary SHA-256: `8daef59bd762a785983eea06532a52e9d3bd6a19a639284102d5567701872bc9`.
- Ubuntu/Chrome Studio webfont visual sweep: 한국어 1, 2쪽, PDF/SVG 모두 2쪽,
  exit 0. `/tmp/rhwp-6712-stage11-ko-sweep/issue6712-ko-stage11/review/`의 두 이미지를 열었다.
  2쪽에서 서울특별시/간호사회/영유아 로고가 다시 보인다. 다만 아래쪽 일부가 clip되고
  외부 셀 하단선이 사라져 있으므로 완전 복원으로 판정하지 않는다.
- 1쪽의 세로 간격 차이와 하단선 소실도 남아 있다. 이 단계에서는 원본 도형의
  페이지 소유/방출만 해결했으며 **#6712 전체는 미해결**이다.
- 로그와 중간 PNG/SVG/JSON은 커밋하지 않는다. 전체 회귀 및 필수 lint 묶음은
  전체 수정 완료 후 별도로 수행해야 한다.
