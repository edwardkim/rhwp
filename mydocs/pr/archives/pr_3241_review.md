# PR #3241 검토 — imgDim 없는 그림 crop 폴백을 적응식으로 복원

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3241](https://github.com/edwardkim/rhwp/pull/3241) |
| 작성자 | [@planet6897](https://github.com/planet6897) |
| 관련 이슈 | [#3239](https://github.com/edwardkim/rhwp/issues/3239) |
| base / 원 PR head | `devel` / `3847a548aa43cfa232a1c6dce0378464731b96bf` |
| 문서 작성 시점 참고값 | 원 PR은 `BEHIND`. 최종 merge 전 최신 head·merge 가능 상태·required checks 재확인이 필요하다. |
| 통합 검토 적용 | 최신 `upstream/devel` `973de548faedc6709ef862a1a12aa7146c225ac5` 위 `6ce59e4a537b131917a57c8919a918215b88177e` |

## 관련 이슈와 변경 범위

- #3239은 imgDim을 보존하지 않는 구형 HWP5의 비-96dpi 스캔 그림에서, #2990 이후 고정된
  75 HU/px 폴백 때문에 원본 그림이 확대되고 우·하단이 절단되는 r22 렌더 회귀다.
- `compute_image_crop_src`의 폴백 순서를 imgDim → crop `right`/`bottom`과 디코딩
  크기를 대응시키는 적응식 → 둘 다 무효일 때만 75 HU/px로 정리한다. imgDim이 있는
  #2990의 원래 보호 경로는 바꾸지 않는다.
- 회귀 단위 테스트 2건을 새 의미론에 맞추고, #3239의 200dpi TIFF 실측값과 최후 75 HU/px
  폴백을 각각 고정한다. 재현 HWP·r19 레퍼런스 PNG와 `tools/verify_issue3239.py`를
  함께 추가한다.

## 렌더 영향과 visual sweep

- 그림 source crop과 SVG·skia·web canvas 공통 헬퍼를 바꾸므로 시각 검증 필수 대상이다.
- 기준 원본은 `samples/issue3239/evaluation_form_200dpi_scan.hwp`
  (SHA-256 `34e5ca1df74196c2ddcfa3b24cbda9cdb0099441dfc7e066d15a780d52668367`)다.
  HWP 2020 변환으로 새로 생성한 기준 PDF는
  `pdf/issue3239/evaluation_form_200dpi_scan-2020.pdf`
  (SHA-256 `c4d53bd1b29f2148b6c300e0b9c5025757dbb82f3ce2f34a47fbf6e97c5ca952`,
  1쪽)이며 변환 결과는 `status: success`, `run_status: 0`, `validation: ok`다.
- 144dpi sweep은 PDF 1쪽과 rhwp 산출물 1개를 단일 페이지 fallback으로 1:1 대응해
  자동 후보 **0/1**을 보고했다. rhwp 산출물의 내부 페이지 표기는 `p200`이다.
  다만 SVG export 경로는 이 HWP의 TIFF 그림을 rasterize하지 않아
  `review/review_200.png`의 그림 영역이 비었다. 따라서 해당 sweep의
  pixel match **96.94602%**와 visual accuracy proxy **2.66955%**는 최종 시각 판정
  근거로 사용하지 않는다.
- 최종 검토 자산은
  `mydocs/pr/assets/pr_3241_planet6897_issue3239_p001_review.png`다.
  Windows native-skia에서 실제 `export-png`로 생성한 현재 결과와 r19 레퍼런스 PNG,
  그리고 R=레퍼런스·GB=현재 결과 overlay를 나란히 보관한다. 두 원본은 모두 794×1123이고
  SHA-256 `992895b5952a1c6cdd9ef14d33ceef3b25b474f19372311405ca8399262b68fa`로
  정확히 일치한다. 따라서 `diff>40` 픽셀 비율도 **0.00%**다.

## 사전 검증

- macOS 통합 검토 브랜치에서
  `CARGO_INCREMENTAL=0 cargo test test_compute_image_crop_src --lib --quiet`를 실행해
  **8 passed, 0 failed**를 확인했다.
- Windows 검증 호스트 `win10-ted`의 PowerShell에서 최신 통합 검토 스냅샷으로
  `CARGO_INCREMENTAL=0 cargo build --features native-skia`를 성공했다
  (dev profile, 5분 28초).
- 같은 Windows 환경에서
  `python tools/verify_issue3239.py --exe target-local/debug/rhwp.exe`를 실행해
  **`OK: diff>40 픽셀 비율 0.00% (임계 2.0%)`**를 확인했다. 이는 이슈의
  r19 레퍼런스 PNG와 현재 native-skia 출력이 재현 기준에서 일치함을 뜻한다.

## 재현성 리스크와 판단

- `verify_issue3239.py`는 `numpy`와 `Pillow`를 사용하지만 프로젝트 의존성으로
  명시하지 않으며, 기본 exe 경로도 `target/debug/rhwp.exe`로 고정한다. Windows 검증
  checkout은 `.cargo/config.toml`의 `target-dir = "target-local"` 때문에
  `--exe target-local/debug/rhwp.exe`를 명시해야 했다. 이는 수정의 정확성과는
  별개인 재현성 문서화 보완 과제다.
- 핵심 폴백은 crop 전체 범위와 실제 디코딩 크기를 비교하므로, 단순 75 HU/px 단정 대신
  비-96dpi 구형 그림을 복원한다. imgDim이 있거나 crop 범위가 무효인 기존 경로는 개별
  테스트로 보호한다.

## 최종 권고

- 최신 `devel` 위 통합 검토 스냅샷에서 renderer 회귀가 재현되지 않고, Windows
  native-skia 출력이 r19 레퍼런스와 바이트 단위로 일치하므로 수용 후보로 권고한다.
  HWP 2020 PDF sweep은 TIFF SVG rasterization 공백을 드러낸 보조 점검으로만 기록한다.
- 최종 merge 조건은 통합 PR 최신 head의 GitHub Actions 통과, 원 PR의 `BEHIND` 해소 또는
  동일 변경의 최신 통합 경로 확인, 작업지시자 승인이다.

## Merge 완료 및 후속 처리

- [#3256](https://github.com/edwardkim/rhwp/pull/3256)이 2026-07-24
  `e05a5a8bdcf636844e80744d91fff706c1c28ce7`로 `devel`에 merge 됐다.
- 통합 PR의 CI(Build & Test·Native Skia·기본 기능 테스트 8 shard), CodeQL, Render Diff가 모두 성공했다.
- [#3239](https://github.com/edwardkim/rhwp/issues/3239)는 closing keyword 반영을 10초 간격으로 세 번
  확인한 뒤에도 `OPEN`이어서, merge commit·CI·macOS/Windows 검증·시각 자산 링크를 남기고 수동 close했다.
- [#3241](https://github.com/edwardkim/rhwp/pull/3241)는 통합 완료 안내와 감사 코멘트 후 supersede close했다.
