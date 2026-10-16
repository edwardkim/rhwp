# #7902 최종 직접 PNG 증적

## 고정한 입력과 실행 코드

- 검증 devel: `f5251e7de4e31ee0fb939f352c6ef144bb340bdb`. PR 준비 중 fetch에서도 동일했다.
- 후보 renderer source: `40ef22cbe4d137398149b296ff3af42c0af0b607`.
- Native CLI는 runtime 동일 source `3a6c7d0e0895fb5258f6cfd28ca4e9cef244e1b8`의 실제 빌드다. 후속 renderer 보완은 WASM 전용 child이며 Native depfile 소비0건이다.
- Native SHA-256: `30f68b405f71a9b95581cd9c16c0d06dc3685d229551f5082af17bee03a1d089`.
- fresh WASM SHA-256: `ec9b6c4d7f90f7caba9438725946daa5ee1613367e338f84651ea9b0368ae519`.
- 96dpi·Print profile·`pdftoppm -thinlinemode shape`·2px 이웃 관용 거리·threshold32·흰 배경 alpha 합성. 원본/Print를 이동하거나 맞추지 않았다.
- 전체 Native835쪽·새 WASM835쪽을 실제 출력했다. 아래 PNG는 해당 전쪽 결과에서 `make_compares`/`make_overlay_page`/`make_review_panels`로 합성했으며 SVG를 생성하지 않았다.
- 원본19개와 Print19개 모두 HEAD Git blob과 SHA-256이 같다. 기존 독립 한컴 Print를 재사용했으며 대상 문서의 정답 PDF를 글꼴 공급 재료로 사용하지 않았다.
- 원본 HFT는 [별도의 전체 한글 표본](../../../manual/verification/hft_original_font_supply.md)으로 준비했다. 글꼴 bytes는 공개 증적에 포함하지 않는다.

## 전쪽 결과

모든 쪽90% 이상·쪽수 일치·누락0이다. 고정 devel 대비0.01%p 이상 하락0쪽, 최대 하락0.00846%p다. 숫자는 내용 실루엣 보조값이며 전체 픽셀 동일이나 모든 모양의 완벽한 일치를 뜻하지 않는다.

| key | 쪽수 | Native 최저 | fresh WASM 최저 |
| --- | ---: | ---: | ---: |
| `6761-later` | 104 | 91.39728% | 90.20061% |
| `6761-shifted` | 103 | 91.39728% | 90.20061% |
| `6761-control` | 103 | 91.39728% | 90.20061% |
| `chemical-original` | 103 | 91.39728% | 90.20061% |
| `7336-nested` | 6 | 96.33368% | 96.39821% |
| `6778-square` | 12 | 98.65081% | 99.12658% |
| `7336-stored` | 7 | 96.61646% | 96.83128% |
| `scattered-hwp` | 52 | 91.65664% | 91.93501% |
| `rowbreak-hwp` | 18 | 92.50158% | 92.80959% |
| `rowbreak-hwpx` | 18 | 92.50594% | 92.74222% |
| `cellimg-hwp` | 8 | 95.64519% | 96.39187% |
| `overfill-hwpx` | 48 | 98.71784% | 99.10705% |
| `vpos-hwp` | 5 | 98.88461% | 99.50987% |
| `cellimg-hwpx` | 8 | 95.64519% | 96.39187% |
| `pictoggle-hwp` | 1 | 99.93139% | 99.93088% |
| `vpos-hwpx` | 5 | 98.88461% | 99.51014% |
| `nested-hwp` | 115 | 97.20064% | 97.16568% |
| `nested-hwpx` | 115 | 97.20064% | 97.16568% |
| `5543-carried-anchor` | 4 | 95.12191% | 95.40382% |

## 대표 PNG와 직접 판독

review는 왼쪽 RHWP·가운데 독립 PDF·오른쪽 overlay다. overlay는 파랑 RHWP만·빨강 PDF만·주황 양쪽 차이·회색 일치다. 같은 쪽의 Native/fresh WASM 모두 직접 열었다.

