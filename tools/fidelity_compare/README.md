# fidelity_compare — 기록된 한컴 기준 PDF 페이지별 대규모 비교 하네스 (#3389)

`render-diff`(자기 일관성 기하 게이트)를 보완하는 **한컴 출력 기준 PDF 대조** 도구다.
기준 PDF와 rhwp `export-svg` 렌더를 페이지별로 나란히 시트로 만들고,
픽셀 diff% 랭킹으로 **최악 페이지부터 사람이 감사**하게 한다. 동시에 기준 PDF
텍스트층과 SVG `<text>`의 쪽별 문자 멀티셋을 대조해 폰트 대체 잡음과 무관한
소실·과잉·치환 후보를 `text-report.tsv`에 남긴다. 이 루프가 실제로
#3385(PUA 원문자 CharOverlap tofu)를 찾아냈다.

한컴 출력은 도구·버전·출력 경로·폰트에 따라 달라질 수 있다. 재현 기록에는 해당 환경과
원본/기준 PDF의 provenance를 남기며, diff%는 보편적 절대 판정이 아니라 후보 검출 근거로 쓴다.
최종 시각 판정은 [시각 검증 거버넌스](../../mydocs/manual/verification/visual_verification_governance.md)를
따른다.

## 요구사항

```bash
# 저장소 루트에서 최초 1회
python3.12 -m venv venv
venv/bin/python -m pip install pypdf pypdfium2 pillow
# Chrome/Chromium 설치 필요 (SVG → PNG 캡처)
```

저장소 로컬 `venv/`의 설치·Git 제외 계약은
[개발 환경 가이드](../../mydocs/manual/dev_environment_guide.md)를 따른다. 시스템 Python에
직접 설치하거나 `--break-system-packages`를 사용하지 않는다. 아래 POSIX 예시는 저장소 루트의
`venv/bin/python`을 사용하며 Windows에서는 `venv\\Scripts\\python.exe`로 바꾼다.

`--text-only`는 `pypdf`만 필요하며 Chrome과 `pypdfium2`를 요구하지 않는다.

실행 파일은 플랫폼에 맞춰 자동 탐색한다.

- `rhwp`: `target/release-test/rhwp[.exe]` → `target/release/rhwp[.exe]` → `PATH`
- Windows Chrome: `PATH` → `Program Files`/`LocalAppData`
- macOS Chrome: `PATH` → `/Applications/Google Chrome.app`/`Chromium.app`
- Linux Chrome: `google-chrome`, `google-chrome-stable`, `chromium`, `chromium-browser`

자동 탐색이 맞지 않으면 `RHWP_BIN`과 `CHROME_BIN`으로 각각 실행 파일을 지정한다.

Linux 예시:

```bash
# rhwp와 google-chrome/chromium이 PATH에 있으면 환경변수 없이 실행
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 9 \
  --out-dir /tmp/rhwp-fidelity-plan

# 저장소 밖 빌드·배포 경로를 쓸 때만 명시적으로 지정
RHWP_BIN=/opt/rhwp/bin/rhwp \
CHROME_BIN=/usr/bin/google-chrome \
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 9 \
  --out-dir /tmp/rhwp-fidelity-plan
```

## 사용

```bash
venv/bin/python tools/fidelity_compare/fidelity_compare.py <키> <시작쪽> <끝쪽>   # 0 기준, 끝쪽 포함
# 예: 업무계획 전체 35쪽
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 34

# 저장소 밖에 산출해 worktree를 깨끗하게 유지
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 9 \
  --out-dir /tmp/rhwp-fidelity-plan

# REG에 없는 HWP/PDF 쌍: 215쪽 첫 후보 수집에는 PNG/Chrome을 생략하고 SVG를 한 번만 전수 생성
RHWP_BIN=target/release-test/rhwp \
venv/bin/python tools/fidelity_compare/fidelity_compare.py 0 214 \
  --source 'samples/입력.hwp' \
  --reference-pdf 'pdf/한컴-기준.pdf' \
  --label issue-3738-hwp \
  --reference-grade '한컴 2020 기준 PDF' \
  --text-only --export-all-svg --layout-ledger \
  --out-dir /tmp/rhwp-fidelity-issue-3738
```

`--out-dir`은 지정한 디렉터리 자체를 산출 루트로 쓴다. 생략하면
`output/fidelity/<키>/`를 쓴다. 주요 산출물은 다음과 같다.

