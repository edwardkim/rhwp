# Stage84: Print payload와 원본 조회를 구분하는 기존 검사 교정

Issue: #7445. 부모 HEAD `4bd20fd54ac25d8054b518d2e1bc45943b1726d0`, 고정 base
`d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`의 후속 검증이다. 제품 렌더링 코드는 바꾸지 않았다.
HWP 보정 대상·독립 PDF·90%·2px·devel 대비 2%p 기준도 그대로다.

## 실패의 선행 분석과 독립 검증

기존 구현·검사의 `51891703c9` → `482933b476` → `d8dc0609b7` 및 차트의 `2714b18e5d`를
확인했다. Stage82의 공통 Print JPEG 전달과 Stage83의 종이 격자 소비 뒤 전체 nextest는
11751건 중 11747 PASS / 4 FAIL / 45 skip, 1294.649초였다. 다음 단계는 실패한 기존 검사의
원본을 독립 한컴 Print와 Native/fresh WASM으로 먼저 확인한 뒤 기대 계약을 바로잡은 것이다.

| 기존 입력 | 고정 base Native / WASM | 후보 Native / fresh WASM |
| --- | ---: | ---: |
| bitmap-crop-palette-controls | 99.98227 / 99.98255 | 99.97923 / 99.97922 |
| bitmap-white-palette-controls | 99.75947 / 99.75938 | 100 / 100 |
| bitmap-indexed-color-controls | 99.74919 / 99.74906 | 100 / 100 |

모두 같은 입력·기준 PDF의 전 1쪽이며, 96DPI·PDFium152.0.7947.0 smooth RGB v1로 비교했다.
색상 보정 전 raw 값이다. Native/fresh WASM review와 overlay를 직접 판독해 crop의 다섯 그림,
예약색의 여섯 그림, indexed의 열두 그림과 녹색·흑색 patch가 보존됨을 확인했다.
crop 경계에는 작은 래스터 차이가 남으며 이를 완전 일치로 보고하지 않는다.
이 입력에는 비어 있지 않은 HWPX 텍스트가 없다. 구형 base WASM의 환경 설정 API 차이는
양쪽 WASM에 명시적 portable-none 환경을 공급해 동일하게 비교했고 글꼴 guard를 우회하지 않았다.

## 검사 계약

- 배치별 `OutputDevice`·`SourceWindow` payload는 crop·출력 폭·JPEG 품질에 따라 달라지므로
  전역 원본 BinData 키로 조회하는 바이트와 같다고 가정하지 않는다. `src/paint/json.rs`의 기존
  계약대로 재조회할 키가 없으면 `omit_image_bytes`에서도 inline 바이트를 보존해야 한다.
  기존 키가 있는 경로의 조회·생략 검사는 유지했다.
- 원본 색·해상도·알파·문단 소유·그림 순서·중복 금지를 유지한다. 원본 조회는 원본 모델의
  픽셀과 정확히 같아야 한다. Screen도 원본 색을 보존해야 한다.
- Print는 원본 RGB 배열 대신 같은 입력의 독립 한컴2024 Print PDF에서 추출한 JPEG 18개와
  디코딩 후 전체 RGBA를 정확히 대조한다. RHWP 출력에서 기준 이미지를 만들지 않았다.
  파일·입력·PDF SHA는 `tests/fixtures/print_bitmap_payload_oracles/provenance.json`에 기록했다.
- 이전 단색 128→131·240→246 검사는 JPEG 경계의 색 번짐까지 단색이라고 가정했다.
  실제 독립 PDF도 경계에 `[226,255,237]`을 가지므로 ±1 검사를 완화하는 대신 독립 JPEG의
  전체 픽셀과 정확히 같도록 검사한다. 원본 계조 표본의 존재와 프로필 왕복·캐시 신원은 유지했다.
- 차트의 원본 7개 격자선은 `image.data`에서 찾는다. 압축된 Print payload에서 원본 줄 수를
  세지 않는다. 실제 paint에는 resolved를 포함한 op 전체를 그대로 사용하며, 기존 5개 내부
  격자의 독립 PDF 대비 검사는 그대로 유지한다. 원본 행은 15/87/159/231/303/375/448이다.
  로드맵 153쪽은 Native/fresh WASM 모두 raw 100%이며 전 302쪽도 각각 최저
  94.97284% / 94.80964%, 누락·쪽수 불일치 없이 산출됐다.

집중 nextest는 crop 2 PASS, reserved/indexed 2 PASS, 차트 2 PASS, 총 6 PASS / 0 FAIL이다.
중간 SHA digest 출력 컴파일 오류와 suite 재생성 누락에 따른 0건 실행은 최종 통과가 아니다.
각각 수정하고 `--prepare` 후 실제 실행 건수를 확인했다. 전체 nextest와 잔여 게이트는 새 head로
다시 실행해야 하며, 이 단계의 6건 결과로 전체 통과를 대신하지 않는다.

## ubuntu-ted2의 독립 도구 lock 정합

원격 `task/8160-sample16-raw100`의 `5f7e7e7f00827a2cf3fc5dffbc4e8be89166b51e`를
읽기 전용으로 확인했다. 삭제한 독립 테스트는 Stage80에서 이미 같은 방식으로 제거했고 15개
assertion은 기존 관계 검사에 통합했다. 원격 브랜치 전체를 가져오지는 않았다.

독립 `tools/skia-surface-probe/Cargo.lock`의 JPEG fork 연결을 반영한 뒤, 이 브랜치에만 있는
root path dependency의 read-fonts·skera graph도 함께 정합했다. root Cargo.lock과 manifest는
바꾸지 않았다. 의도한 metadata 정합 후 `cargo metadata --locked --format-version 1
--manifest-path tools/skia-surface-probe/Cargo.toml`은 exit 0이다. 이는 lock의 정합 확인이며
Mac에서 Vulkan probe를 빌드·실행했다고 주장하지 않는다.

안정 TSV·대표 review/overlay·검사 요약은 assets의 `bitmap-regression-contract/`에 보존한다.
실행 로그·중간 실패·원본 JPEG 추출·source 행 진단은 ignored output의
`bitmap-regression-qualification/`, `chart-source-contract-diagnostic/`,
`ubuntu-ted2-8160-assessment/`에 보존했다. 전쪽 미달과 실제 소비 확인은 계속 진행 중이며 PR 준비 완료가 아니다.
