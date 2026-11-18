---
kind: report
status: local-validation-complete
issue: 8863
last_verified: 2026-11-10
---

# #8863 삭제 시 기존 글자 모양 보존

## 현재 상태

실제 공개 HWP의 삭제·치환·전체 치환과 HWP/HWPX 저장 왕복 회귀는 통과했다.
기본 Native 전체 회귀도 13,138개 통과, 45개 skip으로 완료했다.
한컴에서 독립적으로 같은 편집을 수행한 Print PDF를 확보했으며, 수정 CLI 저장본을
한컴으로 인쇄한 결과와 전 19쪽의 텍스트 및 72dpi 래스터가 일치한다.

**구현·API·전체 회귀·lint·Native Skia 및 fresh WASM Canvas 전 19쪽 측정을 완료했다. 별도 base/head renderer 2%p 게이트는 미실행이며 PR 수용 완료를 뜻하지 않는다.**
아래 한컴 출력 간 일치는 rhwp renderer의 96dpi 실루엣 gate 통과를 뜻하지 않는다.
필수 검증이 끝나기 전에는 PR 제출·수용 완료로 판정하지 않는다.

- 이슈: <https://github.com/edwardkim/rhwp/issues/8863>
- 코드 후보: `fb5a25f868609ff81cb9fedb5b0a29a83ebf63c6`
- 회귀: `tests/cases/issue_8863_delete_preserves_existing_shapes.rs`
- 수정: `src/model/paragraph.rs`, `Paragraph::delete_text_at`
- 이 보고서 작성 후의 최종 source/producer SHA와 추가 검증 결과는 완료 시 갱신한다.

## 입력과 재현

- 원본: `samples/issue8863/articles-of-incorporation.hwp`
- 출처: <https://github.com/NomaDamas/k-skill/blob/main/corporate-registration-consulting/templates/attachment-hwp/articles-of-incorporation.hwp>
- 공개 원본 SHA-256: `244f5eddd3bda1b200ad28c2a4bd1295182949c502255821d418f6605dc8cb52`
- 출처와 MIT 고지: `samples/issue8863/README.md`, `samples/issue8863/LICENSE`
- 원본 저장 제품: Hancom Office 2010, `8.0.0.466`, HWP5 `5.0.3.0`.
- 원본 정보: 구역 1개, 문단 411개, 19쪽.
- 동작: `delete_text_native(0, 4, 11, 2)`로 첫 `회사` 두 글자를 삭제한다.
- 기대 본문: `제 1조(상호) 본 는 {{COMPANY_NAME}} 라 한다.`

구역 0 문단 4에는 시작 위치 0의 글자 모양 참조가 `(0,7), (0,6), (0,12), (0,6)`으로
중복 저장되어 있다. 원본 조회에서는 마지막 모양 6이 유효하다. 이전 삭제 처리는 지우지 않은
본문의 참조까지 첫 모양으로 바꾸어 일반 본문이 굵게 되는 회귀를 일으켰다.

## 도입 경로와 수정 계약

최초 도입은 PR #3580의 `f81e24c26222363658b468e51d526b0298f6c0cd`
(`fix(model): delete_text_at 이 클램핑된 char_shape ref 중복을 제거하도록 (#3576)`)이며,
merge는 `b33502137e7a77ac8b0b6dc1f07bc951b60aeb34`다.
원래 #3576은 날짜 셀을 전체 삭제한 뒤 재입력할 때 작은 잔여 글자 모양이 적용되는 문제였다.
클램핑으로 같은 위치에 모인 참조를 정리하는 수정이 **원본부터 중복된 참조**에도 적용되면서,
원본의 마지막 참조 우선 의미를 잃는 #8863 회귀가 도입됐다.

역사 재현에서는 `f81e24c`의 parent가 통과하고 해당 커밋은 실패했으며,
merge parent는 통과하고 merge 커밋은 실패했다. 릴리스 경계도 `0.8.2` 통과 → `0.8.3` 실패로 확인했다.
이 결과는 도입 경계의 동작 재현이며, 최종 후보의 base/head renderer 전쪽 비교를 뜻하지 않는다.