- `cmp-pNNN.png`: 기준 PDF와 rhwp 렌더의 쪽별 비교 시트
- `report.tsv`: 픽셀 diff% 랭킹
- `text-report.tsv`: 기준 PDF 텍스트층에만 있는 문자와 SVG에만 있는 문자 수·코드포인트
- `svg-glyph-risk-report.tsv`: rhwp SVG의 raw PUA/U+FFFD. 한컴 전용 glyph가 공개
  글꼴에서 두부(□)로 보일 수 있는 독립 후보이며, PDF 텍스트 추출 품질과 무관하게 확인한다.
- `text-owner-shift-candidates.tsv`: 인접한 두 쪽에서 SVG-only와 PDF-only 문자가 크게 상호 일치한
  page-owner 이동 후보. `rhwp_earlier_than_reference`/`rhwp_later_than_reference` 방향을 기록하며,
  PDF visual owner 대조 전에는 결함 판정이 아니다.
- `text-owner-sequence-candidates.tsv`: 문자 Counter가 다른 본문/각주와 상쇄해 놓칠 수 있는 경우를
  보완한다. NFC·공백 정규화 뒤 한 쪽에서 사라진 16자 이상 **순서 보존** 문자열이 바로 다음 rhwp/PDF
  쪽에만 있으면 같은 owner 방향 후보로 기록한다. URL·citation·긴 각주 이동에는 강하지만, 최종 layout
  판정은 아니다.
- `page-boundary-fidelity-candidates.tsv`: 위의 인접 text owner 신호를 한 행으로 결합한 review 우선순위
  원장이다. 8자 이상인 짧은 caption/label 이동도 유지하며, 같은 `(pi, ci)` 표 조각이 인접 쪽에 이어지고
  PDF↔SVG owner 이동까지 있으면 `table_fragment_text_owner_drift`로 승격한다. 즉 p81→p82처럼 표의
  첫 줄이 다음 쪽에 중복되는 경우를 여러 원장을 교차해 읽지 않아도 바로 조사할 수 있다. PDF visual
  대조 전에는 여전히 candidate다.
- `visible-text-excess-candidates.tsv`: raw SVG text 원장이 ancestor clip 밖의 숨은 이전 표 조각까지
  세는 한계를 보완한다. PDF 본문이 거의 모두 존재하면서, 실제 body/cell clip 안에서 보이는 rhwp text가
  48자 이상 과잉이면 page-owner 조기 배치·중복 paint 후보로 기록한다. clip/폰트/추출기 차이를 완전히
  판별하지 못하므로 기준 PDF review 없이 결함으로 확정하지 않는다.
- `float-owner-shift-candidates.tsv`: `rhwp_earlier_than_reference` 본문 owner 이동과 바로 다음
  페이지 상단 25% 안의 substantial Body `TopAndBottom`/`Square`/`Tight`/`Through` 그림을 한 행으로
  묶는다. 그림 자체만으로는 후보가 되지 않으며, PDF↔SVG text owner 차이가 먼저 있어야 한다.
- `page-count-ledger.tsv`: 기준 PDF, `--export-all-svg`의 전체 SVG, `--layout-ledger`의 전체 render tree
  쪽수를 분리 기록한다. 페이지 수 차이는 전역 page-break 보정의 근거가 아니라 individual owner 조사 후보를
  여는 신호다.
- `provenance.tsv`: 원본·기준 PDF 경로와 기준 등급
- `run-state.tsv`: requested/completed/missing 페이지와 완료 여부. 누락이 있으면 종료 코드도 0이 아니다.
- `svg/export-svg-manifest.json`: `--export-all-svg`가 보관한 rhwp SVG 매니페스트
- `layout-candidates.tsv`: `--layout-ledger`가 기록한 body↔각주 TextLine, 표↔footer, 표/그림 page-frame 밖,
  Square/Tight/Through 그림을 3행 이상 침범한 본문 후보
- `table-fragment-candidates.tsv`: 같은 source `(pi, ci)`의 Body `Table`이 인접 물리 쪽에 다시 나온 경우,
  표↔footer·page frame 후보, 또는 쪽 하단 표와 24자 이상 PDF↔SVG text delta가 함께 있는 경우를 한 행에
  묶는다. rows/cols·각 쪽 bbox·하단 여백·text delta를 남기지만 **PDF table row owner나 올바른 표 분할을
  판정하지 않는 candidate**다.
