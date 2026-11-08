# 인쇄 비트맵 축소 C85 증거

Issue: #7445

## 원인과 보정

- 기준 base `fe5725dc384689b45d60ef8ea4c83138bc2701d8`, parent code `1c8ef6cb60a97a24d1ae4cbca3e06f4a73d4f314`.
- 로드맵153쪽의 원본1000×514 PNG를 약559×430 장치 픽셀로 축소할 때 기존 선형 필터와 fractional destination의 재보간이 얇은 회색 격자선을 약하게 만들었다.
- 공통 Rust query가 최종 축소 크기로 Lanczos3 필터링한 PNG와 장치 경계의 paint 사각형을 한 번 생산한다. Native Skia·WebCanvas·실제 CanvasKit가 그 결과를 소비한다. 레이아웃/원본 좌표는 바꾸지 않았다.
- Print/HighQuality, 양의 축 정렬 변환, 양축 축소, 불투명 비트맵만 적용한다. 실제 crop·회전/반전·확대·투명 그림·이미 장치 크기로 래스터한 SVG는 기존 경로다. 최대 디코드 픽셀과 메모 캐시의 byte/entry 상한을 유지한다.
- 이미지 색 관리/EXIF 메타데이터를 포함한 모든 가능한 이미지 형식의 전수 수용을 주장하지 않는다. 해당 자료의 별도 대조는 아직 없다.

## 현재 결과

| 범위 | Native | fresh WASM |
| --- | --- | --- |
| 로드맵302쪽 전쪽 최저 raw | 91.00093% | 91.00093% |
| 로드맵153쪽 raw | 82.83247% →91.00093% | 91.00093% |
| 생물독소205쪽 전쪽 최저 raw | 97.12915% | 97.13157% |
| uncropped image fill14쪽 최저 raw | 99.84430% | 99.64164% |
| 현재48조건 전쪽 비교 | 1745쪽 완료 | 1745쪽 완료 |

정본 raw는 색상 경계 보정 전2px 실루엣이다. 허용치나 마스크를 변경하지 않았다.
현재48조건은 모두 쪽수가 맞지만 sample16 HWP·동일 SHA의2010 alias·HWPX의18/35쪽이 미달이다.
Native 최저76.25384%, WASM 최저70.65059%여서 `re_review_required`이며 PR 준비 완료가 아니다.
Native 전48조건과 WASM 전48조건 TSV는 `tsv/`에 보존한다. 최초 WASM press12는 Chrome TargetCloseError로 중단됐고, 같은 source/build로 별도 재실행한 전12쪽 최저93.70388%/미달0을 사용한다.

`whole-reconciled-summary.json`에는 미달 쪽과 유효한 정확한 devel TSV 대비0.01%p 이상 감소도 남긴다. 이 탐색값 감소를 자동으로 조판 회귀 또는 허용 가능한 AA로 단정하지 않는다. 고정 devel의 쪽수 gate가 실패한16조건은 페이지를 잘라 수치를 만들지 않았다.

## 직접 판독과 소비 증거

- `native-p153-review.png`, `native-p153-overlay.png`, WASM 대응 파일을 직접 열어 점·회귀선·축/격자·대한민국 라벨·캡션의 보존을 확인했다. 미세한 AA/그림 압축 차이가 남으며98% 또는 엄격 픽셀 일치는 아니다.
- `image-region-native-wasm.json`: 원본 그림 소속 영역의 RGB가 Native/WASM 동일하다. 전체 페이지 글리프 일치 주장으로 넓히지 않는다.
- `actual-image-results.json`: 정상 Studio Canvas2D/Print로 문서를 열어302쪽과 profile을 확인한 후 그 실제 layer의 그림 op를 실제 `CanvasKitLayerRenderer` software surface로 소비했다. 해당 그림 영역240370픽셀은 Native와 차이0이다.
- 이는 **그림 소유자 하나의 실제 CanvasKit 소비 검사**다. 전체 문서 CanvasKit, 실제36조건, 별도 glyph 준비의 통과가 아니다. 숨겨진 Canvas2D overlay 없음·unsupported op0·완료/readiness를 기록했다.
- `actual-image-provenance.json`: 초기화 응답 JS/WASM의 실제 hash를 고정한 pkg와 대조했다. Vite JS URL 변환은 별도 근거로 구분했다.
- Mac fresh WASM은 locked wrapper의 web/no-opt 로컬 대체 빌드다. Docker 최적화 빌드 통과를 주장하지 않는다.

## 회귀

`tests/issue_2293_chart_png_text.rs::reduced_scatter_plot_preserves_every_interior_grid_band`는 기존 Native Skia CI 실행 대상에 추가했다. 원본 PNG에서 내부 회색 격자 행을 찾고, 같은 그림 영역에서 이어져 보이는 열의 비율을 검사한다. 98% 미만 페이지의 절대 px 좌표는 고정하지 않았다. 원본 외곽 축과 내부 반복 격자를 구분하며 source 쪽수/그림 소속을 유지한다.

`coverage-counterfactual.json`의 동일 입력/검사: 이전 renderer는 격자 행87의3/277열만 표시하여 실제 FAIL(exit101), 새 renderer는 PASS(exit0)다. 정식 Cargo nextest에서 기존 차트 텍스트 대조군과 새 내부 격자선 검사2건 모두 PASS(0.646초)를 확인했다. manifest와 unit-tier 정책도 PASS이며 source-side4103 tests/296 modules를 유지한다. 전체 Rust 회귀 통과를 뜻하지 않는다.
초기 명암 적분만 보는 진단은 이전의 흐려진 선도 통과하여 채택하지 않았다. 임시 source-side unit 배치는 현행 unit-tier 증가 금지에 맞춰 제거했고 정책을 상향하지 않았다.

## 잔여

WMF18/35쪽, devel 대비 감소의 직접 원인 판정, 새 code head 전체 nextest·lint/build·Native Skia3종·manifest/tiers·실제 소비 경로 검증이 남아 있다. 현재 수정으로 이슈 전체 또는 PR 완료를 주장하지 않는다.