`034df2ec0eef8d93dbc45b618f5f7c8de0ed8886`
(`fix(model): #4162 삽입 취소 후 원문 서식 보존`)는 최초 도입이 아니라
후속 PR #4271의 부분 삭제 오른쪽 경계 보존 보정이다. 현재 코드의 마지막 blame만으로
최초 도입을 판정하지 않으며, #3576의 전체 삭제 재입력 계약과 후속 부분 삭제 경계 계약을
유지한 채 원본 중복 참조의 의미를 별도로 보존해야 한다.

수정은 삭제로 경계를 이동하기 전에 원본의 중복 시작 위치를 마지막 유효 모양으로 정리한다.
첫 참조가 0보다 늦게 시작하고 그 위치가 중복이면, 첫 참조 앞에 암묵적으로 적용되던
모양을 위치 0으로 명시하여 #6788의 앞 구간 계약도 보존한다.
그 뒤 기존 삭제 범위 이동과 경계 병합 규칙을 적용한다. 삭제 때문에 새로 모인 경계를
일괄 마지막 우선으로 바꾸거나, 문서별·backend별 예외를 추가하지 않는다.

보호 범위는 다음과 같다.

- 실제 문서의 삭제·동일 길이 치환·전체 치환 뒤 남은 모든 글자의 문자·굵기·색상.
- HWP와 HWPX 저장 후 다시 열기에서도 같은 계약.
- 원본 중복 경계와 삭제로 모인 경계의 구분.
- BMP/astral 혼합 텍스트의 모든 삭제 범위, 전체 삭제 후 재입력, suffix 삭제.
- 늦게 시작하는 첫 참조의 암묵적 앞 구간과 무동작 삭제의 원본 참조 보존.

## 자동 회귀 결과

| 실행 | 결과 | 증거 |
| --- | --- | --- |
| 수정 전 이슈 회귀 | 8개 중 6개 실패, 2개 통과 | `output/issue8863/red.log` |
| 수정 후 이슈 회귀 | 8/8 통과 | `output/issue8863/green.log` |
| 기본 Native 전체 회귀 | 13,138개 통과, 45개 skip, 실패 0 | `output/issue8863/full-regression.log` |
| 원본 anomaly 검사 | base/head 모두 19쪽, 신호 없음 | `output/issue8863/base-anomaly.json`, `output/issue8863/fixed-anomaly.json` |

anomaly의 신호 없음은 굵기·색상이나 시각 fidelity 통과 판정이 아니다.
이슈 회귀는 API·모델·저장 왕복 계약이며 화면 좌표나 renderer golden을 고정하지 않는다.
전체 회귀 로그에는 설치된 nextest `0.9.137`과 권장 `0.9.140`의 버전 차이 및
`profile.ci-duration-observation.junit.report-skipped` 키 무시 경고가 있다. 실제 실행된
13,138개 검사의 통과와 별개로 이 경고를 숨기지 않는다.

## 독립 한컴 편집 및 Print 기준

정식 기준 PDF: `pdf/issue8863/hancom-edited-expected-2020.pdf`

- PDF SHA-256: `17e1d14b6a6eeb1f654ccba9c984004fc16dd8d7e3fd29eed5bc944f0e98a47d`
- PDF 생성 시각: `2026-11-10T09:09:55+09:00` (`2026-11-10T00:09:55Z`).
- 실제 한컴: Hancom Office 2020 HOffice110, `11.0.0.9166`.
- PDF driver: Hancom PDF `1.3.0.550`.
- 실행 OS build: Windows `26300.9711`.
- PDF: 19쪽, A4 `595 × 841 pt`, PDF 1.4.
- PDF 내장 글꼴: `Haansoft Batang`, `FZSY--SURROGATE-0`.
- 로컬 작업만 수행했으며 원본이나 편집본을 외부 변환 서비스로 업로드하지 않았다.

원본의 로컬 사본을 별도 `HWPFrame.HwpObject` 인스턴스로 열고 다음 동작을 수행했다.

1. `EditMode = 1`로 사본을 편집 가능 상태로 설정한다.
2. `SetPos(0, 4, 13)` 후 `HAction.Run("DeleteBack")`를 두 번 실행한다. 두 호출은 모두 `True`였다.
3. 문단 텍스트를 읽어 `제 1조(상호) 본 는 {{COMPANY_NAME}} 라 한다.`를 확인한다.
4. `SaveAs(..., "HWP", "")`로 독립 편집 HWP를 저장한다.
5. `HAction.GetDefault("Print", HParameterSet.HPrint.HSet)` 후 아래 설정을 적용하고
   `HAction.Execute("Print", HParameterSet.HPrint.HSet)`를 실행한다. 결과는 `True`였다.

