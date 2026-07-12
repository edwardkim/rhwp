# 최종 결과보고서 — Task M100 #1929

## 이슈

[#1929 HWP5 저장: raw 미보존 그림의 imgDim (0,0) 소실 — 왕복 그림 크기 손실 (10k 서베이 잔여 2건, #1916 계열)](https://github.com/edwardkim/rhwp/issues/1929)

## 요약

HWP5 PICTURE 레코드 extra 꼬리의 원본 이미지 크기 칸(18바이트 레이아웃의
offset 9..17)이 IR `img_dim`(HWPX `hp:imgDim` 대응)과 **양방향 모두 미연결**
이었다: 파서는 raw 보존만 하고 읽지 않았고, 직렬화기 non-raw 경로는 crop
폴백만 기록했다. 양측을 연결해 raw 미보존 그림(HWPX 파스 IR·편집기 신설
그림)의 imgDim 이 HWP5 왕복에서 보존된다.

## 수정

- `parser/control/shape.rs` `parse_picture`: extra ≥17 이면 offset 9..17 →
  `pic.img_dim` 적재. native HWP5 는 raw 경로 verbatim 재기록이라 2-round 안정.
- `serializer/control.rs` `serialize_picture_data`: non-raw 경로에서
  `img_dim != (0,0)` 우선 기록, (0,0) 은 종전 crop 폴백 유지 (동작 불변).
- 두 호출처(최상위 pic + 그룹 자식 pic) 공통 적용 — 이슈의 본문 직속/표 셀
  내부/글상자 내부 발현 전부 커버.

## 검증

- 인메모리 핀 2건 (`tests/issue_1929.rs`): (117780,35760) 왕복 + 2-round 보존,
  (0,0)+crop(4321,1234) 폴백 불변.
- **서베이 잔여 2건 실파일 (ird_h5_resid)**: 전체 pic 3개 img_dim 왕복
  **3/3 보존** — 이슈 expected 값 (117780,35760)·(149340,32640) 정확 일치.
- hwp5/hwpx roundtrip baseline + issue_1893 핀 PASS.
- img_dim 은 renderer 미사용 필드(HWPX 직렬화·roundtrip 게이트 전용)라 코퍼스
  렌더 A/B 불필요. 풀 스위트는 PR CI 위임.

## 부수 효과

HWP5→HWPX 변환(export-hwpx)의 `hp:imgDim` 이 0 대신 실값으로 방출 —
한컴 원본과의 구조 정합 개선.

## 산출물

- 수정: src/parser/control/shape.rs, src/serializer/control.rs
- 테스트: tests/issue_1929.rs
- 문서: plans/task_m100_1929.md, 본 보고서
