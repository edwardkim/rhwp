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
pip install pypdfium2 pillow          # PDF 렌더 + 픽셀 diff
# Chrome/Chromium 설치 필요 (SVG → PNG 캡처)
```

실행 파일은 플랫폼에 맞춰 자동 탐색한다.

- `rhwp`: `target/release-test/rhwp[.exe]` → `target/release/rhwp[.exe]` → `PATH`
- Windows Chrome: `PATH` → `Program Files`/`LocalAppData`
- macOS Chrome: `PATH` → `/Applications/Google Chrome.app`/`Chromium.app`
- Linux Chrome: `google-chrome`, `google-chrome-stable`, `chromium`, `chromium-browser`

자동 탐색이 맞지 않으면 `RHWP_BIN`과 `CHROME_BIN`으로 각각 실행 파일을 지정한다.

Linux 예시:

```bash
# rhwp와 google-chrome/chromium이 PATH에 있으면 환경변수 없이 실행
python3 tools/fidelity_compare/fidelity_compare.py plan 0 9 \
  --out-dir /tmp/rhwp-fidelity-plan

# 저장소 밖 빌드·배포 경로를 쓸 때만 명시적으로 지정
RHWP_BIN=/opt/rhwp/bin/rhwp \
CHROME_BIN=/usr/bin/google-chrome \
python3 tools/fidelity_compare/fidelity_compare.py plan 0 9 \
  --out-dir /tmp/rhwp-fidelity-plan
```

## 사용

```bash
python tools/fidelity_compare/fidelity_compare.py <키> <시작쪽> <끝쪽>   # 0 기준, 끝쪽 포함
# 예: 업무계획 전체 35쪽
python tools/fidelity_compare/fidelity_compare.py plan 0 34

# 저장소 밖에 산출해 worktree를 깨끗하게 유지
python tools/fidelity_compare/fidelity_compare.py plan 0 9 \
  --out-dir /tmp/rhwp-fidelity-plan
```

`--out-dir`은 지정한 디렉터리 자체를 산출 루트로 쓴다. 생략하면
`output/fidelity/<키>/`를 쓴다. 주요 산출물은 다음과 같다.

- `cmp-pNNN.png`: 기준 PDF와 rhwp 렌더의 쪽별 비교 시트
- `report.tsv`: 픽셀 diff% 랭킹
- `text-report.tsv`: 기준 PDF 텍스트층에만 있는 문자와 SVG에만 있는 문자 수·코드포인트
- `provenance.tsv`: 원본·기준 PDF 경로와 기준 등급

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
