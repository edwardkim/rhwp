# PR #9471 검토

[seo-rii 님의 원 PR](https://github.com/edwardkim/rhwp/pull/9471)

## 검토 버전

- 원 PR head `2727d0e26b7608c83823a48e6d6eee0f2d0348d0`
- 기준 devel `77e4c3d1369623818e8cf3213e3fdc21d8e6502e`
- 통합 코드 후보 `b25bbad4f477a5f6c9bfc212f58b6f03e2a090dc`
- 검토 브랜치 `review/integration-8678-chart-layout-native-editing`

| 원본 커밋 | 체리픽 커밋 |
| --- | --- |
| `2727d0e26b7608c83823a48e6d6eee0f2d0348d0` | `b25bbad4f477a5f6c9bfc212f58b6f03e2a090dc` |

## 구현과 검증 범위

#9470을 적용한 뒤 현재 문서 revision을 인쇄하는 `exportPdf`를 반영했습니다. 문서 ID·revision·글꼴 세대를 확인하고, 32쪽·32 MiB의 사용자 지정 범위 안에서 Print PDF를 완성합니다. 지정한 출력 바이트를 요청 수락 전에 예약하며, 완성한 결과를 호스트 소유 버퍼로 전달합니다. Skia를 빼고 빌드한 경우에는 기능 조회와 유효한 요청의 `featureDisabled` 응답을 확인합니다.

기존 파일 기반 Direct PDF와 새 worker가 같은 PDF 작성 경로를 사용합니다. 그 경로에서 기존 Print 출력의 픽셀과 링크·책갈피 계약을 확인합니다. 미저장 편집의 신청인·자녀 정보, 원본 해시, 편집 revision, dirty와 실행 취소 상태를 실제 공개 신청서로 확인합니다.

[공개 휴가 신청서](../../../samples/issue5599_oracle/3191107_leave_request_form.hwpx) · [행정업무운영 편람 HWPX](../../../samples/2025%20행정업무운영%20편람%28최종%29.hwpx) · [Native worker PDF 계약](../../../bindings/Native/tests/native_worker_pdf_contract.py) · [Rust 회귀](../../../tests/cases/issue_8701_native_live_pdf.rs) · [Native API 문서](../../../bindings/README.md)

## 회귀 실행 확인

최신 후보의 Skia 포함 macOS C ABI에서 PDF 요청의 쪽수·바이트 한도, 이전 revision, 글꼴 세대, 예약 바이트의 회수, 완성된 PDF 버퍼의 단일 소비, 문서 상태 보존을 검증했습니다.

## 실제 신청서의 Print PDF

공개 신청서의 신청인 쪽 `생년월일`을 `신청일자`로 바꾼 뒤 저장하기 전에 첫 쪽을 출력했습니다. PDF의 바뀐 픽셀은 신청인 항목 내부에만 있었고, 자녀 쪽 `생년월일`과 나머지 인쇄 결과는 유지됐습니다. 원본 파일 SHA-256, 출력 직전의 dirty·실행 취소 상태, 실행 취소 후 선택 복원도 확인했습니다.

Mac의 일반 CLI 설정은 누락 글꼴을 Noto 웹폰트로 자동 보완해 worker와 다른 프로그램을 선택했습니다. CLI의 글꼴 환경을 명시해 자동 보완을 제외한 동일 시스템 글꼴 조건에서, 편집 전 worker PDF와 #9471 적용 전후의 기존 Direct PDF는 96 DPI에서 794×1122 RGB 픽셀이 전부 일치했습니다.

## 편람의 다중 쪽 출력

실제 383쪽 편람에서 256·257쪽을 한 PDF로 출력한 뒤 각 쪽의 개별 출력과 대조했습니다. 두 쪽 모두 96 DPI의 740×1006 RGB 픽셀이 각 개별 결과와 같았습니다. 383쪽 단독은 본문이 포함된 한 쪽으로 출력됐고, 383쪽부터 두 쪽을 요구하면 `invalidArgument`와 함께 버퍼 없이 종료했습니다. 원본 해시와 worker의 revision·글꼴 세대·문서 상태는 유지됐습니다.

Skia를 포함하지 않고 빌드한 실제 macOS C ABI에서도 PDF 기능이 꺼져 있음을 알리는 응답과 버퍼 미생성을 검증했습니다. 포함 빌드와 미포함 빌드는 서로 다른 SHA-256으로 고정했습니다.

## PDF 링크와 책갈피

공개 Textmail HWP의 1쪽과 LH 안내 HWPX의 8쪽에서 외부 URL 주석의 대상과 영역이 기존 Direct PDF와 일치했습니다. 2쪽짜리 재현 HWPX에서는 첫 쪽의 `#` 형식 내부 링크가 2쪽 책갈피로 이어졌고, 링크·목적지 주석은 #9471 적용 전후의 Direct PDF와 같았습니다. 같은 재현의 `?` 형식 링크는 두 Direct PDF와 worker 모두에서 URI 주석으로 기록됐습니다.

## Merge 후 contributor PR comment 계획

Skia 포함·미포함 worker 계약, 실제 신청서의 미저장 PDF와 기존 Direct PDF 대조, 편람 256·257·383쪽 선택 출력을 안내합니다. 시스템 글꼴 조건과 실제 출력의 픽셀 근거를 기록합니다.

통합 PR의 병합 SHA와 최신 head의 실제 CI를 고정합니다. 연결할 증적은 병합 SHA의 raw PNG와 대조하고, 한국어 존댓말로 원 PR에 검토와 반영 결과를 남깁니다.