| 원본 / 쪽 | Native review / overlay | fresh WASM review / overlay | 직접 확인한 범위 |
| --- | --- | --- | --- |
| `chemical-original` / 19 | [native review](native-chemical-original-review-019.png) / [overlay](native-chemical-original-overlay-019.png) | [wasm review](wasm-chemical-original-review-019.png) / [overlay](wasm-chemical-original-overlay-019.png) | 전체 최저 쪽의 목차·점선·번호·내용 순서. 글꼴 굵기/획 차이는 남는다. |
| `chemical-original` / 29 | [native review](native-chemical-original-review-029.png) / [overlay](native-chemical-original-overlay-029.png) | [wasm review](wasm-chemical-original-review-029.png) / [overlay](wasm-chemical-original-overlay-029.png) | 본문 없는 쪽의 생성 번호 -7- 내용·굵게·원점. 엄격 픽셀 동일은 아니다. |
| `rowbreak-hwpx` / 15 | [native review](native-rowbreak-hwpx-review-015.png) / [overlay](native-rowbreak-hwpx-overlay-015.png) | [wasm review](wasm-rowbreak-hwpx-review-015.png) / [overlay](wasm-rowbreak-hwpx-overlay-015.png) | 그림→세 줄 본문→하이픈 본문→후속 표 순서와 누락/겹침. 그림 색·바깥 점선 아래 변·일부 글자/간격 차이는 남는다. |
| `overfill-hwpx` / 22 | [native review](native-overfill-hwpx-review-022.png) / [overlay](native-overfill-hwpx-overlay-022.png) | [wasm review](wasm-overfill-hwpx-review-022.png) / [overlay](wasm-overfill-hwpx-overlay-022.png) | 문서 최저 쪽의 이어받은 본문·소제목·주석·쪽번호 및 내용 보존. |
| `overfill-hwpx` / 48 | [native review](native-overfill-hwpx-review-048.png) / [overlay](native-overfill-hwpx-overlay-048.png) | [wasm review](wasm-overfill-hwpx-review-048.png) / [overlay](wasm-overfill-hwpx-overlay-048.png) | 마지막 본문·주석·48번 쪽번호. 불필요한49쪽 없이 끝나며 마지막 글이 보존된다. |
| `nested-hwpx` / 104 | [native review](native-nested-hwpx-review-104.png) / [overlay](native-nested-hwpx-overlay-104.png) | [wasm review](wasm-nested-hwpx-review-104.png) / [overlay](wasm-nested-hwpx-overlay-104.png) | 이어받기 표→제목/본문→그림→후속 본문. HWP도 같은 실제 raster/PDF 좌표이며 양식별 검사로 보호한다. |

기존102쪽 하단 표 높이 차이는 base와 현재의 해당 영역 pixels·bbox가 같다. 이번 변경으로 해결했다고 주장하거나 그 쪽에 새 px 핀을 만들지 않았다. 남은 초기 출력 차이와 현재 변경이 만든 회귀를 구분한다.

완료 페이지의 선정·독립 좌표·±2px와 관계 검사는 [3단계](../../../working/task_m100_7902_stage3.md#문서별-완료-페이지의-px-회귀), HWP/HWPX104쪽 추가 보호와 실제 수정 전 FAIL/후 PASS는 [11단계](../../../working/task_m100_7902_stage11.md)에 연결한다.

## 원시 기록

ignored `output/pr-review/issue7902-render-repair-20261014/`:

- `stage11-final-develop-comparison.json`: Decimal 기준835쪽×2의 base/head 비교.
- `stage11-final-assets.json`: 실제 원본/Print/PNG SHA, DPI, producer와 합성 metrics.
- `stage11-input-commit-audit.json`: 원본/Print38개 실제 HEAD blob 일치.
- `stage11-rerender-pixel-identity.json`: 수정 WASM835 PNG/19 TSV의 실제 재출력과 이전 raster 동일성.
- `stage11-hwp-hwpx-104-reading-identity.json`: HWP/HWPX 실제 raster와 독립 PDF 좌표 동일성.

[전체 기준/후보 Native/fresh WASM TSV 원본76개](https://gist.github.com/jangster77/c62ec80625417cef370c6258132baee2)를 제출했다. 게시 뒤 API에서77개 파일(README 포함)의 bytes를 모두 재확인했다. 이 디렉터리에는 TSV·실행 로그·중간 JSON·비공개 글꼴을 커밋하지 않는다.