- `svg-table-border-clip-candidates.tsv`: render tree의 Table 외곽 vertical edge와 SVG의 실제 `<line>`을
  연결해, 그 선이 Body/TableCell clip에 의해 가시 폭 20% 이하로 잘린 경우를 남긴다. 즉 선이 **생성은 됐지만
  최종 SVG/Canvas paint에서는 사라진** 외곽선 결함을 빠른 text-only pass에서도 후보화한다. source에 원래
  테두리가 없을 수 있으므로 PDF 시각 대조 전에는 결함 확정이 아닌 candidate다.
- `svg-table-horizontal-border-clip-candidates.tsv`: 같은 방식으로 Table의 direct 가로 `Line`을
  effective clip의 상·하단과 대조한다. stroke 높이의 20% 이상이 잘리고, 같은 table frame의 paint-safe
  sibling이 없을 때만 기록하므로 continuation source의 오래된 off-page 선은 중복 경보하지 않는다.
- `table-cell-text-overlap-candidates.tsv`: 한 `TableCell`이 소유한 실제 paint `TextLine` 둘이
  수직 band와 가로 영역을 함께 크게 공유하면 기록한다. PDF/SVG text가 모두 존재해도 p2처럼 문단이
  같은 좌표에 중복 paint되는 결함을 찾기 위한 구조 후보이며, 의도적 도형 text layer는 PDF 시각 대조로
  제외한다.

`--source`, `--reference-pdf`, `--label`을 모두 지정하면 등록 fixture 대신 임의의 HWP/HWPX와 기준 PDF를
비교한다. 이 direct-pair 형식의 positional은 `<시작쪽> <끝쪽>`뿐이다. 기존 등록 fixture 형식
`<키> <시작쪽> <끝쪽>`은 그대로 유지된다.

`--text-only`는 Chrome·PNG·비교 시트를 만들지 않는다. 기준 PDF text와 SVG `<text>`만 비교하므로
각주/본문/caption의 페이지 owner 이동·누락 후보를 빠르게 전수 수집하는 첫 단계에 적합하다.
동시에 `svg-glyph-risk-report.tsv`로 raw PUA와 U+FFFD를 전수 수집하므로, PDF가 해당 한컴
전용 글꼴을 추출하지 못하더라도 두부 문자 후보는 독립적으로 검출한다.
`text-owner-shift-candidates.tsv`는 인접 쪽의 상호 text difference를 묶어, pN에 너무 이르게 나온
각주가 기준 PDF에서는 pN+1에 있는 경우처럼 page-owner 후보를 바로 보인다.
`text-owner-sequence-candidates.tsv`는 p52→p53처럼 다른 본문 문자와 Counter가 상쇄되는 이동도
순서 보존 URL/citation 문자열로 보완한다. 사진 위치,
같은 문자 수의 줄바꿈/overlap, PDF 기준 표 행 owner는 검출할 수 없다. 다만
`table-fragment-candidates.tsv`는 같은 `(pi, ci)`의 인접 쪽 fragment와 footer/frame·하단 text-delta 신호를
우선순위 후보로 묶는다. `text-report.tsv` 상위 페이지와 `export-svg --json`의 `overflowCellLines` 및 bbox
ledger를 합친 뒤에만 pixel diff와 visual sweep으로 확정한다.

`page-boundary-fidelity-candidates.tsv`는 이 세 원장의 **교집합을 다시 계산하지 않도록** 만든 단일
경계 queue다. 8자 이상 reciprocal owner 이동은 `text_owner_shift`로, 동일 source table fragment까지
만나면 `table_fragment_text_owner_drift`로 표시한다. 이 파일은 전수 `--text-only --export-all-svg
--layout-ledger` sweep 뒤 PDF 시각 대조할 경계를 빠르게 고르는 용도이며, 자동 merge gate나 결함 확정
근거로 쓰지 않는다.

`visible-text-excess-candidates.tsv`는 이 raw SVG 경로와 별도로 clip 교집합을 통과한 baseline band만
비교한다. 따라서 off-page/완전 clip된 이전 표 조각 때문에 현재 쪽 SVG-only 문자가 부풀어 owner 이동을
놓치는 경우를 줄인다. 반대로 회전·복잡한 transform·PDF text 추출 자체의 누락은 보수적으로 포함하거나
후보만 남기므로, 이것도 hard failure가 아니라 PDF 대조 대상으로 해석한다.

