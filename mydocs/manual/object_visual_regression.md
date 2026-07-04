# 개체 단위 시각/geometry 회귀 하니스 (`object_visual_regression.py`)

page/PI 레벨(`verify_pi_page_vs_hangul.py`)로는 잡히지 않는 **개체(중첩표·그림) 단위** 배치 차이를
rhwp vs 한글(OLE) 로 검출한다. #1718 잔여 under-pagination(개체 배치 누적 차이) 정밀 조사용.

## 무엇을 하나
1. **rhwp 개체 geometry** — `export-render-tree` 의 render tree 에서 depth≥1 중첩 개체를 추출한다.
   - `Table` 노드(pi/ci/rows/cols/bbox). 1×1 = 그림/도형 프레임(`image`), 그 외 = 중첩표(`table`).
   - 외곽 RowBreak 컨테이너(depth0, 매 페이지 반복)는 제외.
2. **한글 권위 렌더** — COM→PDF→PyMuPDF(fitz) 로 페이지를 96 DPI 래스터 + 이미지 bbox 추출.
3. **rhwp 래스터**(옵션 `--rhwp-png`) — `export-png`(native-skia) 로 페이지 PNG.
4. **개체 매칭** — **내용 기반**(개체 셀 텍스트의 문자 3-gram Jaccard) 우선 매칭. rhwp 는 render-tree
   TextRun, 한글은 `find_tables().extract()` 셀 텍스트로 서명을 만들어 Jaccard≥0.12 로 짝짓는다.
   텍스트 없는 개체(그림)는 크기(면적+종횡비) 기반 폴백. 페이지 오프셋(−N쪽) 무관하게 내용으로 정합
   → 전폭 표들이 크기가 우연히 겹쳐도 정확히 구분(예: 표7 70×5 ↔ 한글 표7, J=0.88).
5. **산출**
   - `objects.tsv` — 개체별 rhwp/한글 page·bbox·delta.
   - `gallery.html` — 개체별 rhwp↔한글 side-by-side 크롭(작업지시자 시각 판정).
   - `baseline.json` — rhwp 개체 geometry 스냅샷.

## 좌표계
render-tree bbox 는 96 DPI px. 한글 PDF(pt, 72 DPI)는 `96/72` 배율로 래스터하여 정합.

## 사용
```bash
# 한글 대조 + 시각 갤러리 + baseline 저장
python tools/object_visual_regression.py <file.hwp> -o output/poc/ovr --save-baseline

# rhwp 래스터 크롭까지(권장, native-skia 빌드 필요)
cargo build --release --features native-skia
python tools/object_visual_regression.py <file.hwp> -o output/poc/ovr --rhwp-png

# rhwp 버전 간 회귀만(한글 불필요, 빠름) — CI/게이트용
python tools/object_visual_regression.py <file.hwp> -o output/poc/ovr --baseline output/poc/ovr/baseline.json --no-hwp
```

## 회귀 게이트 사용
1. 기준 커밋에서 `--save-baseline` 으로 `baseline.json` 확보(개체 geometry 스냅샷).
2. 변경 후 `--baseline baseline.json --no-hwp` 로 재실행 → 개체 page 이동/크기 변경(±`--tol`px) 검출.
3. 종료코드 1 = 회귀 존재(개체 이동/리사이즈). page/PI 게이트와 상보적으로 개체 레벨을 커버.

## 요구
- rhwp release 바이너리 (`--rhwp-png` 시 `--features native-skia`).
- `--no-hwp` 아니면: Windows + 한컴오피스 + pyhwpx + PyMuPDF(fitz) + Pillow.

## 한계
- render-tree 는 표/프레임 위주 — 프레임 없는 인라인 그림은 미포착 가능(한글 이미지 bbox 로 보완).
- 내용 기반 매칭은 셀 텍스트가 충분할 때 정확(표). 텍스트 적은/없는 개체(그림)는 크기 폴백이라 근사 —
  갤러리 육안 확인 병행. 표가 페이지 경계로 분할되면 rhwp(전체/조각)와 한글(조각) 리포팅 단위가 달라
  높이 delta 는 조각 경계에서 직접 비교가 어려울 수 있다(페이지·내용 매칭은 정확).
- 한글 COM 배치 크래시 시 해당 파일 한글측 생략(rhwp-only 진행).
- **문서 유형 의존**: 두 엔진이 같은 표를 검출할 때 매칭이 유효(테두리 있는 데이터 표, 예: 기술기준
  중첩표). 폼/기안문은 한글 `find_tables` 가 무테 결재란을 미검출하고 rhwp 와 검출 부분집합이 달라
  겹침이 낮다 — 이때 내용 매칭은 오매칭 대신 정직하게 "매칭 없음"으로 표기(크기기반의 오매칭 회피).
- 적용 대상 문서에 nested(depth≥1) 개체가 없으면(단일 최상위 표 별표 등) rhwp 개체 0 — 페이지/PI
  게이트(`verify_pi_page_vs_hangul.py`)로 커버.
