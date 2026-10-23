# 중첩 컷의 물리 높이와 원자 TAC 호스트 종료 간격

## 동일 입력과 독립 기준

기존 67개 입력 목록의 40번을 검증했다. 새 HWP 보정 대상을 추가하지 않았다.
입력 확장자는 HWPX이지만 실제 바이트는 한컴 2018 HWP5이며 compat 2022를 적용했다.

- `samples/issue6776/78494-virtual-convergence-industry-decree.hwpx`: SHA-256 `5090e9d20ce75814febeb4b79e106ea6abfd2546e511f229900d137d42623617`
- `pdf/issue7642-legal-table-origin/78494-virtual-convergence-industry-decree-2020.pdf`: SHA-256 `674ef63b51adc926c7c89f530bd142a45bb64cb655a7117bc20285d7a6e5a8d2`

96 DPI, PDFium 153.0.7999.0, `pdfium_smooth_rgb_v1`, webfont SVG rasterizer,
2px 관용 raw 90% 기준으로 각 backend의 전 74쪽을 비교했다.
`before`는 고정 devel이 아니라 직전 후보 Stage63이다. 고정 devel 대비 전쪽 검증을 대신하지 않는다.

## 생산 출처

- 부모 커밋: `1cd5b41b85d6cc423e630db36ca7b1ce557e7e7f`
- `table_layout.rs` SHA-256: `f75e9d4ec1ed9c5f665e48da4961cc00c3003d530f25cdbac317ec1ead4c5987`
- Native SHA-256: `d377e430f6e23604ab371e8fdd383fc5312201609a95fa831ccfbeb2c7320a95`
- fresh WASM은 locked web/pkg/no-opt 로컬 대체 빌드다. Docker 최적화 통과가 아니다.
- JS SHA-256: `aea5f6639676442e0628b640b75d0f70eb300481421112d23b661a082f695a68`
- WASM SHA-256: `a9be11d9a3ecc81c541aee1113e732f122fccbb26ec1b52ed64c4701b162d080`
- root pkg/Studio public/frozen package hash 일치. 실제 브라우저 초기화 응답 검증은 남았다.

## 전쪽 및 직접 판독

- native: 전 74쪽, 보정 후 최저 95.21322%(19쪽), 90% 미만 0쪽.
- wasm: 전 74쪽, 보정 후 최저 95.21405%(19쪽), 90% 미만 0쪽.
- 21쪽은 Native/WASM 모두 100%다. 원자 표 뒤 간격과 후속 제목의 기준선이 복원됐다.
- 37쪽은 Native 99.98294%, WASM 99.98291%다. 종료 행과 후속 행의 소유가 복원됐다.
- 36쪽은 Native 97.21908%, WASM 97.21106%다. `시행령`이 앞쪽에 남고 다음 쪽에 `제4조, [별표2]`만 이어진다.
- 보정 후 18/19/20/21/36/37쪽과 보정 전 21/37쪽의 review와 overlay를 두 backend에서 직접 판독했다.
- 각 backend의 전후 PNG 68쪽은 byte 동일하고 변경된 6쪽도 raw 하락이 없다.
- 1,686개 텍스트 소유 경로의 내용·순서·누락·중복을 전쪽 대조했다. 이미지·수식·차트·OLE 소유 수가 같다.
  페이지 소속 변화는 위 `시행령` 문단이 36쪽에서 닫히는 것뿐이다.
- 19쪽의 작은 잉크·테두리 차이와 36쪽 하단의 미세 테두리 차이가 남는다. RGB 완전 동일성을 주장하지 않는다.

## 회귀와 잔여

기존 기대값을 유지했다. 새 두 검사는 동일한 원자 표 호스트의 원본 120% 줄간격 소비 관계와
독립 PDF 21쪽의 text matrix 원점을 ±2 CSS px로 검사한다. 전 74쪽 양쪽 90% 이상 및
해당 21쪽 양쪽 98% 이상을 확인한 뒤 등록했다. 보정 전 동일 계약 2 FAIL → 보정 후 2 PASS다.
기존 6776 모듈은 최종 파일로 5 PASS다.

집중 검사는 66건 중 56 PASS / 10 FAIL이다. 최신 전체 nextest, 나머지 실패 보정,
필수 전체 게이트·전체 제출 범위 전쪽 비교·실제 소비 경로는 남았다. 전체 PR 준비 완료가 아니다.
상세 보고서는 `mydocs/working/task_m100_7445_rebase_repair_stage64.md`다.
원본 TSV를 byte 그대로 보존했다. 로그·진단·중간 JSON·생산 hash manifest는 ignored
`output/pr-review/7445-completed-20261009/rebase-8155-20261022/stage64-recursive-opening-reservation/v4/`에 있다.