`--layout-ledger`를 함께 주면 `float-owner-shift-candidates.tsv`도 쓴다. 이는 generic owner
candidate와 successor-page의 상단 Body float를 결합해, 그림 앞 문단의 줄바꿈이 한 페이지 이르게
확정된 p118→p119 같은 경계를 바로 triage한다. 그림이 없는 일반 owner shift 또는 페이지 하단의
무관한 그림은 이 파일에 넣지 않는다.

`--export-all-svg`는 지정 범위와 관계없이 `export-svg`를 한 번 실행해 SVG cache를 채운다. 긴 문서의
전수 text-only pass에서 페이지마다 rhwp 프로세스를 재기동하지 않기 위한 선택지다. 이후 같은 `--out-dir`에
대해 후보 범위만 pixel 비교하면 기존 SVG를 재사용한다.

`--layout-ledger`는 전체 render tree를 한 번 export하므로, 선택 page만 text compare하더라도
`page-count-ledger.tsv`에 render tree 전체 페이지 수를 남긴다. `--export-all-svg`를 함께 주면 SVG 전체 쪽수도
기록한다. 선택 page SVG cache 수는 partial run과 stale cache를 구분할 수 없어서 전체 수로 가장하지 않는다.

`--layout-ledger`는 `export-render-tree`를 한 번 실행해 `layout-candidates.tsv`,
`table-fragment-candidates.tsv`, `svg-table-border-clip-candidates.tsv`,
`svg-table-horizontal-border-clip-candidates.tsv`,
`float-owner-shift-candidates.tsv`를 만든다.
`body_footnote_lines`는 Body `TextLine`의 하단이
`FootnoteArea` 상단보다 1px 이상 아래인 경우, `table_footer`는 Body 표의 하단이 Footer 상단보다 1px 이상 아래인
경우다. `*_outside_frame`은 Body 표/그림이 page frame 밖에 나간 경우다. 표 fragment ledger는 source `(pi, ci)`가
인접 render-tree 쪽에 연속한 것, 표/footer·frame 충돌, 또는 page 높이의 하단 15%에 걸친 표와 24자 이상
PDF↔SVG text delta를 함께 기록한다. 이것은 rhwp 쪽의 source-table 연속성 및 위험 신호일 뿐 PDF의 행 owner나
올바른 분할을 판정하지 않는다. `square_wrap_text_overlap`은 Square/Tight/Through 그림에 대해 두 종류의 Body
`TextLine` 후보를 센다. 그림 물리 box를 폭의 절반 이상 가로지르는 3행 이상은 `physical_overlap`이고, 그림 바로
왼쪽/오른쪽의 3행 이상이 edge에서 `≤1px`로 맞닿거나 얕게 침범하면 `edge_clearance_loss`다. 후자는 HWP outer
margin 유실처럼 glyph가 그림 테두리와 접촉하는 결함을 빠르게 후보화한다. BehindText/InFrontOfText 그림은 의도된
overlay일 수 있어 제외한다. stroke 반올림과 zero-margin source도 후보가 될 수 있으므로, 0이 아닌 값은 곧바로 결함이
아니라 PDF visual review 대상으로 해석한다.

`svg-table-border-clip-candidates.tsv`는 page raster diff와 별개로, SVG가 emit한 Table outer vertical
border의 stroke interval이 ancestor `body-clip-*` 또는 `cell-clip-*`과 만나 가시 폭 20% 이하가 된 경우를
찾는다. line이 실제 Table의 direct border node와 일치해야 하므로 임의의 shape line을 표 결함으로 분류하지
않는다. 이 구조 신호는 p4처럼 큰 표의 우측선 한 면만 사라져도 픽셀 diff 순위·text ledger가 놓치는 경우를
보완하지만, PDF가 의도적으로 해당 border를 생략한 source를 구분하지 못하므로 반드시 review PNG로 확정한다.

`svg-table-horizontal-border-clip-candidates.tsv`는 이 검사를 물리 페이지 경계의 가로 frame에
대칭 적용한다. 표가 clip을 가로지르며 direct 가로선을 냈는데 stroke의 가시 높이가 80% 미만이면 후보로
남긴다. 단, 같은 표가 해당 clip 안쪽에 완전한 frame을 이미 냈다면 원래 source line의 off-page 잔존은
후보에서 제외한다. 따라서 p9–p14처럼 표의 상·하단 선이 반폭으로 잘리는 결함을 text-only pass에서도
후보화할 수 있으며, 최종 판정은 기준 PDF review PNG로 한다.