| Print 설정 | 값 |
| --- | --- |
| PrinterName | Hancom PDF |
| PrintMethod | 0, 한 쪽씩 |
| ZoomX / ZoomY | 100 / 100 |
| Range | 0, 전체 문서 |
| PrintToFile | 1 |
| UseThread | 0 |

PDF 저장/내보내기가 아니라 실제 Print 경로다. 원본 저장 제품이 2022 이하이므로
2020 Print profile과 이후 renderer 비교의 `compat 2022`를 사용한다.

독립 편집 HWP SHA-256은 `ddaa57c6af58cc577fd5e5816c6aeff312aeb1e852a8b6e643ff5f322eecb366`이다.
수정 CLI가 같은 삭제를 수행한 `native-delete.hwp`의 SHA-256은
`d72724f4f85027f85d2bbbb200f10965f9e57934d5d667d777a85b90c21efa53`이다.
이를 같은 한컴 Print 설정으로 출력한 `native-delete-2020.pdf`의 SHA-256은
`5d1740c75674fb6c70968924dde286418cdf899e0ce62f112f62c172e67a8fc7`이다.

두 PDF는 전 19쪽의 추출 텍스트와 PyMuPDF 기본 72dpi RGB 래스터 샘플 바이트가 각각 일치한다.
1쪽을 직접 열어 삭제된 위치와 주변 일반/굵은 본문의 보존을 확인했다. 이는 수정 저장본이
독립 한컴 편집과 같은 인쇄 결과를 낸다는 증거이며 Native Skia/Canvas raw gate와는 구분한다.

세부 작업 기록은 `mydocs/report/task_m100_8863_assets/hancom-print-evidence.json`에 있다.
초기 `hancom-delete*` 산출물은 COM 기본 `EditMode=0`에서 삭제 호출이 실패한 탐색 결과이므로
기준으로 사용하지 않는다. 유효 기준은 `hancom-edited-expected*`뿐이다.

## Native Skia 전쪽 측정 (2026-11-10)

- source: `fb5a25f868609ff81cb9fedb5b0a29a83ebf63c6` (후속 변경은 테스트 헤더·보고서).
- producer SHA-256: `327de56629baa75f4ac2e5709d5a17d22ce53269e39eca74c4b9e4783110361c`.
- backend: `native-skia`, Print profile, `compat 2022`, 96dpi, 2px raw 실루엣.
- PDF raster: PDFium `153.0.7999.0`, pypdfium2 `5.13.0`, `pdfium_smooth_rgb_exact_dpi_v2`.
- 전체 쪽수: Native 19 / PDF 19. 누락 0쪽, 90% 미만 0쪽.
- 변경 1쪽: **96.43597%**. 전쪽 최저 9쪽: **90.96057%**. 다음 최저 12쪽: 91.35219%.
- 전쪽 TSV: `mydocs/report/task_m100_8863_assets/native-silhouette.tsv`.
- 원시값·이미지 해시·producer·글꼴 provenance: `mydocs/report/task_m100_8863_assets/native-silhouette-manifest.json`.
- 실행 로그: `output/issue8863/visual/native-scores.log`.
- 대표 직접 비교: `mydocs/report/task_m100_8863_assets/native-compare-page1.png`.
- 대표 review: `output/issue8863/visual/native-review/native-delete/review/review_001.png`.
- 대표 overlay: `output/issue8863/visual/native-review/native-delete/overlay/overlay_001.png`.

명령은 다음과 같다. Native CLI는 빌드 완료된 절대 경로를 사용했다.

```bash
/home/edward/mygithub/rhwp/venv/bin/python scripts/visual_sweep.py \
  --silhouette-only --rasterizer png \
  --hwp output/issue8863/visual/native-delete.hwp \
  --pdf pdf/issue8863/hancom-edited-expected-2020.pdf --key native-delete \
  --rhwp-bin /home/edward/mygithub/rhwp/target/pr-review/release-test/rhwp \
  --compat 2022 --dpi 96 --out output/issue8863/visual/native-scores
```

