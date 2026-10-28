# 최종 결과보고서 — Task M100 #1917

## 이슈

[#1917 HWPX BinData 64MB 엔트리 상한 — 대형 이미지 로드 거부 + 왕복 시 pic 컨트롤 소실 (10k 서베이 4건)](https://github.com/edwardkim/rhwp/issues/1917)

## 요약

`MAX_BINDATA_SIZE`(zip-bomb 방어)의 64MB 가 실문서를 거부했다 — 정부 보도자료
계열의 비압축 BMP/TIF 대형 이미지(최대 103.7MB, 한글 정상 열람). 로드 거부는
그림 소실에 그치지 않고 재직렬화에서 pic 컨트롤 드롭(왕복 데이터 손실,
IR_DIFF 하드 실패)으로 증폭됐다. 상한을 512MB 로 상향 (무제한 팽창 차단이라는
방어 목적 유지).

## 검증

- 인메모리 핀 (`tests/issue_1917.rs`): 압축 해제 70MB BinData(종전 상한 초과)의
  HWPX 왕복 보존 + pic 컨트롤 유지 — 수정 전 로드 거부/pic 드롭, 수정 후 PASS.
- **서베이 4건 전수 재검: IR_DIFF → PASS 4/4** (diff 0, r2 0 — bmp 2, tif 1, jpg 1).
- 타깃 게이트: hwpx/hwp5_roundtrip_baseline PASS. 풀 스위트는 PR CI 담당.

## 남는 축 (후속 판단)

512MB 초과 실문서는 서베이 10k 에 부재. 그 너머의 견고성(상한 초과 시에도
pic 컨트롤·binaryItemIDRef 보존 + 원본 ZIP 엔트리 pass-through)은 실수요
확인 시 별도 타스크로.

## 산출물

- 수정: src/parser/hwpx/reader.rs (MAX_BINDATA_SIZE 64→512MB)
- 테스트: tests/issue_1917.rs
- 문서: plans/task_m100_1917.md, 본 보고서 (#1916 과 배치 PR)