`scripts/visual_sweep.py`는 자신의 render tree 분석에서 이 Square/Tight/Through 후보 함수를
재사용해 `square_wrap_text_overlap` flag와 annotation을 남긴다. 따라서 sweep의 `flagged=0`이 이 특정
기하 후보를 덮어쓰지는 않는다. 다만 text owner, table fragment, page-count는 sweep이 재계산하지 않으므로
실물 PDF 대조에서는 이 도구의 전체 ledger를 먼저 보존한다. bridge에 필요한 render tree가 없거나
손상되면 sweep은 후보 0으로 fail-open하지 않고 run을 실패시킨다.

단계별 확장(10쪽 → 전수 → 고난도 문서)으로 돌리고, 픽셀 랭킹과 문자 멀티셋 격차를
교차해 후보를 좁힌다. **랭킹 상위 페이지의 시트를 눈으로 감사**한 뒤 실질 결함만
이슈로 승격한다. 문자 멀티셋도 후보 검출용이다. PDF 텍스트층에 없는 path 글리프,
숨김 텍스트, 추출기 문자 매핑 차이가 있으므로 최종 시각 판정을 대신하지 않는다.

## 등록 쌍과 기준 등급

`REG`는 한글 경로 인코딩·NFC/NFD 함정을 피하려고 ASCII 글롭을 사용한다. `pdf/` 아래의
버전 접미사 PDF는 저장소의 장기 기준 자료이며, `samples/` 동반 PDF는 입력과 가까이 둔
참고 사본이다. 후자는 도구·버전·출처를 별도로 확인하기 전에는 최종 기준으로 승격하지 않는다.

| 키 | 원본 | REG가 고르는 PDF | 등급 | 난도 특성 |
|---|---|---|---|---|
| plan | `samples/2022* *.hwp` | `pdf/2022* *-2022.pdf` | 한컴 2022 기준 PDF | 보고서 — 표·도해·강조 혼합 |
| manual | `samples/2025 *.hwpx` | `pdf/2025 *-2024.pdf` | 한컴 2024 기준 PDF | 장문 편람 |
| bunjang | `samples/21868765*.hwp` | `samples/21868765*.pdf` | 참고 PDF — 버전·provenance 별도 확인 | 표 중심 |
| korexam | `samples/21_*.hwp` | `pdf/21_*-2022.pdf` | 한컴 2022 기준 PDF | 법학적성시험 언어이해 15쪽, **A3**, 2단 조판 |
| math | `samples/exam_math.hwp` | `pdf/exam_math-2022.pdf` | 한컴 2022 기준 PDF | 수학 시험지 20쪽, **수식** |
| eng | `samples/exam_eng.hwp` | `pdf/exam_eng-2022.pdf` | 한컴 2022 기준 PDF | 영어 시험지 8쪽, 라틴 혼합 |

## 실측 기록 (2026-07-26)

- 업무계획 35쪽 전수: 구조·내용·줄바꿈 위치까지 대체로 동일. 최악 페이지 감사에서
  **#3385 발견** (PUA 원문자 U+F02B1~F02C4 가 CharOverlap 문맥에서 tofu).
- math 20쪽: diff 6~11% — 수식 렌더 정합이 강함을 실측.
- korexam 15쪽(A3): 2단·헤더·지문 박스·30문항 구조 재현. 잔여 = 본문 자간/글자폭
  미세 확대로 단 내 줄바꿈이 밀리는 부류 — 폰트 폴백 메트릭 의심.
  `RHWP_FONT_PATH_DIR=<폰트 폴더>` 로 폰트를 고정해 재측정하는 것이 다음 실험.

## 함정 노트 (재현 시 시간 절약)

- SVG 캡처 창은 **SVG 판형을 읽어 자동 맞춤**한다 — 고정 창은 A3 문서를 크롭해
  가짜 diff 를 만든다 (초기 버전의 실수).
- diff% 는 랭킹용이다. 자간 미세 차가 픽셀로 누적되므로 절대값이 아니라
  **순위 + 사람 감사**로 쓴다.
- `text-report.tsv`는 공백과 문자 순서를 무시하고 NFC 정규화한 문자 멀티셋을 비교한다.
  `reference_only`은 소실 후보, `svg_only`는 과잉 후보이며 둘이 함께 나타나면 치환 후보로 본다.
- 배경 셸에서 한글 argv/경로는 cp949 로 깨질 수 있어 키·글롭만 쓴다.
- Chrome 캡처는 실패 시 한 번 재시도하고 각 실패의 exit code와 stderr를 표면화한다.