원본에 선언된 글꼴과 한컴 PDF의 실제 글꼴, rhwp 공급 글꼴은 구분한다. 이번 측정은
자동 로컬/웹폰트 공급을 사용했으며 `굴림 → NotoSansKRExtraLight`,
`바탕 → NotoSerifKRExtraLight-Regular`, `한컴바탕확장 → NotoSansKR-Regular` 매핑을 기록했다.
한컴 PDF의 `Haansoft Batang`과 동일 글꼴 프로그램으로 측정한 것이 아니다.
글꼴 환경 SHA-256은 `c9b003f735496f348b568e41383e419f81788be47d59575fe3b18bf2389b8c44`이고,
각 실제 font program·source WOFF2 해시는 manifest의 `font_supply.files`에 있다.

1쪽 compare와 최저 9쪽의 양쪽 전체 PNG를 직접 확인했다. 삭제한 위치가 일반 본문으로 남고
주변 제목의 굵기 구분 및 줄 흐름은 보존됐다. 반면 본문과 제목의 글꼴 모양·자간 차이는 명확하다.
따라서 **90% 수치 조건 충족과 완전 시각 일치는 구분**한다. `--silhouette-only` manifest의
최종 review gate는 `not_evaluated`이며, 수치만으로 전체 검토를 완료하지 않는다.
이 측정은 base/head 회귀 비교와 fresh WASM 확인을 대체하지 않는다.

## fresh WASM 실제 편집과 Canvas 측정 (2026-11-10)

최적화 web package를 사용한 실제 `deleteText`·export·reopen API 검증 뒤 생성한
`output/issue8863/visual/wasm-delete.hwp`를 입력으로 전쪽 Canvas PNG를 비교했다.
이 파일의 SHA-256은 `d72724f4f85027f85d2bbbb200f10965f9e57934d5d667d777a85b90c21efa53`으로,
Native 편집 저장본과 바이트 수준에서 동일하다.

- 빌드: Docker 네트워크 풀 고갈로 실행 실패 후 공식 host locked-wrapper 경로를 사용했다.
  `output/issue8863/wasm-host-build.log`에 release 최적화와 `wasm-opt` 완료를 기록했다.
- `pkg/rhwp.js`: `cb4d7fea693a432c873a47f3cd0605b6c0cad2eb92a33523d1789eb28e0c50ed`.
- `pkg/rhwp_bg.wasm`: `cf5a2467b1fb8e357dc7d72c541a766bee1d2f9b16aa4482933502e0bfd0c819`.
- Studio public 파일과 위 package 해시 일치: `output/issue8863/wasm-hashes.txt`.
- 브라우저: Chrome for Testing `156.0.8078.47`.
- 조건: Canvas PNG Print profile, `compat 2022`, 96dpi, 2px raw 실루엣.
- 전체 쪽수: WASM 19 / PDF 19. 누락 0쪽, 90% 미만 0쪽.
- 변경 1쪽: **97.39113%**. 전쪽 최저 12쪽: **92.99618%**. 다음 최저 9쪽: 93.33124%.
- 전쪽 TSV/manifest: `mydocs/report/task_m100_8863_assets/wasm-silhouette.tsv`,
  `mydocs/report/task_m100_8863_assets/wasm-silhouette-manifest.json`.
- 실제 page count·브라우저·worker별 글꼴 로딩: `mydocs/report/task_m100_8863_assets/wasm-export-complete.json`.
- 대표 compare/review/overlay: `output/issue8863/visual/wasm-review/wasm-delete/`.
- 실행 로그: `output/issue8863/visual/wasm-scores.log`, `output/issue8863/visual/wasm-review.log`.

```bash
/home/edward/mygithub/rhwp/venv/bin/python scripts/visual_sweep.py \
  --silhouette-only --rasterizer png \
  --hwp output/issue8863/visual/wasm-delete.hwp \
  --pdf pdf/issue8863/hancom-edited-expected-2020.pdf --key wasm-delete \
  --rhwp-bin /home/edward/mygithub/rhwp/target/pr-review/release-test/rhwp \
  --wasm-pkg pkg --compat 2022 --dpi 96 --out output/issue8863/visual/wasm-scores
```

