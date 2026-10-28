# 로드맵 격자선 검사의 PDFium 기준 정정

Issue: #7445. 제품 구현이나 기준 PDF를 바꾸지 않고 기존 검사 기대값의 래스터 출처를 정정했다.

- 원본: `samples/issue5941/1490000-201600081_roadmap_research.hwp`.
- 원본 SHA-256: `a06f46ec3f175c7cfa84eb3178b8b3fbdf78e94f71b31d7d87f3417a2617dae9`.
- 독립 한컴 2020 Print: `pdf/issue7382-regression-review/1490000-201600081_roadmap_research-2020.pdf`.
- PDF SHA-256: `133f79cad7b756ebef777d253befb14f9407faaf7fe04eeea66036f3e049bbe7`.
- 출력 code head: `fb258e6a6d97e001b180c93e005139e73df6e85e`, base `def947c2149df5bc812d801e6e79be19a403b4d1`.
- PDFium 153.0.7999.0 / pypdfium2 5.13.0 / `pdfium_smooth_rgb_v1` / 96 dpi / Print / compat 2022 / raw 2px 조건이다.
- Native와 fresh WASM은 각 전 302쪽, 최저 92.63744%, 90% 미만 0쪽이다. 대표 153쪽의 두 review와 overlay를 직접 판독했다. 농도·점 가장자리 차이가 남고 RGB 완전 일치는 아니다.

## 독립 PDF의 내부 격자선 관측

원본의 회색 행을 통과하는 중앙 절반의 열을 검사한다. 각 열은 기존과 같은 세로 ±2CSSpx에서 RGB 최대 성분의 최솟값을 읽는다. 분위수는 정렬한 값의 `floor((n−1)×q)` 순서통계량이다.

| 원본 행 | PDF의 5/95분위수 | 이전 pdftoppm 범위 | 정정 범위 내 PDF 열 | 동일 범위 내 Native·WASM 열 |
| ---: | --- | --- | ---: | ---: |
| 87 | 223 / 224 | 215 / 218 | 263 / 276 | 각각 276 / 276 |
| 159 | 226 / 229 | 215 / 218 | 266 / 279 | 각각 279 / 279 |
| 231 | 233 / 235 | 215 / 221 | 265 / 276 | 각각 276 / 276 |
| 303 | 219 / 226 | 224 / 238 | 261 / 272 | 각각 272 / 272 |
| 375 | 230 / 235 | 215 / 222 | 266 / 276 | 각각 276 / 276 |

이전 범위를 현재 PDFium 기준 그림 자체에 적용해도 다섯 밴드가 모두 실패한다. 정정은 후보 픽셀에서 상한을 가져온 것이 아니라 같은 독립 PDF의 현재 정본 측정값을 사용한다. 열의 90% 이상 연속 표시와 중앙값의 하한 검사는 유지한다. 네 번째 상한은 238에서 226으로 엄격해진다.

## 검증 신원과 범위

기존 전체 nextest는 11,750 PASS / 1 FAIL / 45 skip이며, 해당 실패는 첫 격자선의 1 / 276열 검출이었다. 밝기 정정만 적용한 중간 자동 suite의 차트 두 검사는 2 PASS, 1.732초였다. 원본 소유·포함·내용 순서를 함께 확인하는 최종 정식 차트 검사도 2 PASS, 1.779초다. 이 집중 통과를 전체 재실행의 통과로 대신하지 않는다.

원 해결 #2293의 글꼴 라벨 보호와 후속 `32a6f9df323d2884c61a3289747552ff620e8eb9`의 인쇄 비트맵 보호를 유지했다. 리베이스 후 ClipPath 순회 복구 `bfc746853888068c304642da2bbd801f190dfc79`도 유지한다.

로그·측정 스크립트·JSON은 ignored `output/pr-review/7445-completed-20261009/rebase-8155-20261022/rebase-def947-preservation/`의 `grid-band-*` 및 `roadmap-current/`에 보존한다.

## 위치 핀의 소유 관계 전환

현재 153쪽은 양 backend에서 98% 미만이므로 기존 ±2px 절대 네 변 핀을 재고정하지 않는다. 같은 쪽에 해당 원본 그림이 한 번 존재하며, 질의와 paint가 구역·문단·컨트롤 소유를 함께 유지하고 그림이 쪽 안에 포함되는지 확인한다. 그림 아래의 원문 출처, 캡션 첫 줄과 마지막 줄의 소속·앞뒤 순서도 보호한다. 초기 layout bbox 등식의 실패는 별도로 보존했다. Print가 `PagePaintContext::image_frame`에서 종이·600 dpi 장치 축으로 투영되는 실제 경로를 확인해 수치 등식을 요구하지 않았다. 이 관계 검사는 독립 PDF와의 전쪽 시각 비교를 대신하지 않는다.

## 제출 상태

제품 구현은 동일하다. 기존 C354의 devel 대비 하락, C356 진단 미달, 최신 전체 회귀·범위 게이트·기존 제출 범위의 전쪽 비교와 실제 소비 경로 검증은 별도로 남아 있다. PR 준비 전체 완료를 주장하지 않는다.

## 보존 파일 SHA-256

- `native-p153-overlay.png`: `daa9fc84d4265ec9ca03387659f0ccb2b3e8e7f1c0af628176879f56911921a6`.
- `native-p153-review.png`: `895df4a55590efa30558bcc51873e0690cde5d04f54965553a6c789a5736f8e6`.
- `native-roadmap.tsv`: `b1a39ccfc243266874beec7a1f95a45709892a8844409413f0c2e2a01d7fc3a6`.
- `wasm-p153-overlay.png`: `60a2f6b0f1360129fb104e63b83779202b65869d3e458a84abd3541179a0a6ad`.
- `wasm-p153-review.png`: `b946e87069450f47a51f403feb141ff26a823d47bbdffa92b0a118c2a3e828ed`.
- `wasm-roadmap.tsv`: `d9f7e6e1a51e72badf3ee52fd7a273bd4eb1b90c6b54ac845be1bc0896ace456`.
