---
kind: guide
status: active
canonical: mydocs/manual/verification/visual_verification_governance.md
last_verified: 2026-10-08
---

# 개체 단위 시각/geometry 회귀 하니스 (`object_visual_regression.py`)

page/PI 레벨(`verify_pi_page_vs_hangul.py`)로는 잡히지 않는 **개체(중첩표·그림) 단위** 배치 차이를
rhwp vs 한글(OLE) 로 검출한다. #1718 잔여 under-pagination(개체 배치 누적 차이) 정밀 조사용.

## 무엇을 하나
1. **rhwp 개체 geometry** — `export-render-tree`의 render tree에서 그림과 중첩표를 추출한다.
   - 모든 `Image` 노드(bbox/textWrap 포함)를 수집하므로 최상위 Paper/Page float도 `image`로 검토한다.
   - `Table` 노드는 depth≥1 중첩 개체만 수집한다. 1×1은 그림/도형 프레임(`image`), 그 외는 중첩표(`table`)다.
   - 외곽 RowBreak 컨테이너(depth0, 매 페이지 반복)는 Table 개체에서 제외한다.
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
# 원커맨드 before/after(권장) — devel 을 임시 worktree 로 빌드해 baseline 자동 생성 →
# 현 트리와 대조 → PR 본문용 markdown 요약(ovr_diff.md). 한컴 불필요.
python tools/object_visual_regression.py --preset ovr5 -o output/poc/ovr --diff-against devel

# 한글 대조 + 시각 갤러리 + baseline 저장
python tools/object_visual_regression.py <file.hwp> -o output/poc/ovr --save-baseline

# HWP2024 MCP로 산출한 권위 PDF를 사용한다. COM 변환 없이 PDF를 분석한다.
python tools/object_visual_regression.py <file.hwpx> -o output/poc/ovr \
  --reference-pdf pdf/<hancom-reference>.pdf --rhwp-png

# rhwp 래스터 크롭까지(권장, native-skia 빌드 필요)
cargo build --release --features native-skia
python tools/object_visual_regression.py <file.hwp> -o output/poc/ovr --rhwp-png

