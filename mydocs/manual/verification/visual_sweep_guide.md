---
kind: guide
status: active
canonical: mydocs/manual/verification/visual_verification_governance.md
last_verified: 2026-10-05
---

# PDF/SVG visual sweep 가이드

## 목적

조판·렌더링 영향 변경은 실제 소비 경로 기준으로 Native/fresh WASM Visual Sweep·페이지별 TSV를
반드시 산출한다. 편집 command·parser·model·serializer도 적용 대상이 될 수 있다.
검증 범위의 한 페이지라도 **90% 미만** 또는 측정 불가이면 작성자가 자기 branch에서 PDF/overlay
원인을 재검토·수정하고 새 head로 재실행한다. **정확히 90%는 통과**한다.
쪽수·페이지 분할 변경은 전체 문서를 비교하며 평균값·높은 페이지 선별로 미달을 숨기지 않는다.
기준 PDF는 [원본 저장 버전에 맞는 한컴 Print 출력](../mcp_hwp2024Convert_usage.md#기준-pdf-인쇄-계약)을 따른다.
MCP는 저장 제품에 따라 engine 2020/2024를 명시하고, 수동 출력도 PDF 저장/내보내기 대신 인쇄를 쓴다.
TSV·실행 로그·중간 JSON은 ignored `output/pr-review/<id>/`에 보존하고 Git에 커밋하지 않는다.

`scripts/visual_sweep.py`는 rhwp가 만든 SVG/render tree와 한컴 기준 PDF를 비교해
문항 흐름 drift, frame overflow, 줄 순서 겹침 같은 후보를 자동으로 찾는 보조 도구다.

이 도구는 메인테이너의 최종 시각 판정을 대체하지 않는다. 대신 다음을 빠르게 확인한다.

- SVG/PDF 페이지 수 일치
- PNG/PDF raster overlay 차이 위치
- 페이지별 픽셀/잉크 영역 일치율
- 문항 marker y drift 후보
- frame/tail overflow 후보
- 수식/본문 겹침 후보
- 줄 band/order drift 후보
- **그림·float 주변의 본문이 좁은 세로 열로 재흐름한 단별 text-flow collapse 후보**
- **Square/Tight/Through 그림의 물리 box를 본문 TextLine이 3행 이상 가로지르거나 edge에 맞닿는 후보**
- **우측 Body 표의 좌측 strip은 PDF에 본문 잉크가 있지만 rhwp에서 거의 비는 non-inline table wrap 후보**
- **표 셀의 visible line 경계 침범 또는 visible-ending 자연 text 폭 위험 후보**
- **명시적 SVG clip이 glyph 근사 band의 상·하단을 2px 이상 부분 절단하는 후보**
- **구조 heuristic에 걸리지 않는 glyph·PUA·제품명 표시 차이**의 review 후보

### PDF와 같은 인쇄 프로필

Visual Sweep의 Native SVG는 글꼴 공급 방식과 관계없이 `export-svg --profile print`로,
fresh WASM SVG는 `renderPageSvgWithProfile(page, 'print')`로 생성한다. 빈 누름틀의
편집 화면 안내문은 PDF에 표시되지 않으므로 두 출력에서 제외한다. 누름틀에 실제로
입력된 본문은 인쇄 내용이어서 그대로 비교한다. 같은 인쇄 규칙으로 투명 테두리 등
다른 편집 화면 전용 표시도 제외하며, 문서의 쪽수·본문 줄·표·그림을 비교 대상에서
가리지 않는다. `run_manifest.json`의 `comparison_profile`과 WASM
`wasm/manifest.json`의 `comparisonProfile`로 사용한 프로필을 확인한다.

이는 낮은 점수의 본문 영역을 마스킹하는 예외가 아니다. 변경 전 화면 프로필의
PNG는 인쇄 프로필 실행의 점수·증적과 섞지 않고 새 출력에서 전체 영향을 다시
확인한다. 누름틀 안내문 표시 자체는 편집 화면 프로필의 별도 회귀 검사로 확인한다.

원문에 특수 인쇄 방식이 저장돼 있으면 `rhwp info --json`의 `printMethod`와
`printMethodImpliesNup`을 먼저 확인한다. 값 4·5의 모아 찍기는 현재 rhwp 출력에
반영되지 않으므로, 같은 쪽수·96dpi라도 한컴 PDF의 내용 배율이나 용지 배치가 다를 수
있다. 이런 차이를 DPI·글꼴 문제로 단정하거나 SVG를 사후 확대해 통과시키지 않는다.
같은 원문을 지정한 한컴 엔진으로 다시 PDF 출력해 용지 크기·글자 크기·대표 PNG를
대조하고, 여전히 다른 쪽은 시각 보류와 출력 구현 범위로 기록한다(#6778).

## 실루엣 보조값만 빠르게 TSV 산출

`--silhouette-only`는 기존과 같은 2px 관용 실루엣 계산식으로 `silhouette.tsv`를
저장한다. compare·overlay·review PNG와 상세 구조 분석은 생성하지 않는다.
원문 입력 시 비교용 SVG/PDF raster는 필요하므로 생성하며, 이미 있는 PNG는
`--png-pair`로 재사용해 원문 export·raster 작업도 생략할 수 있다.

```bash
python3 scripts/visual_sweep.py --silhouette-only \
  --png-pair "output/<기존 실행>/<key>/rhwp_png" "output/<기존 실행>/<key>/pdf_png" \
  --out "output/<새 실행>/silhouette"
```

```bash
python3 scripts/visual_sweep.py --silhouette-only \
  --hwp "samples/<원문>.hwp" --pdf "pdf/<기준>.pdf" --key "<key>" \
  --rhwp-bin target/pr-review/release-test/rhwp --dpi 96 \
  --out "output/<실행>-native-scores"
```

위 예시의 `<...>`는 실제 경로·이름으로 바꾼다. 현재 코드로 web package를 새로 빌드한 뒤
fresh WASM도 같은 입력·기준 PDF·DPI·글꼴 환경으로 실행한다.

```bash
python3 scripts/visual_sweep.py --silhouette-only \
  --hwp "samples/<원문>.hwp" --pdf "pdf/<기준>.pdf" --key "<key>" \
  --rhwp-bin target/pr-review/release-test/rhwp --wasm-pkg pkg --dpi 96 \
  --out "output/<실행>-wasm-scores"
```

원문 입력 결과는 `output/<실행>-native-scores/<key>/silhouette.tsv` 및 WASM 대응 경로에,
`--png-pair` 결과는 지정한 `--out` 바로 아래에 저장된다. 각 TSV를 스프레드시트에서 탭 구분으로
열어 `page`별 `tolerant_content_match_percent`를 Native/WASM 간 대조하고, `below_90`과
최저값을 확인한다. 이 값은 **실루엣 일치율 보조값**이며 전체 렌더링 정확도나 구조 일치율이 아니다.

검증 대상 전체 문서에서 Native와 fresh WASM TSV를 각각 먼저 산출하고,
비교 쪽수·최저 일치율·90% 미만/누락 쪽을 결과보고와 PR 본문에 기록한다.
WASM은 같은 원문·PDF에 `--wasm-pkg pkg`와 검증한 `--rhwp-bin`을 명시한다.
원문 전체 쪽수는 Native `native-export.json`의 `pageCount`, WASM `wasm/manifest.json`의
`pageCount`와 독립 PDF 메타데이터를 별도로 대조한다. 선택 raster 개수를 전체 쪽수로 쓰지 않는다.
기존 `--png-pair`는 양쪽 번호 누락을 거부하지만 원문에 몇 쪽이 있어야 하는지는 입증하지 않는다.

미달/구조 차이 쪽과 대표 변경 경계의 PNG만 일반 모드 `--pages`로 추가 생성한다.
일반 모드는 대상 key 디렉터리를 새로 정리하므로 TSV 실행과 **서로 다른 output 경로**를 사용한다.
예를 들어 점수는 `output/<실행>-scores`, 직접 판독은 `output/<실행>-review`에 저장한다.
대표 이미지의 PR 본문 표시 의무는 유지하며, 그 밖의 전쪽 overlay 합성은 기본 요구가 아니다.
코드가 바뀌면 영향 범위의 TSV와 대표 이미지도 새 head에서 다시 산출한다.

TSV의 첫 세 열은 `page`, `tolerant_content_match_percent`, `below_90`이며,
`silhouette_raw_match_percent`, `silhouette_boundary_reconciled_pixels`를 뒤에 함께 기록한다.
232 이진화 경계가 유색 PDF 픽셀을 빈 영역으로 오판하는 경우를 구분하기 위해 원값을 보존한다.
내용 마스크232·2px 반경은 유지하며, 원래 불일치 픽셀 중 같은 위치 양쪽 모두 흰 배경
(모든 RGB 채널244 이상)이 아니고 RGB 최대 차이가 고정32 이내이면 내용의 존재가 일치한다.
32는 기존 엄격 overlay의 기본 색상 허용치이며 사용자 `--pixel-diff-threshold`와 무관하게
실루엣 산출에서는 고정한다. 실제 흰 영역의 그림 누락, 2px 밖 이동, 큰 색상 변화는 제외하지 않는다.
엄격 색상·픽셀 지표도 그대로 남기며 변경된 보조값만으로 색상 정확도나 최종 승인을 주장하지 않는다.
기존 실행과 비교할 때에는 방법 버전 `threshold_boundary_color_support_v1`·원값·조정 픽셀 수를
함께 보고 전체 검증 대상으로 다시 산출한다. 특정 문서만 임계값·영역을 바꾸지 않는다.
`--pages 10,17,28`로 선택할 수 있으며, 번호 중복·양쪽 입력 누락을 허용하지 않는다.
`silhouette_manifest.json`에 입력 PNG 해시와 비교 쪽을 기록한다. 기존 PNG 해시는
현재 코드로 출력했다는 증명이 아니므로 원래 실행의 source/build provenance를 함께 확인한다.
일반 PNG checkpoint를 사용하는 `--resume`과는 함께 실행하지 않는다.

90% 미만이면 TSV를 남기고 exit 1로 끝난다. 모두 90% 이상이어도 이 모드의
PR 판정은 `not_evaluated`다. 전체 쪽수, 각주·문단 소속, 누락·중복은 별도로 확인하고,
문제가 있는 쪽은 일반 실행으로 review PNG를 생성해 직접 검토한다.
실루엣 보조값만으로 PR 승인이나 회귀 fixture 적합성을 판정하지 않는다.

## PR review 실루엣 gate

렌더링 변경의 새 회귀 테스트 추가에도 [회귀 추가 선행 조건](../pr_review/visual_fixture_evidence.md#렌더링-회귀-테스트-신규-추가의-시각-검증-선행-조건)을 적용한다. 관련 모든 페이지·fixture의
Native/fresh WASM 최저 일치율이 90% 미만이거나 측정 불가이면 회귀를 추가하지 않고 출력을 먼저
개선한다. 쪽수 검사는 전체 페이지를 비교하며 평균값·글꼴 예외로 이 조건을 면제하지 않는다.

조판·렌더링 영향 변경은 Visual Sweep을 반드시 사용하고 검증 범위 전체 TSV와 각 대표 review PNG의
`tolerant_content_match_percent`(2px 이웃 관용 내용 실루엣 일치율 보조값)는 **90% 이상**이어야 한다.
`scripts/visual_sweep.py`는 90% 미만 또는 측정 불가 페이지가 있으면 PNG와 manifest를 남긴 뒤 exit
non-zero로 끝내며, manifest의 `pr_review_gate.status`를 `re_review_required`로 기록한다. 이 상태에서는
새 PR을 만들지 않고 이미 열린 PR은 승인·통합하지 않는다. 기여자는 자기 branch에서 PDF와 overlay를 다시 판독해
원인을 수정한 새 head로 재실행하고, gate를 통과할 때만 PR을 생성·갱신한다. reviewer는 보류를 기록하며 메인터너
보정으로 그 변경을 대신하지 않는다. 이 규칙은 지표를
올리기 위해 tolerance·DPI·대상 영역을 사후 변경하는 근거가 아니다.

양쪽 캡처의 내용 픽셀이 모두 0개인 실제 빈 쪽은 빈 실루엣의 일치율 100%로 계산한다.
한쪽에만 내용이 있으면 0%이며, 캡처·페이지·지표 누락은 여전히 측정 불가로 보류한다.
빈 쪽도 전체 페이지 수와 원본의 쪽 소유 비교에서 제외하지 않는다.

같은 원본·출력 환경의 기준 PDF와 rhwp **전체 페이지 수**가 다르면 선택 페이지의 실루엣 게이트가
통과하더라도 PR을 재검토한다. `--page`/`--pages`로 선택한 쪽의 산출물 개수는 전체 페이지 수의
증거가 아니다. 누락·추가된 쪽의 시작 경계와 앞뒤 내용을 확인하고 새 head에서 다시 비교한다.
글꼴 예외도 페이지 수 차이를 면제하지 않는다.

Native의 `--page`/`--pages`는 SVG와 render tree도 선택 쪽만 내보낸다. sweep의 쪽 번호는
1부터이며 CLI의 0부터 시작하는 `-p`로 내부 변환한다. `--resume`에 새 선택 쪽을 추가하면 빠진
쪽만 생성하고, 선택 없이 전체를 요청하면 전체 내보내기를 수행한다. `native-export.json`의
`pageCount`와 최종 manifest의 `native_document_pages`는 CLI가 보고한 **전체 문서 쪽수**다.
`exported_svg_pages`/`exported_render_tree_pages`는 실제 저장한 파일 수이며 전체 쪽수로 쓰지 않는다.
선택 비교 통과는 전체 페이지 일치나 fresh WASM 검증을 대신하지 않는다. WASM은 기존 전체
내보내기 경로를 사용한다. 로그와 중간 산출물은 `--out output/...` 아래에 보관한다.

한컴 PDF와 rhwp raster의 실제 글꼴이 다르면 원인 진단 자료로
`--font-mismatch-evidence <UTF-8 파일>`을 지정해 각 쪽의 원래/대체 font family, 확인 방법과
representative PNG를 기록한 증거 파일의 경로·SHA-256을 manifest에 남긴다. 단순 anti-aliasing, 작은
baseline 차이, 글꼴 이름의 추정, `flagged=0` 또는 CI 녹색으로 gate를 면제하지 않는다.
이 증거도 90% 미만·측정 불가를 `font_mismatch_exception`으로 통과시키지 않는다.
PDF와 rhwp의 표 괘선·문단 시작·그림 경계 좌표를 먼저 비교한다.
이 위치가 어긋나면 낮은 점수를 글꼴 탓으로 분류하지 않고 배치를 고쳐 다시 캡처한다.
#7359 14쪽은 표 행 높이가 같아도 표 전체가 약 15px 위에 있어 68.17%였고,
페이지 첫 문단의 저장 간격을 복구한 뒤 97.48%가 됐다.

```bash
RHWP_FONT_PATH="/절대/경로/검증된-한컴-글꼴" python3 scripts/visual_sweep.py \
  --file-target <key> <입력 HWP/HWPX> <기존 한컴 PDF> \
  --rhwp-bin target/pr-review/release-test/rhwp --pages <영향 쪽> --dpi 96 \
  --out output/<review>
```

실행 전에 `RHWP_FONT_PATH`의 모든 디렉터리가 실제로 존재하고, 입력 문서가 요구한 face가 그 경로 또는
운영체제에 설치됐는지 확인한다. 존재하지 않는 `ttfs/hwp`, `ttfs/windows` 같은 과거 경로를 설정하면
환경변수 자체는 전달돼도 renderer가 fallback face로 조판해 낮은 지표를 만들어 낸다. 이 경우에는
`--font-mismatch-evidence` 기록으로 통과 처리하지 말고, 먼저 올바른 글꼴 공급으로 다시 실행한다.

이 도구의 절차상 지위는 [시각 검증 거버넌스의 라우팅 표](visual_verification_governance.md)를
따른다. 독립 기준 PDF와 실제 사용자-visible 실패를 조사할 때는 bug-hunter가 상위이고, sweep은
후보 검출·재현 범위 축소·수정 전후 무회귀만 담당한다. 이미 원인과 발동 페이지가 확정된 renderer/layout
PR에서는 이 가이드를 직접 시작점으로 쓴다.

한컴 기준 PDF가 있는 실물 문서에서는 먼저
`tools/fidelity_compare/fidelity_compare.py --text-only --export-all-svg --layout-ledger`로 전수 후보를
수집한다. visual sweep은 그 중 Square/Tight/Through 그림↔본문 기하 규칙을 같은 render tree에서
직접 재사용하여 `square_wrap_text_overlap` flag와 annotation으로 남긴다. 반면 PDF↔SVG text owner,
그림 앞 문단의 `float-owner-shift`, 표 fragment, page-count ledger는 sweep이 다시 계산하지 않으므로
fidelity 원장을 함께 보존해야 한다.
표 우측선 밖으로 문단이 돌출되는 경우에는 `table-cell-text-boundary-candidates.tsv`, 페이지 경계나
셀 clip에서 글자 윗부분·아랫부분이 잘리는 경우에는 `svg-text-band-clip-candidates.tsv`를 먼저 본다.
전자는 중첩 셀을 자기 소유 Cell에만 귀속하고, overflowing edge가 선행/후행 공백뿐인 TextRun은
제외한다. visible 문자로 끝나는 자연 폭은 `natural_visible_width_risk`로 올리지만 저장 자간/justify
뒤에는 실제 glyph가 안쪽에 들어올 수 있다.
후자는 완전히 clip 밖인 stale text를 제외한다. 글꼴 bbox와 근사 glyph band 때문에 false positive가
가능하므로 physical page의 PDF/rhwp raster로 반드시 확정한다.
이 bridge의 render tree가 없거나 JSON이 손상되면 sweep은 `flagged=0`을 보고하지 않고 실패해야 한다.
`fidelity_compare`의 Python 환경과 실행 명령은
[도구 README](../../../tools/fidelity_compare/README.md)를 따른다. 저장소 로컬 `venv/`의 공통
계약은 그 문서가 연결하는 [개발 환경 가이드](../dev_environment_guide.md)가 정의한다.

## 새 WASM 출력 비교

`--wasm-pkg <폴더>`를 지정하면 `wasm-pack --target web`으로 만든 `rhwp.js`와
`rhwp_bg.wasm`을 실제 Chrome에서 실행한다. SVG는 `renderPageSvgWithProfile(page, 'print')`, 분석용 render tree는
**같은 WASM 문서의 `getPageRenderTree`**에서 얻는다. Native render tree를 WASM 출력의
기하 근거로 대신 쓰지 않는다.

```bash
venv/bin/python scripts/visual_sweep.py \
  --file-target reg80168 samples/80168_regulatory_analysis.hwp pdf/80168_regulatory_analysis-2022.pdf \
  --rhwp-bin target/pr-review/debug/rhwp \
  --wasm-pkg /path/to/new-web-pkg \
  --pages 21,49,75-77,108-109 --dpi 96 --out output/wasm-review
```

CLI는 같은 원본의 `export-svg --font-style --profile print`가 만든 글꼴 별칭과 note-shape 메타데이터를 제공한다.
Sweep은 `@font-face`만 WASM SVG에 보충하며 텍스트·좌표·그리기 노드는 수정하지 않는다.
이후 기존 Chrome webfont rasterizer로 비교·overlay·review PNG를 생성한다. 별도 HTML에
raw SVG만 붙이면 macOS의 legacy `휴먼명조` 등의 설치 폰트가 잘못 선택될 수 있으므로
브라우저 증적도 이 경로로 캡처한다. PNG 제목에는 `(WASM)`을 표시한다.

`wasm/raw_svg`에는 수정 전 WASM SVG, `render_tree`에는 WASM의 분석 트리를 보존한다.
`wasm/manifest.json`은 Chrome 버전·쪽 수·원 SVG 해시를, `run_manifest.json`은 JS/WASM
패키지와 exporter 해시를 기록한다. 패키지가 바뀌면 `--resume`은 이전 증적을 거부한다.
WASM의 쪽 수가 Native와 달라도 Native의 페이지 구성을 강제하지 않으며, 실제 WASM SVG와
render tree 중 한 쪽이 누락되면 성공으로 처리하지 않는다. 패키지 빌드는 Sweep 실행 전에 완료한다.

## 원 글꼴을 명시적으로 공급하는 진단

로컬 글꼴 이름이 존재해도 Chrome이 실제로 같은 face를 사용하는지는 별도 확인한다.
글꼴 굵기 검토에서 대체 face를 원 face의 증거로 세지 않는다. 필요한 경우 CLI와 같은
`--embed-fonts=full --font-path <디렉터리>`를 sweep에 전달한다. `--embed-fonts`만 지정해도
전체 임베딩이며 `=subset`은 거부한다. 현재 CLI의 PDF용 subsetter는 Unicode `cmap`을
제거하므로 SVG `<text>`용 검증 폰트로 사용할 수 없다. Native와 WASM에
같은 `@font-face` 공급을 적용하며 WASM의 text·좌표·render tree는 Native 것으로 대체하지 않는다.
mode와 해당 디렉터리의 폰트 파일 hash가 바뀌면 `--resume`은 이전 증적을 거부한다.

```bash
venv/bin/python scripts/visual_sweep.py \
  --file-target bold samples/issue2470/36382471_masked.hwpx pdf/issue2470/36382471_masked-2022.pdf \
  --rhwp-bin target/pr-review/release-test/rhwp --pages 1,2 \
  --embed-fonts=full --font-path /path/to/private/validated-font-subsets --out output/bold-review
```

폰트 소유·사용 범위가 확인된 파일만 검증용 scratch에 둔다. embedded SVG/폰트 바이너리를
공개 증적에 추가하지 않고 PNG와 source font/subset 해시·실제 선택 face를 기록한다.
사용 glyph의 outline/advance를 유지한 유효한 subset은 full 모드로 전달할 수 있다. 브라우저
OTS 오류나 LastResort가 있으면 성공 캡처로 세지 않는다. 원 face를 공급한 정합성 검증과
실제 Studio fallback 환경의 비교는 서로 다른 증거로 구분한다.

Sweep은 Native/WASM의 선택 SVG를 캡처하거나 resume checkpoint를 재사용하기 전에
임베딩 폰트의 Unicode `cmap`을 검사한다. 이를 위해 임베딩 모드는 Python `fonttools`가
필요하다(`python -m pip install fonttools`). 손상된 폰트·없는/빈 `cmap`·검사 의존성 누락은
`analysis/embedded_font_check.json`에 face·폰트 SHA-256·실패 이유를 남기고 중단한다.
기존 성공 요약도 `re_review_required`로 바꾸며 `--font-mismatch-evidence`로 우회할 수 없다.
이 검사는 Unicode 매핑의 존재를 확인할 뿐 모든 글자 표시나 실제 선택 face를 입증하지 않는다.
설치 폰트·외부 URL 폰트도 검사 범위 밖이다. 대표 review/overlay PNG를 열어 한글·숫자·기호,
표 안 글자와 각주까지 확인한 뒤 나머지 Sweep을 진행한다. 두부나 누락이 보이면 점수와
무관하게 그 캡처를 실패 증거로 보존하고 올바른 원본 폰트로 재산출한다.

`--embed-fonts=full`에서 `--font-path`를 생략하면 `RHWP_FONT_PATH`의 디렉터리를 사용한다.
이 환경변수도 없고 macOS 사용자 글꼴 폴더 `~/Library/Fonts`가 있으면 그 폴더를
자동으로 사용한다. 실제 공급 경로·글꼴 파일 SHA-256·선택 출처는 `run_manifest.json`의
`font_supply`에 기록한다. 명시한 경로 또는 환경변수 경로가 없거나 글꼴 파일이 비어 있으면
캡처 전에 실패한다. 입력 문서에 필요한 face가 실제로 공급되는지와 대표 PNG의 글꼴 표시를
직접 확인해야 한다. 한글 두부가 있어도 실루엣 점수만으로 gate가 통과할 수 있다.

## 필수 도구

스크립트는 실행 시작 시 다음 CLI가 `PATH`에 있는지 확인한다.

| CLI | 용도 | Ubuntu/WSL/Debian 패키지 |
|---|---|---|
| Chrome 또는 Chromium | Studio 공통 webfont를 적용해 SVG를 PNG로 변환 | GitHub hosted runner 기본 설치 또는 Chromium 패키지 |
| `node` | webfont projection을 읽고 headless browser raster를 실행 | Node.js LTS |
| `pdftoppm` | PDF 페이지를 PNG로 변환 | `poppler-utils` |
| `pdftotext` | PDF bbox-layout 추출 | `poppler-utils` |

`--svg-rasterizer rsvg`로 기존 librsvg 경로를 명시적으로 선택할 때만
`rsvg-convert`가 필요하다. 이 경우 Ubuntu/WSL/Debian 패키지명은
`libsvg2-bin`이 아니라 `librsvg2-bin`이다.

설치 예:

```bash
sudo apt update
sudo apt install chromium-browser nodejs poppler-utils
```

macOS Homebrew 환경:

```bash
brew install --cask google-chrome
brew install node poppler
```

Fedora 계열:

```bash
sudo dnf install chromium nodejs poppler-utils
```

설치 확인:

```bash
which google-chrome || which chromium || which chromium-browser
which node
which pdftoppm
which pdftotext
```

## 폰트 환경

visual sweep은 SVG를 PNG로 변환한 결과와 한컴 기준 PDF raster를 비교한다. 따라서
폰트 환경이 다르면 실제 레이아웃 회귀가 없어도 `line`, `column`, `order` 후보가
false positive로 남을 수 있다.

권장 기본 폰트:

```bash
sudo apt install fonts-noto-cjk fonts-nanum
fc-list :lang=ko | head
```

한컴/HY 계열 전용 폰트는 라이선스가 있는 로컬 환경에서만 사용하고, 저장소나 PR
첨부물에 포함하지 않는다. 정확한 한컴 기준 재현이 필요한 경우 프로젝트 외부의 폰트
디렉터리를 사용한다.

```bash
rhwp export-svg samples/exam_kor.hwp \
  --font-path /path/to/ttfs \
  --output output/font-check/
```

`--font-path`는 여러 번 지정할 수 있으며, 기본 탐색 경로(`ttfs/`, 시스템 폰트)보다
우선한다. 자세한 폰트 fallback 동작은 [export-png 명령 가이드](../export_png_command.md)의
폰트 섹션을 참고한다.

기본 `scripts/visual_sweep.py`는 `export-svg --font-style --profile print` 뒤에 Chrome headless를 사용한다.
브라우저 제어에는 Studio의 `puppeteer-core`를 재사용하므로 최초 사용 전
`npm --prefix rhwp-studio ci`로 해당 의존성을 준비한다. Chrome 실행 파일은 별도로
설치하거나 `VISUAL_SWEEP_CHROME`으로 지정한다. 캡처는 실제 content viewport를
SVG 크기로 설정하고 글꼴 로딩과 screenshot 완료를 기다린다. `--window-size`만으로
외부 창 크기를 맞추면 브라우저 장식 영역 때문에 페이지 하단이 누락될 수 있다.
`rhwp-studio/src/core/generated/font-rule-projections/webfont-supply.ts`의 현재 Studio 공통
webfont projection에서 SVG에 실제로 나타난 family만 선택해 `@font-face`로 다시 공급한다.
따라서 `함초롬바탕` 같은 CDN webfont와 `한양중고딕` 같은 번들 대체 webfont가 Studio와 같은
규칙으로 적용되며, 규칙에 없는 family는 Noto Sans KR webfont를 마지막 fallback으로 사용해
두부(□)를 피한다. SVG 좌표는 `rhwp export-svg`가 결정한 값을 그대로 유지한다.

이 경로는 CDN 응답과 webfont projection의 현재 상태에 의존한다. 실행마다 output의 rasterizer
로그와 run manifest에 선택한 rasterizer 및 Git HEAD가 남으므로, PR 판정에는 실행 OS와
`webfont` 경로 사용 여부를 함께 기록한다. 한컴/HY 전용 실폰트의 glyph 형태까지 동일하다는
증명은 아니며, 그러한 결론에는 한컴 PDF와 OVL 또는 개체 단위 대조가 추가로 필요하다.

전체 글꼴을 포함한 대형 SVG에서 `Navigation timeout of 30000 ms exceeded`가 발생하면
`RHWP_VISUAL_RASTER_TIMEOUT_MS=180000`을 sweep 명령 앞에 지정할 수 있다. 기본값은
30000ms이며 양의 정수만 허용한다. Chrome 실행·프로토콜·페이지 로딩의 대기 한도만
조절하며, DOM load·`document.fonts.ready`·screenshot 완료와 기존 글꼴·좌표·viewport는
그대로 확인한다. 타임아웃 실행은 PNG가 만들어지지 않았다면 미완료로 남기고, 새 실행의
명령·환경변수·실제 완료 결과를 기록한다. 대기 한도 변경으로 시각 gate를 면제하지 않는다.

하단선이나 도형이 누락된 경우 SVG의 요소 좌표와 조상 clip을 먼저 확인한다. SVG에는
페이지 안에 존재하는데 PNG의 동일 높이 이하가 통째로 비면 renderer 결함으로 확정하지
않고 캡처 환경을 검사한다. 브라우저 변경 뒤에는 다음 실측 검사를 사용할 수 있다.

```bash
VISUAL_SWEEP_CHROME=/path/to/chrome venv/bin/python scripts/tests/test_webfont_raster_viewport.py
```

이 검사는 독립 SVG의 네 모서리를 1배·2배 PNG 픽셀로 확인한다. 캡처 도구를 수정한
뒤 기존 checkpoint를 `--resume`으로 재사용하면 잘못된 PNG도 남을 수 있으므로 새
출력 디렉터리에서 다시 산출한다.

네트워크 없이 설치 글꼴만 비교해야 하는 특수 상황에서는 다음처럼 기존 경로를 명시한다.

```bash
python3 scripts/visual_sweep.py --target 2024-09-between20 --svg-rasterizer rsvg
```

이 `rsvg` 결과는 Studio webfont 결과와 혼용하지 않으며, PR 보고서에는 사용한 rasterizer를
명시한다.

## 사전 빌드

현재 checkout 기준 `target/debug/rhwp`가 필요하다.

```bash
cargo build
```

## 실행

전체 교육 통합 target sweep:

```bash
python3 scripts/visual_sweep.py --target all
```

특정 target만 실행:

```bash
python3 scripts/visual_sweep.py --target 2024-09-between20
```

특정 페이지만 비교:

```bash
python3 scripts/visual_sweep.py \
  --target 2024-09-between20 \
  --page 22 \
  --out output/visual-p22
```

여러 페이지 또는 범위만 비교:

```bash
python3 scripts/visual_sweep.py \
  --hwp /path/to/input.hwpx \
  --pdf /path/to/baseline.pdf \
  --pages 43-46 \
  --out output/visual-p43-46
```

`--page`는 여러 번 지정할 수 있고, `--pages`는 `1,3,5-7` 형식을 허용한다. 페이지 번호는
사용자가 PDF viewer에서 보는 1-based 번호다. `export-svg`와 render tree 추출은 문서 단위로 수행하지만,
`--page`/`--pages`가 지정되면 SVG rasterizer와 `pdftoppm`의 raster 생성부터 선택 페이지로 제한한다.
비교·overlay·analysis도 동일한 선택 페이지로만 수행한다. 따라서 `compare/compare_022.png`,
`overlay/overlay_022.png`, `analysis/annotated_022.png`처럼 실제 페이지 번호가 파일명에 남는다.

저장소 preset에 없는 일반 파일을 실행:

```bash
python3 scripts/visual_sweep.py \
  --key so-sueop \
  --hwp samples/SO-SUEOP.hwpx \
  --pdf pdf/SO-SUEOP-2024.pdf \
  --out output/visual-so-sueop
```

`--hwp`에는 `.hwp`와 `.hwpx` 모두 지정할 수 있다. 파일을 `samples/`나 `pdf/`로 복사하지 않아도
된다. 절대 경로와 현재 checkout 기준 상대 경로를 모두 허용한다. `--key`를 생략하면 문서 파일명
stem을 target 이름으로 사용한다.

여러 일반 파일을 한 번에 실행:

```bash
python3 scripts/visual_sweep.py \
  --file-target so-sueop samples/SO-SUEOP.hwpx pdf/SO-SUEOP-2024.pdf \
  --file-target pr1674 samples/pr-1674.hwpx pdf/pr-1674-2024.pdf \
  --out output/visual-custom
```

preset target과 일반 파일 target을 섞을 수도 있다.

```bash
python3 scripts/visual_sweep.py \
  --target 2024-09-between20 \
  --file-target so-sueop /path/to/SO-SUEOP.hwpx /path/to/SO-SUEOP-2024.pdf
```

일반 파일에서도 특정 페이지만 비교할 수 있다.

```bash
python3 scripts/visual_sweep.py \
  --key so-sueop-p22 \
  --hwp samples/SO-SUEOP.hwpx \
  --pdf pdf/SO-SUEOP-2024.pdf \
  --page 22 \
  --out output/visual-so-sueop-p22
```

작은 글자나 셀 clip 경계를 확대 판정할 때는 `--dpi 144`처럼 목표 DPI를 높일 수
있다. 이 값은 PDF raster와 rhwp SVG raster 양쪽에 같은 배율로 적용된다. 기본값은
96dpi이며 0 이하 값은 허용하지 않는다.

일부 공개문서 축약 샘플은 rhwp SVG/PNG 파일명이 문서 내부 원래 페이지 번호나 문서번호를 따라가고,
기준 PDF는 해당 페이지만 잘라낸 단일 페이지라 `pdf-1.png`로 생성될 수 있다. 예를 들어 rhwp 쪽은
`rhwp_177.png`인데 기준 PDF는 `pdf-1.png`인 경우다. 이때 `--page 1`처럼 사용자가 PDF viewer에서 보는
단일 페이지를 지정했고, SVG/render tree/rhwp PNG/PDF PNG 산출물이 모두 1개뿐이면 visual sweep은 자동으로
이 단일 산출물을 1:1 매칭한다. 출력 파일명은 rhwp 산출물의 실제 번호를 따라 `compare_177.png`,
`overlay_177.png`, `review_177.png`처럼 남을 수 있으므로 리뷰 문서에는 이 대응 관계를 함께 적는다.

현재 스크립트의 기본 output:

```text
output/task1274/
```

주요 산출물:

| path | 설명 |
|---|---|
| `output/task1274/summary.json` | 전체 target 요약 |
| `output/task1274/<target>/svg/` | rhwp SVG export |
| `output/task1274/<target>/rhwp_png/` | SVG를 PNG로 변환한 결과 |
| `output/task1274/<target>/pdf_png/` | PDF를 PNG로 변환한 결과 |
| `output/task1274/<target>/compare/` | rhwp/PDF 비교 이미지 |
| `output/task1274/<target>/overlay/` | rhwp/PDF PNG overlay diff 이미지와 metrics |
| `output/task1274/<target>/overlay/overlay_metrics.json` | overlay diff 페이지별 지표. manifest에는 요약이 포함됨 |
| `output/task1274/<target>/review/` | `compare`와 `overlay`를 한 장에 나란히 붙인 검토 이미지 |
| `output/task1274/<target>/analysis/metrics.json` | 페이지별 후보 상세 |
| `output/task1274/<target>/analysis/question_flow.json` | 문항 marker 흐름 비교 |
| `output/task1274/<target>/overlay_contact_sheet.png` | overlay diff 전체 요약 이미지 |
| `output/task1274/<target>/review_contact_sheet.png` | 나란히 보기 전체 요약 이미지 |

## Codex 보고 규칙

Codex가 visual sweep을 실행해 특정 페이지를 검토할 때는 결과 설명만 하지 말고, 항상 다음 세 가지를
함께 제공한다.

- `compare/compare_{page}.png` 절대 경로
- `overlay/overlay_{page}.png` 절대 경로
- `review/review_{page}.png` 절대 경로
- 해당 페이지의 `visual_accuracy_proxy_percent`

또한 Codex 화면에는 `review_{page}.png`를 먼저 열어 `compare`와 `overlay`를 한 화면에 나란히 보여준다.
필요하면 `compare_{page}.png`와 `overlay_{page}.png` 개별 파일도 추가로 연다. `compare`는 좌우 배치로
전체 시각 차이를 보고, `overlay`는 빨강/파랑/주황 차이 위치를 판단하는 용도다.
`review_{page}.png`에서는 overlay 비교 PNG 바로 아래에
`코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 약 N%.` 한 줄만 포함한다.

Codex 응답에서 이미지를 보여준 바로 아래에는 반드시 한국어 코멘트를 붙인다. 코멘트는 다음 4줄 형식을 따른다.
`visual_accuracy_proxy_percent` 값은 백분율로 환산해 첫 줄에 표시한다.

보고 예:

```text
page 22
- compare: /Users/tsjang/rhwp/output/pr-review/<review-id>/compare/compare_022.png
- overlay: /Users/tsjang/rhwp/output/pr-review/<review-id>/overlay/overlay_022.png
- review: /Users/tsjang/rhwp/output/pr-review/<review-id>/review/review_022.png
- visual_accuracy_proxy_percent: 91.23456

코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 약 91.23%.
높을수록 좋음: 기준 PDF와 rhwp PNG가 더 비슷함
낮을수록 나쁨/검토 필요: 잉크 위치나 형태 차이가 큼
단, 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값입니다
```

예를 들어 값이 `13.8381`이면 다음처럼 적는다.

```text
코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 약 13.84%.
높을수록 좋음: 기준 PDF와 rhwp PNG가 더 비슷함
낮을수록 나쁨/검토 필요: 잉크 위치나 형태 차이가 큼
단, 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값입니다
```

페이지별 값을 빠르게 확인:

```bash
jq '.pages[] | {page, overlay_png, visual_accuracy_proxy_percent}' \
  output/task1274/<target>/overlay/overlay_metrics.json
```

특정 페이지 한 개만 확인:

```bash
jq '.pages[] | select(.page == 22) | {page, overlay_png, visual_accuracy_proxy_percent}' \
  output/task1274/<target>/overlay/overlay_metrics.json
```

<a id="pr-body-visual-evidence"></a>

## PR 본문 직접 증적

renderer, layout, paint처럼 문서 비교 결과를 reviewer의 판단 근거로 쓰는 PR은 merge 전에도 대표
PNG를 PR 본문에서 바로 볼 수 있게 한다. review 문서·임시 output 경로·asset 파일명만 적어 두고
reviewer가 저장소를 찾아 열게 하지 않는다.

1. 비교를 마친 최종 PR head에 대표 review와 standalone overlay PNG를 `mydocs/pr/assets/` 아래 안정
   경로로 commit한다. PR 번호를 아직 모르면 `issue_<N>_<topic>/`처럼 issue 또는 변경 주제를 쓴다.
   PNG가 바뀌면 같은 경로를 써도 되지만, 반드시 새 head SHA로 URL을 바꾼다.
2. PR 본문에는 해당 PR의 `headRepositoryOwner/headRepository`와 정확한 `headRefOid`를 사용한 아래 형식의
   Markdown 이미지를 넣는다. target repository나 branch 이름으로 대신하지 않는다. 외부 fork PR도
   contributor fork의 head repository와 SHA를 사용한다.
3. Native/fresh WASM 등 실제 실행한 각 출력 경로마다 대표 review와 overlay를 한 장씩 표시한다. 실행하지
   않은 경로는 이미지 행을 지우고 사유를 적는다. 자동 수치는 보조값이며 사람의 직접 판독·남은 차이도
   이미지 위나 아래에 함께 기록한다.
4. `gh pr create` 또는 `gh pr edit --body-file` 뒤 `gh pr view N --json body`로 URL·한글·실제 head SHA를
   재확인하고, `gh api repos/<head-owner>/<head-repo>/contents/<asset>?ref=<head-sha>`로 asset이 그 head에
   존재하는지 확인한다. GitHub PR 화면에서 이미지가 렌더링되는 것도 직접 확인한다.

~~~markdown
## Visual Sweep 직접 증적

- 대상: 기준 PDF p55 ↔ rhwp p78, 표 외곽·PS 마크·앞뒤 내용
- 판독: Native/fresh WASM 모두 표 외곽과 마크 상대 위치를 확인했다. 자동 일치율은 보조값이다.

| 출력 경로 | review | overlay |
| --- | --- | --- |
| Native | ![Native review](https://raw.githubusercontent.com/<head-owner>/<head-repo>/<head-sha>/mydocs/pr/assets/issue_<N>_<topic>/native_review_078.png) | ![Native overlay](https://raw.githubusercontent.com/<head-owner>/<head-repo>/<head-sha>/mydocs/pr/assets/issue_<N>_<topic>/native_overlay_078.png) |
| fresh WASM | ![fresh WASM review](https://raw.githubusercontent.com/<head-owner>/<head-repo>/<head-sha>/mydocs/pr/assets/issue_<N>_<topic>/wasm_review_078.png) | ![fresh WASM overlay](https://raw.githubusercontent.com/<head-owner>/<head-repo>/<head-sha>/mydocs/pr/assets/issue_<N>_<topic>/wasm_overlay_078.png) |
~~~

PR 본문 URL은 해당 제출 head를 고정하고, [merge 후 GitHub comment](#github-merge-comment)는 merge commit SHA와
`edwardkim/rhwp`를 고정한다. 두 시점을 섞지 않는다.

## GitHub merge comment

renderer, layout, paint처럼 **문서 비교 결과를 merge 판단 근거로 쓴 PR**의 공식 비교 절차는 이
Visual Sweep 가이드다. merge 후 GitHub comment는 이미지 하나 또는 raw URL만으로 판정하지 않고, 이 절의
절차와 review 문서에 기록한 실제 결과를 함께 가리킨다.

comment에는 다음을 포함한다.

- 이 절의 [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)
  direct link
- review 문서에 기록된 실제 페이지, 후보 수, `pixel match`와 `visual_accuracy_proxy_percent`
- 수치가 자동 일치율 보조값이며 사람의 최종 판정을 대체하지 않는다는 설명
- merge commit SHA에 고정한 representative review PNG

`raw.githubusercontent.com` URL은 GitHub comment에서 PNG를 표시하는 증적 전달 수단일 뿐, 문서 비교 절차의
정본 링크가 아니다. branch URL 대신 asset을 포함한 merge commit SHA를 사용해, 이후 `devel`이 전진해도
comment가 가리키는 증적을 고정한다.

~~~markdown
- 문서 비교: [PDF/SVG visual sweep 가이드](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)를 따름
- 대상: pN, flagged=0/N, pixel match NN.NNNNN%

코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 약 NN.NN%.
높을수록 좋음: 기준 PDF와 rhwp PNG가 더 비슷함
낮을수록 나쁨/검토 필요: 잉크 위치나 형태 차이가 큼
단, 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값입니다

![PR N pN visual review](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/<review>.png)
~~~

## PNG overlay 비교

스크립트는 각 페이지에 대해 `rhwp_png`와 `pdf_png`를 같은 canvas 크기로 padding한 뒤, RGB 채널 차이가
`--pixel-diff-threshold`보다 큰 픽셀을 overlay 이미지로 표시한다. 기본 임계값은 `32`다.

```bash
python3 scripts/visual_sweep.py \
  --hwp /path/to/input.hwpx \
  --pdf /path/to/baseline.pdf \
  --pixel-diff-threshold 32 \
  --out output/visual-one
```

overlay 색상 의미:

| 색상 | 의미 |
|---|---|
| 회색 | 임계값 이하로 거의 같은 픽셀 |
| 빨강 | rhwp 쪽에만 잉크가 있거나 rhwp가 더 많이 그린 후보 |
| 파랑 | PDF 쪽에만 잉크가 있거나 PDF 기준에만 보이는 후보 |
| 주황 | 양쪽 모두 잉크가 있지만 위치/색상 차이가 큰 후보 |
| 연분홍 | 배경/anti-aliasing 계열 차이 후보 |

`overlay/overlay_metrics.json`에는 다음 보조 지표가 기록된다.

| 필드 | 의미 |
|---|---|
| `pixel_match_percent` | 전체 canvas 픽셀 중 임계값 이하로 일치한 비율 |
| `ink_match_percent` | 양쪽 중 하나라도 내용 픽셀인 영역에서 일치한 비율 |
| `visual_accuracy_proxy_percent` | 자동 시각 판정 보조 일치율. 잉크 영역이 있으면 `ink_match_percent`, 없으면 `pixel_match_percent` |
| `tolerant_content_match_percent` | 내용 실루엣의 상대편이 `tolerant_content_match_radius_px` 이웃에 있을 때 일치로 보는 기하 보조값 |
| `diff_bbox` | 차이가 난 픽셀들의 bounding box |
| `mean_abs_channel_delta` | RGB 채널 평균 절대 차이 |
| `max_channel_delta` | 페이지 내 최대 RGB 채널 차이 |

주의: `visual_accuracy_proxy_percent`는 사람이 내린 정답/오답 판정에 대한 실제 정확도가 아니다.
PDF raster와 rhwp raster가 얼마나 비슷한지를 보여주는 자동 보조 지표다. 여백이 넓은 문서는
`pixel_match_percent`가 과하게 높게 나올 수 있으므로 실제 판정에는 `overlay_contact_sheet.png`,
페이지별 `overlay_*.png`, `ink_match_percent`, 기존 `analysis/metrics.json` 후보를 함께 본다.

계산 의미:

- 각 픽셀에서 RGB 채널 최대 차이가 `--pixel-diff-threshold` 이하이면 일치로 본다.
- `pixel_match_percent = 100 * (1 - diff_pixels / total_pixels)` 이다.
- `ink_match_percent = 100 * (1 - ink_diff_pixels / ink_union_pixels)` 이다.
- `visual_accuracy_proxy_percent`는 잉크 영역이 있으면 `ink_match_percent`, 잉크 영역이 없으면
  `pixel_match_percent`를 쓴다.
- `tolerant_content_match_percent`는 기본 2px 이웃까지 허용한 내용 실루엣 일치율이다. 글꼴
  anti-aliasing·sub-pixel rasterization의 프린지를 기하 위치 차이와 분리해 보여 주기 위한 값이며,
  `ink_match_percent`나 `visual_accuracy_proxy_percent`를 대체하지 않는다. 다만 PR review에서는 위
  `PR review 실루엣 gate`에 따라 90% 미만을 재검토 신호이자 보류 조건으로 사용한다.

따라서 이 값은 "자동 시각 판정 정확도"가 아니라 "내용 픽셀 중심 raster 일치율"에 가깝다. 폰트,
anti-aliasing, PDF rasterizer, 전체 위치 이동의 영향을 크게 받으므로, 낮은 값은 우선 검토 신호이지
그 자체로 불합격 판정은 아니다.

### glyph·PUA·제품명 표시 차이도 잡는 방법

기존 `analysis.flagged`는 frame overflow, line/order drift처럼 구조적으로 규칙화한 후보만 집계했다.
현재 sweep은 여기에 render tree의 옛자모·PUA `TextRun` bbox를 PDF/SVG raster에 대조하는
`legacy_glyph_visual_mismatch`도 더한다. 해당 bbox의 잉크가 충분하고 국소 일치율이 80% 이하이면
페이지를 flag하고, `legacy_glyph_visual_candidates`에 text·code point·render-tree/raster bbox·국소
잉크 지표를 남긴다.

그래도 **`flagged=0`은 모든 glyph·글자폭·자간·제품명 convention이 PDF와 같다는 뜻이 아니다.**
이 자동 후보는 옛자모·PUA로 범위를 의도적으로 좁혔으므로, 일반 글꼴·색·자간 차이는 별도 review가
필요하다. 같은 bbox 안에서 `ᄒᆞᆫ글`과 `한글`처럼 다른 glyph를 paint하거나, PUA/글머리표의 의미가
달라질 때 구조 heuristic만으로는 0건일 수 있다.

다음은 합격 임계값이 아니라, 이런 차이를 놓치지 않기 위한 필수 triage다.

1. 대상 페이지의 `overlay_metrics.json`을 `visual_accuracy_proxy_percent` 오름차순으로 정렬한다.
   가장 낮은 페이지와 옛자모·PUA·목록 marker가 있는 페이지는 `flagged` 값과 무관하게 review PNG를
   연다.
2. review의 좌우 비교에서 glyph 모양, 글자폭, baseline, bullet/번호가 다르면 overlay가 구조 flag를
   내지 않아도 **시각 fidelity 후보**로 기록한다.
3. 해당 SVG/render tree의 `TextRun.text`, raw code point, CharShape/font family, 문단 context를
   확인한다. 원문 IR의 보존과 paint-time display projection은 별도 계약이다.
4. PDF text layer 추출이 실패했으면 문자 멀티셋 대조를 "차이 없음"으로 처리하지 않는다. raster
   review와 raw/render-tree 대조만으로 후보를 남기고, text layer 한계를 함께 기록한다.

`legacy_glyph_visual_mismatch`는 불합격을 자동 확정하는 규칙이 아니라 review 우선순위다. `analysis/`
annotated PNG에는 해당 후보 bbox가 보라색으로 표시된다. 한 페이지에 후보가 여러 개면 국소
`ink_match_percent`가 낮은 순으로 최대 20개를 기록한다.

페이지별 보조값의 낮은 순서를 보는 예시는 다음과 같다.

```bash
jq -r '.pages[] | [.page, .visual_accuracy_proxy_percent, .ink_match_percent, .pixel_match_percent] | @tsv' \
  output/task1274/<target>/overlay/overlay_metrics.json \
  | sort -t $'\t' -k2,2n
```

예를 들어 HWP 97 안내문의 `가. ᄒᆞᆫ글 드라이버 사용`은 rhwp render tree에 표준 옛자모가
남아 있고, 기준 PDF에는 제품명 `한글`로 보일 수 있다. 이 경우 HWP3 parser만 바꾸거나 모든
`ᄒᆞᆫ`을 현대화하면 실제 옛한글 문서를 훼손할 수 있다. **문서·CharShape·PDF로 증명된 제품명
context만** paint-time projection 후보로 삼고, 일반 옛한글과 다른 문서는 negative regression으로
보호한다.

## 결과 해석

실행 중 출력 예:

```text
analysis: 2024-09-between20 flagged=1/24 frame=[] red=[] line=[11] column=[11] eq=[] title=[] order=[11] tail=[] question=[]
summary: /path/to/rhwp/output/task1274/summary.json
```

핵심 필드:

| 필드 | 의미 |
|---|---|
| `flagged` | 후보가 감지된 페이지 수 / 전체 분석 페이지 수 |
| `overlay_metrics` | PNG overlay 기반 픽셀/잉크 일치율 요약 |
| `frame` | 편집 frame 밖 overflow 후보 |
| `red` | 빨간 문항 marker drift 후보 |
| `line` | 페이지 전체 line band drift 후보 |
| `column` | 단별 line band drift 후보 |
| `flowcollapse` | 같은 단에서 PDF와 line-band 수와 y 흐름이 함께 크게 달라진 본문 flow 붕괴 후보 |
| `eq` | 수식/본문 겹침 후보 |
| `title` | 문항 제목/본문 겹침 후보 |
| `order` | 줄 순서 겹침 후보 |
| `tail` | render tree 기준 tail overflow 후보 |
| `question` | PDF/rhwp 문항 marker y drift 후보 |
| `glyph` | 옛자모·PUA TextRun의 국소 raster 불일치 후보 |
| `wrap` | fidelity와 같은 Square/Tight/Through 그림↔본문 physical-overlap 또는 edge-clearance-loss 후보 |
| `tablewrap` | 우측 Body table 좌측 strip의 PDF 본문 잉크가 rhwp에서 소실된 non-inline wrap 후보 |

권장 판정 기준:

- `svg_pages == pdf_pages`는 기본 조건이다.
- `overlay_contact_sheet.png`에서 빨강/파랑/주황이 본문 흐름에 집중되면 우선 검토한다.
- `visual_accuracy_proxy_percent`는 자동 일치율 지표일 뿐 최종 시각 판정을 대체하지 않는다.
- `flagged=0`이어도 낮은 `visual_accuracy_proxy_percent` 또는 옛자모·PUA·목록 marker가 있으면
  review/overlay를 반드시 확인한다. glyph·제품명 표시 차이는 이 경로에서 후보가 된다.
- PR의 실제 조판 소비 경로를 먼저 확인한다. 조판 영향 변경은 Visual Sweep·TSV가 필수다.
  조판 영향이 없어도 Visual Sweep을 PR 수용 근거로
  첨부했다면 90% gate를 적용한다. 기준 PDF 재산출처럼 renderer 출력을 주장하지 않는 PR은 review PNG를
  만들지 않고 fixture 원본성·소비 경로만 별도로 검토한다.
- `frame`, `question`, `title`, `tail`, `eq` 후보는 우선 검토 대상이다.
- `tail`은 render tree의 page bbox를 **현재 raster DPI 좌표**로 투영한 뒤, 해당 bbox에
  실제 rhwp 잉크가 있는 TextLine만 세어 만든다. 페이지 밖에만 남은 continuation node나
  ancestor clip으로 보이지 않는 node는 tail 후보가 아니다. 따라서 고 DPI sweep의 `tail`은
  render-tree 논리 좌표만으로는 재현·판정하지 않는다.
- `wrap`은 Square/Tight/Through 그림이 본문 흐름 영역과 outer clearance를 예약해야 하는 계약의 강한
  후보다. annotation의 `candidate_kind`가 `physical_overlap`이면 image/첫·마지막 교차 line bbox를,
  `edge_clearance_loss`이면 image edge와 최소 clearance를 PDF review와 즉시 대조한다. 후자는 HWP
  outer margin 유실로 glyph와 그림 테두리가 맞닿는 결함을 포착한다. 의도된 overlay·zero-margin source와
  render-tree source 정보의 한계가 있으므로 자동 불합격이나 PDF 정답 판정으로 승격하지 않는다.
- `wrap` 판정에 필요한 render tree가 빠지거나 손상된 run은 clean 결과가 아니라 **infrastructure
  failure**다. tree export를 복구한 뒤 다시 실행한다.
- `tablewrap`은 `Table`이 Body의 오른쪽에 있고, 해당 table의 세로 범위에서 Body 좌측 strip의 PDF
  content ink density가 `0.025` 이상인데 rhwp ink가 그 15% 이하일 때만 후보가 된다. 따라서 단순
  우측 정렬 standalone table(양쪽 strip이 비어 있음)과 소폭 font raster 차이는 제외한다. HWPX
  non-inline Square table이 다음 문단 prefix를 소실하는 형상을 빠르게 찾는 용도이며, PDF review로
  wrap 의도를 확인하기 전에는 자동 결함 확정이 아니다.
- `flowcollapse`은 본문이 그림 옆의 비정상적인 세로 열로 분해되는 회귀를 우선 올리는 강한 후보다.
  자동 불합격은 아니지만 `review`와 PDF를 즉시 대조한다.
- `flowcollapse` 계산은 render tree의 **Body bbox**를 우선 frame으로 쓰고, Body table 영역을 양쪽
  raster에서 제외한다. 테두리 없는 페이지의 넓은 table rule이나 cell raster 분할이 본문 line band로
  오인되는 false positive를 막기 위한 것이며, 표 row fragment·owner가 같다는 뜻은 아니다. 표가 관련된
  페이지는 `fidelity_compare` text ledger, table/footer/frame 후보와 3-way review를 별도로 확인한다.
- `line`, `column`, `order` 후보는 실제 시각 차이인지 false positive인지 비교 이미지를 열어 확인한다.
- `table-cell-text-boundary`의 `line_boundary_overflow`는 line bbox 자체의 침범이고,
  `natural_visible_width_risk`는 visible 문자로 끝나는 run의 자연 폭 위험이다. 후자는 PDF와
  최종 SVG glyph 위치에서 실제 선을 넘는지로 확정한다. 중첩 표 text는 외부 셀 후보로 합산하지 않는다.
- `svg-text-band-clip` 후보는 glyph 상·하단의 partial clip만 뜻한다. 같은 줄의 연속 glyph가 여러 행으로
  기록될 수 있으므로 `(page, clip_ids, baseline_y, edges)` 단위로 묶어 review한다. 근사 ink band는
  baseline `-0.8em..+0.2em`이므로 exact font outline이나 raster 판정을 대신하지 않는다.
- 후보가 남아도 메인테이너 SVG/웹/한컴 시각 판정이 통과하면 blocker가 아닐 수 있다.

요약만 빠르게 보기:

```bash
jq -r '.[] | [.key, .svg_pages, .pdf_pages, (.visual_metrics.flagged_page_count // 0), (.visual_metrics.frame_overflow_pages|join(",")), (.visual_metrics.line_band_drift_pages|join(",")), (.visual_metrics.column_line_band_drift_pages|join(",")), (.visual_metrics.column_text_flow_collapse_pages|join(",")), (.visual_metrics.square_wrap_text_overlap_pages|join(",")), (.visual_metrics.right_table_left_strip_text_deficit_pages|join(",")), (.visual_metrics.line_order_overlap_pages|join(",")), (.visual_metrics.question_marker_drift_pages|join(",")), (.visual_metrics.legacy_glyph_visual_pages|join(","))] | @tsv' output/task1274/summary.json
```

## PR에 기록할 때

PR 리뷰/보고서에는 다음을 분리해 적는다.

- 설치/환경 문제로 실행하지 못한 경우: 어떤 CLI가 없는지 명시
- 실행 완료한 경우: target별 페이지 수와 후보 페이지를 표로 기록
- 후보가 남은 경우: 메인테이너 시각 판정과 blocker 여부를 별도로 기록

예:

```markdown
| target | SVG/PDF pages | flagged | frame | line | column | wrap | tablewrap | order | question | glyph |
|---|---:|---:|---|---|---|---|---|---|---|---|
| `2024-09-between20` | 24/24 | 1 | `[]` | `[11]` | `[11]` | `[]` | `[]` | `[11]` | `[]` | `[]` |
```

## 한계

- PDF는 한컴 편집기 직접 시각 판정의 완전한 대체물이 아니다.
- 폰트/anti-aliasing 차이 때문에 line/column/order 후보가 false positive로 남을 수 있다.
- 표의 같은-page geometry·row fragment는 `flowcollapse`만으로 판정하지 않는다. 이 신호는 표 영역을
  의도적으로 mask하므로, PDF text owner 차이와 render tree/table geometry를 함께 대조해야 한다.
- `wrap`은 80px 이상 Square/Tight/Through 이미지와 image 폭의 절반 이상을 가로지르는 Body TextLine
  3행 이상(`physical_overlap`), 또는 image 왼쪽/오른쪽 edge에서 `≤1px`로 맞닿거나 얕게 침범하는 3행
  이상(`edge_clearance_loss`)을 후보화한다. 1–2행·Body 밖 text·PDF와 위치만 다른 경우는 놓칠 수 있어,
  fidelity text/table 원장 및 PDF review의 대체물이 아니다.
- `tablewrap`은 PDF/rhwp raster와 render-tree Table bbox를 함께 요구하므로 PDF가 없는 자체 회귀
  검증에는 적용할 수 없다. 우측 표의 left strip에 의도적인 빈 여백이 있더라도 PDF도 비어 있으면
  후보가 되지 않지만, PDF의 그림·색면처럼 본문 이외 잉크가 strip을 채운 경우는 review에서 제외한다.
- 반대로 glyph·글자폭·PUA/제품명 convention 차이는 구조 후보가 0건이어도 실제 fidelity 결함일 수
  있다. 옛자모·PUA는 `legacy_glyph_visual_mismatch`로 우선 후보화하지만, 낮은 잉크 일치율과
  review의 반복 차이는 raw/IR/paint 경로로 분리한다.
- PDF text layer가 손상되거나 추출기에 실패하면 text 기반 자동 분류를 생략할 수 있다. 이 경우
  raster overlay와 render tree를 사용하되, 문자 멀티셋 무차이를 주장하지 않는다.
- 최종 수용 여부는 자동 sweep + 회귀 테스트 + 메인테이너 시각 판정을 함께 보고 결정한다.

### 기준 PDF의 폰트 환경을 명시할 때

기준 PDF가 원본 문서에 선언된 폰트 대신 다른 폰트로 생성되었다면
`--font-environment <JSON 파일>`로 그 환경을 명시할 수 있다. Native와 `--wasm-pkg` 경로가
동일한 설정을 사용하며, 설정 파일 해시가 달라지면 이전 `--resume` 결과는 재사용하지 않는다.
프로필 선언과 페이지 수만으로 PDF 일치를 판정하지 않는다. 대응 페이지의 내용과 폰트를
직접 확인한다. JSON 형식과 지원 범위는 [폰트 환경 가이드](../font_environment.md)를 따른다.