Canvas의 공급 목록 출처는 `system_fontconfig`이며 실제 요청 family는 `Noto Serif KR`이다.
worker별 기록에 variable/정적 weight의 로딩 파일 해시가 있다. 예를 들어 variable program은
`ad964df5f03b757746aeb34c93ba26c2353939e8dc024216d425d5c7b9f21ae1`,
Regular는 `78e2a43d067065fa0934765ddf27b7c051a6a43a8407c1dbf6774d3e71b489f4`,
Bold는 `df809f42883c47180dbcd592432c83b3f3abbb67b801dc6e7f9b18bb72efb512`다.
Native 자동 웹폰트 경로와 Canvas의 실제 공급 경로가 같다고 주장하지 않는다.
backend별 base/head 비교에서는 각각 이 조건을 고정해야 한다.

1쪽 compare와 최저 12쪽의 전체 PNG 양쪽을 직접 확인했다. 삭제 위치의 일반 본문,
주변 제목의 굵기 구분과 내용·줄 흐름은 유지됐고 글꼴·자간 차이는 남아 있다.
원본 1쪽에서 보이는 모든 문장이 새로 굵어지는 결함은 재현되지 않았다.
`silhouette-only`의 review gate는 `not_evaluated`이므로 이 결과도 수치 조건과
직접 확인 범위만 입증하며 전체 최종 수용 판정을 대신하지 않는다.

## 추가 검증 완료와 남은 범위

- `cargo fmt --all -- --check`: 통과.
- 기본 Clippy, WASM lib Clippy, workspace all-targets Clippy: 모두 `-D warnings` 통과.
- `cargo build --locked --workspace`: 통과.
- manifest 검사: base `b5064fae1eefff92d60464c51844a38debec09f9` 대비 통과.
- 헤더 시각 갱신 및 suite 재생성 후 이슈 회귀 재실행: 8/8 통과.
- Native Skia lib: 총 4,035개 통과, 12개 ignored. rhwp 3,853개, 나머지 workspace lib 182개.
- Native placeholder: 2/2 통과. Native direct-PDF: 35/35 통과.
- 위 로그: `output/issue8863/final-checks.log`.
- 최소 글꼴 회귀는 bwrap으로 OS/사용자 글꼴을 숨기고 저장소 opensource 글꼴만 노출했다.
- 새 샘플 security sweep 입력을 명시해 기본 전체 회귀를 실행했다.
- Native/WASM source는 코드 후보 `fb5a25f868609ff81cb9fedb5b0a29a83ebf63c6`; 이후 변경은 테스트 주석과 증적뿐이다.
- 별도 devel base/head renderer 재빌드 쌍과 2%p 무회귀 게이트는 아직 미실행이다.
- 구현과 로컬 증적만 저장했고 push, PR 생성, 외부 게시, 병합은 하지 않았다.

## 최소 증적 보존과 비교 범위의 한계

영구 증적은 `mydocs/report/task_m100_8863_assets/`에 보존했다.
전쪽 Native/Canvas TSV와 원본 manifest, 각 backend의 1쪽 비교 PNG, WASM export 메타데이터,
독립 한컴 Print 기록 및 `SHA256.json`을 포함한다. 모든 쪽 PNG는 중복 보존하지 않았다.
Canvas 대표 비교는 `mydocs/report/task_m100_8863_assets/wasm-compare-page1.png`다.
manifest 내부 `output/`·절대 경로는 실제 실행 provenance로 유지했으며,
영구 보존 파일은 assets의 README와 SHA256 목록으로 식별한다.

**Native와 Canvas 전 19쪽 측정은 모두 실제 수정 후보와 독립 한컴 Print PDF의 비교다.
별도의 base/head renderer 쌍은 아직 없다.** 따라서 90% 수치 조건 충족이나 Native와 Canvas의
상호 비교로 devel 대비 2%p 무회귀를 주장하지 않는다. 최종 제출 판단 시 이 미검증 범위를
별도로 처리해야 한다.

## 로컬 작업 종료

원본 작업공간의 tracked 파일은 변경하지 않았고 격리 브랜치에만 구현과 증적을 커밋했다.
검증 종료 및 Cargo/Rust/WASM 프로세스 부재를 확인한 뒤 저장소 지침대로 공유
`target/pr-review/debug`만 정리했다(14,022,496,256 bytes 확보).
`release`, `release-test`, `wasm32-unknown-unknown` 프로필과 증적은 보존했다.
정리 로그는 `output/issue8863/cache-cleanup.log`에 있다.