# rhwp 버전 간 회귀만(한글 불필요, 빠름) — CI/게이트용
python tools/object_visual_regression.py <file.hwp> -o output/poc/ovr --baseline output/poc/ovr/baseline.json --no-hwp
```

여러 파일을 나열하거나 `--preset ovr5`(KTX/aift/biz_plan 관례 세트; 시험지는 사용자 요청으로 별도 작업에서 재추가)로 일괄
실행할 수 있다. 다중 샘플이면 `-o` 아래 샘플별 서브디렉터리에 산출한다.

## 기존 회귀 검사의 기대값과 이슈 등록 여부

이 하니스의 geometry 차이값은 변화 후보를 찾는 진단 자료다. 그 좌표·차이값을 그대로 실물
문서의 절대 px 회귀 기대값으로 고정하지 않는다. 현재 브랜치에서 전쪽 Native/fresh WASM
90% 이상·독립 PDF 쪽수 일치·직접 판독을 확인한 문서가 실패하면 **쪽수 검사까지 포함**해
독립 기대값을 재검토하고, 잘못된 핀을 내용·개체 소속과 순서·보존·포함·겹침 관계로 수정한다.
정확한 검사는 유지하고 구현을 고친다.

전쪽 Native/fresh WASM 검증과 직접 판독을 마친 **현재 보정 대상 문서**는 기존 회귀가
통과 중이어도 그 문서를 사용하는 절대 px 좌표·고정 영역 검사를 확인하고 관계 검사로
바꾼다. 대상 문서의 원본 경로를 따라 검사·공용 corpus 참조를 확인하며 다른 문서로 범위를
확장하지 않는다. 기존 검사 함수 안에서 쪽 소속·내용 순서·누락·중복·포함·겹침을 보호하고,
독립 PDF 쪽수 대조에는 대응 내용 보존을 함께 검사한다. 원본 글자모양·셀 속성에서 계산한
크기와 실제 줄·개체의 포함 또는 비율 관계는 유지할 수 있다. 소스 단위를 px로 환산했다는
이유만으로 이 원본 계약을 제거하지 않는다. 변경 보고에는 기존에 이미 관계 검사였던 범위,
실제로 교체한 핀, 새로 보완한 내용 관계와 재사용한 시각 증거를 구분해 기록한다.

미달 문서는 같은 원본·같은 미해결 범위의 기존 이슈 등록 여부를 먼저 확인한다. 등록된 범위는
해당 회귀와 수집 참조를 제거한 뒤 남은 검증을 계속하고 원본·PDF는 보존한다. 관련 이슈가
없을 때만 별도 등록한다. 개인의 작업 이슈를 공통 이관 대상으로 자동 지정하지 않는다. 상세 조건은
[기존 기대값 재검토](../pr_review/visual_fixture_evidence.md#기존-회귀-테스트의-기대값-재검토)와
[기존 이슈 확인·처리](../pr_review/visual_fixture_evidence.md#90-미만-회귀-대상의-별도-이슈-이관)를 따른다.

## 회귀 게이트 사용

### 원커맨드 (`--diff-against <ref>`)
1. `--diff-against devel` — ref 를 임시 worktree 로 체크아웃해 release 빌드(최초 1회,
   이후 `target/ovr-baseline/target_<sha>` 캐시 재사용) → 샘플별 baseline 자동 생성.
2. 현 트리를 `cargo build --release` 후 대조 → 개체 page 이동/크기 변경(±`--tol`px)·신규/소실 검출.
3. `ovr_diff.md`(markdown 표)를 출력 — PR 본문에 그대로 붙여넣는 용도.
4. worktree 는 성공/실패 무관 항상 제거(잔재 없음). 종료코드: 0 무회귀 / 1 회귀 / 2 실행 실패.

### 수동 3단계 (기존 흐름, 하위 호환)
1. 기준 커밋에서 `--save-baseline` 으로 `baseline.json` 확보(개체 geometry 스냅샷).
2. 변경 후 `--baseline baseline.json --no-hwp` 로 재실행 → 개체 page 이동/크기 변경(±`--tol`px) 검출.
3. 종료코드 1 = 회귀 존재(개체 이동/리사이즈). page/PI 게이트와 상보적으로 개체 레벨을 커버.

## 요구
- rhwp release 바이너리 (`--rhwp-png` 시 `--features native-skia`).
- `--no-hwp` 아니면: PyMuPDF(fitz) + Pillow. `--reference-pdf`를 지정하지 않으면 추가로 Windows + 한컴오피스 + pyhwpx가 필요하다.

## 한계
- render-tree의 `Image` 노드는 수집하지만, 인라인 그림이 render tree에 노출되지 않는 형식은 미포착 가능하다.
- 내용 기반 매칭은 셀 텍스트가 충분할 때 정확(표). 텍스트 적은/없는 개체(그림)는 크기 폴백이라 근사 —
  갤러리 육안 확인 병행. 표가 페이지 경계로 분할되면 rhwp(전체/조각)와 한글(조각) 리포팅 단위가 달라
  높이 delta 는 조각 경계에서 직접 비교가 어려울 수 있다(페이지·내용 매칭은 정확).
- 한글 COM 배치 크래시 시 해당 파일 한글측 생략(rhwp-only 진행).
- **문서 유형 의존**: 두 엔진이 같은 표를 검출할 때 매칭이 유효(테두리 있는 데이터 표, 예: 기술기준
  중첩표). 폼/기안문은 한글 `find_tables` 가 무테 결재란을 미검출하고 rhwp 와 검출 부분집합이 달라
  겹침이 낮다 — 이때 내용 매칭은 오매칭 대신 정직하게 "매칭 없음"으로 표기(크기기반의 오매칭 회피).
- 적용 대상 문서에 nested(depth≥1) 개체가 없으면(단일 최상위 표 별표 등) rhwp 개체 0 — 페이지/PI
  게이트(`verify_pi_page_vs_hangul.py`)로 커버.
