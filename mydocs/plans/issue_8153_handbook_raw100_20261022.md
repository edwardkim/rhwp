# Issue #8153 — 편람 전쪽 raw 100%와 중요 위치 회귀 검사

- 이슈: https://github.com/edwardkim/rhwp/issues/8153
- 작업 브랜치: `task/8153-handbook-raw100`
- 시작 base: `f7abae4a8e70c062c378db06ad64d2d2eacb88cd` — PR #8155 머지 및 후속 처리 완료 후 최신 devel.
- 대상: `samples/2025 행정업무운영 편람(최종).{hwp,hwpx}`, 각각 383쪽.
- 독립 기준: `pdf/2025 행정업무운영 편람(최종)-{hwp,hwpx}-kopub-2024.pdf`.
- 목표: 두 원본 전쪽 Native/fresh WASM `silhouette_raw_match_percent` 100% 및 각 페이지 중요 영역의 독립 Print 기준 ±2px 회귀 검사.

## 기존 해결과 재사용 근거

#7751·#7752의 90% 이상 개선은 PR #8148에서 완료됐다. #3930 → PR #4058의 저장·바탕쪽 소유 계약,
#3820 → PR #4517의 도장 배치 계약을 유지한다. PR #8155는 공통 중첩 overlay 가로 앵커와 바깥여백 소비를
보정하고 144쪽을 양쪽 raw 98% 이상으로 개선했다. #8151은 종료했으며 전쪽 100% 완료로 해석하지 않는다.

PR #8155의 code/trailing CI·머지·후속 처리를 완료한 뒤 새 작업을 시작했다.
기존 전쪽 TSV·PNG·render tree와 실제 producer를 재사용한다. 같은 결과를 다시 산출한 값이 아니다.
runtime `d81b8b4410779017d50a8891caa6a83ebec994d6`와 새 base의 실행 소스가 같음을 확인했다.
원본·Print SHA와 실제 공급 face는 기존 [입력·producer 원장](../pr/assets/issue_8151/README.md)을 따른다.
재사용 캐시는 저장소 밖 `/home/tsjang/.cache/rhwp/visual-sweep/d81b8b4410779017d50a8891caa6a83ebec994d6/`에
원본별·backend별 PNG·tree·TSV·Print raster·binary/WASM과 해시 manifest를 보존했다.

| 입력 | backend | 쪽수 | raw 100% | 100% 미만 | 최저 raw |
| --- | --- | ---: | ---: | ---: | ---: |
| HWP | Native | 383 | 74 | 309 | 92.49647% |
| HWP | fresh WASM | 383 | 87 | 296 | 92.51694% |
| HWPX | Native | 383 | 83 | 300 | 92.49658% |
| HWPX | fresh WASM | 383 | 112 | 271 | 92.51660% |

전체 1,532개 페이지·backend 결과 중 356개가 100%, 1,176개가 미달이다.
두 형식 공통 저점은 94·90·92쪽이며, 알려진 86·87쪽 본문/도장 세로 차이와 형식·backend 간 차이도 분리한다.

## 실행 단계

1. 기존 전쪽 TSV를 원장으로 사용하고 미대응 마스크·실제 Print/Native/fresh WASM crop을 연결해
   차이를 조판 위치·글꼴 공급/선택·paint·그림으로 분류한다. 같은 원인과 적용 경로를 묶고
   평균값이나 높은 대표 점수로 개별 미달 쪽을 상쇄하지 않는다.
2. 우선 94·90·92쪽과 86·87쪽의 실제 원본 소유·저장 메트릭·최종 출력 경로를 확인한다.
   `입력 속성 → 공통 생산 결과 → 측정 소비 → 실제 배치 소비 → 최종 원점/paint`를 추적하고,
   독립 Print에서 관측한 기대값과 반례를 기존 기록에 연결한다.
3. 공통 로직에서 원인을 보정한다. 문서 ID·우연한 수치 보정, clamp, 출력 숨김, mask·DPI·2px 반경·
   기준 PDF·허용치 변경으로 100%를 만들지 않는다. 실제 공급 글꼴과 프로그램 해시를 유지하며 글꼴을 배포하지 않는다.
4. 작은 영향 경계와 정상 대조군을 먼저 확인한다. 이후 후보의 두 원본 전 383쪽 Native/fresh WASM을
   실제 재출력해 기존 고정 base의 TSV와 비교한다. 기존 base 자료는 실행 조건·해시가 같은 동안 재사용한다.
   새 코드에 과거 후보 결과를 그대로 적용하지 않는다. #8148의 나머지 입력 70개를 공통 변경의 영향 대조군으로 유지한다.
5. 100% 표시는 반올림만으로 판정하지 않고 raw 미대응 픽셀 0을 함께 확인한다. 쪽수·내용 소속·
   순서·누락·중복과 독립 Print의 미세 간격·괘선·수식·그림을 별도로 판독한다.
6. 각 페이지의 실제 중요 영역에 소유·내용 식별자를 사용한 정식 회귀를 구축한다. 같은 동작의 후보는
   양쪽 raw 최솟값이 100% → 99%대 → 98%대인 순서로 고른다. 독립 Print 기대 좌표 ±2px와
   관계 검사를 함께 유지한다. 빈 페이지는 빈 쪽·앞뒤 내용 소유를 보호하고 좌표 대상이 없는 이유를 기록한다.
   전 페이지에 동일한 머리말 검사만 반복해 중요 영역 보호를 대신하지 않는다.
7. 기존 저장·왕복·바탕쪽·내용 보존·편집 계약 검사는 유지한다. 같은 입력·경로·동작의 좌표 검사만
   새 검사가 동일 계약을 보호하는지 입증한 뒤 통합한다. 시각 점수로 다른 계약을 대체하지 않는다.
8. 최종 code head에서 필수 회귀·lint·Native Skia·최적화 fresh WASM·직접 시각 검증과
   다른 문서의 영향 검증을 완료한다. 새 정식 검사는 `tests/cases/`의 자동 suite로 수집하고
   CI workflow에 불필요한 개별 호출을 추가하지 않는다.

## 실행 경계와 증적

- Cargo는 고정 `/home/tsjang/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=8`, 순차 실행한다.
- Rust 검증은 현재 canonical 절차에 따라 별도 검증 worktree에서 파생 suite를 준비한다.
- 로그·중간 산출물: ignored `output/pr-review/issue8153-handbook-raw100-20261022/`.
  이 경로는 #8155의 output 전체 제거를 확인한 뒤 새로 생성했다.
- 최종 TSV·대표 review/overlay·독립 좌표 근거는 검증 완료 후 정식 asset 경로에 보존한다.
- 현재는 기존 baseline을 재사용한 조사 단계다. 전쪽 100%·전쪽 ±2px 회귀 구축·구현 완료를 주장하지 않는다.

## 첫 원인 조사 — 묶음 그림의 테두리 소비 누락

94쪽은 원본 `section2.xml`의 묶음 `1995777895`, `1995777897`에 각각
`image372.bmp`(731×238), `image373.bmp`(473×559)를 가진다. 두 그림 모두
`lineShape style=SOLID, width=0, color=#000000`이다. 독립 Print에는 그림 프레임의
검은 외곽선이 있지만 Native/fresh WASM의 해당 Image 노드 뒤에는 테두리가 없다.
Print 이미지 영역은 각각 (118.432,130.907)–(481.400,277.413)pt,
(127.183,359.725)–(481.400,664.255)pt이며, 별도 검은 외곽 경로가 존재한다.

기존 그림 테두리 공통 helper는 `de57a512423d894b34f257267bcfc109d171b3b0`에서
TAC 경로를 보완했고 PR #7832(`bfa28a77a2714482f647ee6a7b9aa7f7876d8404`)로 통합됐다.
묶음 `ShapeObject::Picture` 경로는 crop·효과를 Image에 전달하면서 이 helper를 소비하지 않는다.
#5568의 원래 crop 해결 `a5c71572e8`도 유지한다. 한컴 Print pen 공통 처리의 별도 계보는
PR #8064의 `b340c05ce7`이며 기존 일반 선 계약을 임의로 변경하지 않는다.

소비 경로는 `Group`의 저장 affine 행렬 → 자식 위치·크기 확정 →
`layout_shape_object_with_group_origin(matrix_positioned=true)` → Image bbox/확정 transform →
Native Skia/Canvas다. 이미 행렬에 반영한 회전·대칭을 테두리에서 원본 속성으로 다시 적용하면 안 된다.
그림과 테두리가 같은 확정 프레임·transform을 소비하도록 기존 공통 helper를 연결하는 방향을 검증한다.
그림 내부 crop과 외곽 프레임은 구분하며, 기존 TAC·일반 그림·선 없음과 묶음 경로를 대조한다.
이는 구현 전 조사 결과이며 아직 보정 후 출력이나 100% 달성을 확인한 결과가 아니다.

첫 후보는 묶음 그림도 기존 `append_picture_image_node`를 소비하게 연결했다.
그 공통 경로의 테두리는 원본 회전 속성을 다시 해석하지 않고 실제 Image 노드의 확정 transform과
바깥 bbox를 받아 생성한다. 기존 일반 그림·TAC 테두리의 선 스타일과 두께, crop·효과·이미지 위치는
유지했다. 이 후보의 실제 Native 출력부터 확인하며 아직 시각 개선이나 검증 통과로 판정하지 않는다.

### 첫 Native 관측 — source `9f868694ef`

별도 검증 worktree의 전체 fmt는 exit 0, jobs 8·공유 `target/pr-review`의
`release-test`/`native-skia` CLI 빌드는 7분 59초·exit 0이었다.
새 producer SHA-256은 `d2fbacf8fb227c0f9ccefd66cc2b4747f317d35d5c91c5ffa75ae365836bdb77`이다.
이전 기준 PNG·Print raster는 해시 고정 캐시에서 재사용했다.

| 원본 | 쪽 | 기준 raw | 첫 후보 Native raw | 차이 %p |
| --- | ---: | ---: | ---: | ---: |
| HWPX | 90 | 92.61726 | 92.61726 | 0 |
| HWPX | 92 | 92.81992 | 97.45048 | +4.63056 |
| HWPX | 94 | 92.49658 | 98.78038 | +6.28380 |
| HWP | 90 | 92.61751 | 92.61751 | 0 |
| HWP | 92 | 92.82064 | 97.45073 | +4.63009 |
| HWP | 94 | 92.49647 | 98.78036 | +6.28389 |

94쪽의 새 Native review에서 외곽선 추가와 원래 두 그림·주석의 위치·순서를 확인했다.
Print보다 선이 옅고, 남은 미세 차이가 있어 100% 완료로 판정하지 않는다.
임시 비교 원장은 `output/pr-review/issue8153-handbook-raw100-20261022/`의
`group-frame-9f868694ef/summary.json`, 각 문서 Native TSV와 review/overlay다.
fresh WASM·관계 회귀·대조군은 이 후보에서 아직 실행 전이다.

별도 조사에서 46쪽 본문 그림 `image361`(section 2, para 208)은 저장 선 굵기가 2 HU인데
기존 그림 helper가 0.02667px로 변환해 Native에서 내용 임계값보다 옅어지는 것을 확인했다.
독립 Print 46·67·92·93·94·95·145·161·162·163쪽의 그림 외곽 path와
600dpi pen의 0.12pt stroke를 읽어 비교 원장에 보존했다.
0 HU·2 HU·28 HU·34 HU와 변환된 그림의 실제 굵기가 모두 같은 계약인지는 추가로 대조한다.
아직 이 조사만으로 선 굵기나 다른 backend의 최소값을 변경하지 않는다.

회귀 실패는 사용자 지시대로 독립 Print와 기존 해결 계보를 먼저 확인한다.
근거 없는 절대 px·고정 영역은 관계 검사로 개선하고, 완료 페이지의 양쪽 raw 98% 이상·
독립 기대값 ±2px 검사는 유지한다. 이관은 고정한 기준 devel에서도 부적합함을 입증한 검사에
원본 코드·helper·입력·기준·해시와 복원 조건을 보존해 적용한다. 현재 수정의 회귀는 구현을 고친다.

### 전쪽 확인과 다음 원인 계층

동일 runtime 후보의 HWPX Native 383쪽은 개선 6쪽(92·94·95·131·161·163), 하락 0쪽,
동일 377쪽이었다. 최저 raw는 90쪽 92.61726%, raw 100%는 83쪽으로 아직 증가하지 않았다.
일부 개선을 전쪽 100% 달성으로 바꾸어 보고하지 않는다. HWP 전쪽과 최적화 fresh WASM은 진행 중이다.

임시 전체 검증 driver의 첫 실행은 main guard 누락으로 spawn 작업자가 렌더 명령까지 중복 실행했다.
소유한 Native process group만 명시적으로 종료(exit 143)하고 해당 실행을 무효화했다.
main guard와 worker import의 무실행을 확인한 뒤 두 원본을 같은 producer로 다시 실행했다.
새 전쪽 TSV는 `whole-score/`에 두어 완료된 90·92·94쪽 단독 비교와 구분한다.
WASM 빌드는 이 오류와 무관하게 계속됐으며, 관측 timeout을 종료로 오인해 재시작하지 않았다.

94쪽 Native의 미대응 mask는 5,051개에서 821개로 감소했고 새 미대응 픽셀은 0개였다.
남은 821개 중 358개는 용지 오른쪽 경계에 있다. 원본 masterpage0의 사각형
`1993203321`은 PAPER 기준 가로 오프셋 55,559 HU(원본 용지 폭), 선언 너비 0,
선 굵기 33 HU다. Print에는 x=555pt의 0.36pt 검은 stroke가 보이지만 현재 Native에서는
논리 종이 끝 x≈740.79px가 물리 Print 폭 740px 밖으로 나가 잘린다.
기존 `bd5207aa02`/PR #8148은 바탕쪽 raster만 물리 참조 프레임으로 투영했으며,
동적 글자·벡터·본문을 그대로 둔 독립 근거를 함께 기록했다. 따라서 이를 전역 축소로
대체하지 않고, PAPER 그래픽의 생산·소비와 정상 대조군을 먼저 확인한다.

90쪽 원본 `image365.jpg`는 1,195×1,441이며 Print에는 잘린 870×952 JPEG가 있다.
원본과 PDF 자원의 JPEG 양자화 표가 다르고, 0,0 정수 crop의 평균 성분 차이는 약
3.278/2.772/2.592다. 92·94쪽 BMP도 PDF에서 서로 다른 양자화 표의 JPEG로 저장돼 있다.
기존 stage2025·2101의 bitmap 팔레트 계약은 JPEG를 별도 대조군으로 유지했으므로
단일 밝기·JPEG quality 상수로 치환하지 않는다. crop·codec·좌표 차이를 분리해 다음 보정을 정한다.

### 첫 후보의 양쪽 전쪽 결과와 회귀 등록

runtime `9f868694ef`의 fresh WASM은 최적화까지 23분 25초·exit 0이었다.
실제 WASM SHA-256은 `be702d53b9623ba7714b0b755ab62cb43f7d5b3060e2adf27ce09932ddddf600`이며
JS는 `da00f4937f352731a2d24a1ba4b54629009aa5edc9c929815aed2cd069cca3bc`다.
같은 원본·Print·글꼴 213개 프로그램/face/family를 공급했고 기준 캐시와 그 목록이 일치했다.

| 원본 | backend | 완료 쪽 | 개선/하락/동일 | 최저 raw | 100% 쪽 |
| --- | --- | ---: | --- | ---: | ---: |
| HWPX | Native | 383 | 6 / 0 / 377 | 92.61726% | 83 |
| HWP | Native | 383 | 6 / 0 / 377 | 92.61751% | 74 |
| HWPX | fresh WASM | 383 | 7 / 1 / 375 | 92.60426% | 112 |
| HWP | fresh WASM | 383 | 7 / 1 / 375 | 92.60462% | 87 |

WASM 248쪽의 -0.00001%p는 두 형식 모두 미대응 359개 유지·새 미대응 0개,
HWPX 기준 내용 합집합 79,090→79,089의 분모 변화다. section 8, para 40의
두 묶음 그림은 원본의 흰색 실선 가림띠다. 그 테두리를 소비하면서 기존 일치 픽셀 한 개가
내용 마스크에서 빠졌고, 확대 대조에서 다른 요소의 이동·누락을 발견하지 않았다.
Native는 원본별 8쪽 RGB 변경/375쪽 RGB 동일이며, 162·248쪽은 raw가 같아도 RGB는 달랐다.

기존 묶음 crop 대조 2건과 160쪽 표 hit-test 대조 1건은 모두 PASS다.
정식 [묶음 그림 회귀](../../tests/cases/issue_8153_group_picture_border.rs)는 두 형식의
94쪽 관계 검사와 163쪽 독립 Print ±2px 검사 총 4건으로 작성했다. 이 새 4건은 실행 전이며,
기준 코드에서의 의도한 FAIL과 후보 PASS를 다음 검증에서 확인한다.
source 4.67의 image399를 식별하는 소유 정보와 BinData ID를 사용하며 좌표 영역으로 찾지 않는다.
검은 외곽선의 같은 핵심 동작을 확인할 수 있는 쪽 중 양쪽 raw 최솟값이 높은
163쪽(두 형식·양쪽 최저 99.35380%)을 px 보호 대상으로 골랐다. 248쪽은 흰색 가림띠라
검은 외곽선 누락 검출의 대체 사례로 삼지 않았다. 94쪽의 양쪽 최저도 98.78036%이며
위치·소속·순서·crop 관계 검사를 유지한다. 전체 100% 목표의 완료가 아니다.

163쪽 기대값은 독립 Print의 669×946 이미지 bbox를 PyMuPDF `get_image_info`로 읽은
`[154.992004,234.696960,466.056000,664.974976]pt`를 4/3배 한
`[206.656006,312.929281,621.408000,886.633301]px`다. 두 PDF에서 같음을 확인했다.
기대값은 후보 render tree에서 복사하지 않았다. 전후 원본/PDF SHA는 기존 입력 원장을 따른다.
대표 94·163쪽 Native/fresh WASM의 compare·review·standalone overlay와 해당 metrics는
`output/pr-review/issue8153-handbook-raw100-20261022/group-frame-9f868694ef/`의
`<backend>/<문서>/`에 보존했다. 확대 판독 시 위치·내용 보존과 아직 남은 선 농도·codec 차이를 구분했다.

첫 전쪽 채점은 시스템 Python(Pillow 10.2.0)으로 실행한 환경 편차를 확인했다.
이후 저장소 `venv`(Pillow 12.3.0)로 94·163쪽 8개 조건을 실제 재채점해 기존 수치와
모두 같음을 확인했고 해당 대표 PNG를 재구성했다. 이후 검증은 `venv/bin/python`을 사용한다.

## 우선순위 변경 — 전쪽 100% 출력 보정 선행

사용자 지시에 따라 추가 회귀 구축과 기준 코드 FAIL 재현은 전쪽 100% 보정 뒤에 진행한다.
위 새 4건의 후보 실행은 `0af1840c78`에서 4 PASS로 완료했다.
기존 전쪽 PNG의 네 조건 1,532쪽도 저장소 venv로 재채점했으며 새 렌더링으로 보고하지 않는다.

다음 보정은 PR #8148의 `bd5207aa02`에서 마련한 논리 종이 → 물리 Print 종이 프레임을
바탕쪽 벡터에도 소비시키는 것이다. 현재는 raster만 이를 소비해 원본 종이 오른쪽에 있는
사각형이 Print에서 잘린다. 원본 `masterpage0.xml`의 PAPER 사각형은 x=55,559 HU,
너비 0, 높이 27,116 HU이며 독립 Print 94쪽에는 x=555pt의 검은 경계선이 실제 존재한다.
기존 후보 94쪽의 미대응 821픽셀 중 358픽셀이 이 경계에 있다. 전체 네 조건에서도
종이 오른쪽 경계만 미대응인 쪽이 반복된다. 원본 ID나 쪽 번호는 적용 조건에 사용하지 않는다.

소비 경로는 원본 바탕쪽 컨트롤 → 논리 RenderNode/바탕쪽 layer → PagePaintContext의
물리 종이 투영 → 공통 LayerTree의 벡터 paint group → Native/Canvas다.
선·사각형·타원·경로의 실제 geometry와 visual bounds가 같은 변환을 소비한다.
회전이 있으면 원본 도형 회전 뒤 종이 투영을 적용하고, 내부 글자는 기존의 독립 글자 좌표와
회전을 유지한다. 본문, 화면 profile, 동적 바탕쪽 글자는 투영하지 않는다.
기준 PDF·마스크·DPI·허용 반경은 유지한다. 후보 출력의 영향 쪽과 정상 대조군을 먼저
실제로 재출력하며, 아직 이 보정의 개선이나 전쪽 100%를 판정하지 않는다.

## 다음 원인 — 그림 테두리의 물리 pen 폭

46쪽 source 2.208의 image361은 실선 2 HU다. 현재 Native는 이를 약 0.02667px로
그려 검은 테두리가 마스크 임계값보다 밝아진다. 미대응 3,574픽셀은 모두 PDF에만 있고,
그중 1,562픽셀은 Print에서 검은색인데 현재 출력의 반경 2px 안 최솟값도 248 이상이다.
독립 PDF는 그림 외곽을 채운 검은 띠와 0.12pt pen으로 그린다. 2 HU의 최소 띠 0.12pt와
외곽 pen을 합하면 0.24pt(0.32px)다. 28 HU의 띠는 0.24pt, 34 HU의 띠는 0.36pt이며
각각 같은 외곽 pen을 갖는다. 서로 다른 세 원본 선언 폭과 Print path를 대조했다.

PR #8064의 공통 600dpi pen 양자화를 재사용하고, 그림 테두리도 실제 채운 띠와 외곽 pen의
총 폭을 소비하게 한다. 0은 기존 기본 0.1mm를 양자화하며, 선 종류·색·crop·프레임은 유지한다.
도형의 기존 실선/점선 총 폭 계산은 바꾸지 않는다. 일반·TAC·묶음 그림이 같은 그림 테두리
helper를 소비하고 Native/fresh WASM에서 같은 style로 전달된다. 아직 실제 후보 출력 검증 전이다.

## JPEG 제외 조건의 독립 재검토

90쪽 남은 차이에는 bitmap의 밝은 면과 선이 포함된다. 기존 stage2025의 JPEG 대조군을
다시 조사하니, 다른 형식의 257색과 달리 JPEG는 디코드 뒤 256색이었다.
따라서 그 형식만 제외하는 조건의 근거가 부족했다. 같은 사진을 형식·crop만 달리한
새 독립 한컴 Print 대조를 수행하고 입력과 PDF를
[정식 보존 경로](../../pdf/issue8153-print-codec-controls-20261022/README.md)에 기록했다.
기존 편람 PDF를 재인쇄한 것이 아니다. 합성 진단 입력과 실제 편람 검증을 구분한다.

새 full/crop JPEG는 각각 14,845/9,484색이며 BMP·PNG와 같은 색 확장 경향을 보인다.
기존 256색 JPEG·indexed PNG는 제외를 유지하고, 2024 Print의 실제 가시 색 수 판정에
JPEG도 포함한다. 공통 payload 변환과 source variant의 재조회가 같은 바이트를 소비한다.
2020·그림 효과·밝기/대비 조작의 적용 조건은 유지한다. 새 코드의 실제 개선 수치는 아직 미검증이다.

## `c1ca8a7091` Native 전쪽 실행 결과와 다음 보정

Native CLI 세 후보 `16f9131921`·`913317779c`·`c1ca8a7091`은 공용 target에서
각각 순차 빌드해 exit 0을 확인하고 덮어쓰기 전에 producer를 보존했다.
현 후보 Native SHA-256은 `9ab072cfa99201a56a63c052bdb3fca4c50b9ba4d3b3e5e92e44f18fc3d663a9`다.
검증 worktree는 각 source SHA로 분리 확인했으며 원본·기존 Print·글꼴·DPI·마스크는 유지했다.
전쪽 Native는 각 383쪽, 총 766쪽의 실제 재출력과 venv TSV 산출을 완료했다.

| 원본 | 직전 후보 raw 100% 표시 | 현 후보 raw 100% 표시 | 현 후보 최저 |
| --- | ---: | ---: | ---: |
| HWPX | 83 | 133 | 95.41165% (240쪽) |
| HWP | 74 | 127 | 95.41299% (240쪽) |

여기서 쪽 수는 TSV 표시 기준이며 전쪽 100%나 모든 표시 쪽의 미대응 0 감사 완료를 뜻하지 않는다.
12쪽은 두 형식 모두 100%이며 HWPX의 미대응 0을 별도 마스크 계산으로도 확인했다.
94쪽은 직전 후보 821개 중 종이 오른쪽의 358개를 제거하고 463개가 남았다.
46쪽은 양쪽 99.99868%, 미대응 1개다. 90쪽은 HWPX 96.69907%·HWP 96.69919%다.
신규 회귀 추가·기준 코드 FAIL 재현은 사용자 지시대로 100% 보정을 먼저 진행한 뒤 수행한다.

낮아진 쪽도 보존한다. base 대비 1쪽은 양쪽 -0.00041%p, 145쪽은 HWPX -0.00031%p·
HWP -0.00032%p다. 직전 묶음 그림 후보 대비 163쪽은 각각 -0.01941/-0.01940%p다.
다른 쪽의 개선으로 이를 완료 처리하지 않으며 100% 목표의 잔여에 포함한다.
전체 차이는 `jpeg-palette-c1ca8a7091/native/<문서>/base-head-delta.json`,
원본 TSV는 같은 경로의 `whole-score/silhouette.tsv`다.

fresh WASM은 같은 `c1ca8a7091`에서 빌드 중이며 Rust 컴파일 뒤 wasm-opt 단계다.
성공 exit를 확인한 후 두 원본 전체 766쪽을 실제 실행하는 controller를 연결했다.
이전 WASM 수치를 현 후보의 수치로 사용하지 않는다. 현재 전쪽 100%는 미달이다.

다음 실제 배치 차이는 240쪽의 section 7, pi 405, 9×3 표다. 독립 Print의 마지막
수용 행(6행) 하단은 y=883.275px인데 현재는 약 874.5px다. 다음 241쪽의 7행은 양쪽
약 90.2px로 일치해 앞 조각에서 물리 높이가 사라지는지 추적한다. 원본의 5행 0열
rowspan=3 선언 높이 17,351 HU와 두 조각의 수용 행·간격·남은 높이를 대조한다.
이는 아직 원인을 확정하거나 구현한 상태가 아니다. PR #7567(`1d6bc70767f`)의
저장 조각·빈 띠 계약과 이후 공통 원점 보정 `335a03fd1e`를 확인하고, 스캔의 예약부터
실제 배치까지 같은 물리 상자를 소비하는지 trace로 확인한다.

별도로 그림 crop을 정수 픽셀로 단순 절삭한 **변형 입력 진단**은 12쪽에 새 미대응 406개를
만들고 90쪽도 하락했다. 이 변형은 제품에 적용하지 않았다. PDF 내부의 정수 bitmap 크기만으로
최종 프레임까지 단순 절삭할 수 없으며, 이 자료를 원본 개선이나 새 baseline으로 사용하지 않는다.

## 240쪽 블록 컷의 실제 경로와 후보 보정

원본 240쪽의 모델 정책은 HWPX `CELL`에 대응하는 RowBreak다. trace에서 완전 행 경계의
걸침 블록 5..8이 `split_block_start=Some(5)`로 잘리므로 일반 끝 행의 빈 띠 경로는 적용되지 않는다.
단순히 일반 분할의 rowspan 제외 조건을 없애는 것으로 해결할 수 없다.
현재 블록의 물리 예약은 약 132.1px, 마지막 남은 7행의 독립 Print 높이는 약 90.2px다.
원본 걸침 상자 231.34667px 중 현재 조각이 소유해야 할 몫은 약 141.1px다.
142.5px라는 전체 가용 방을 그대로 요구하면 다음 행 최소 높이를 침범해 후보가 기각되고,
실제로 사용할 수 있는 빈 띠 약 9px까지 잃는다.

PR #7382의 메인터너 보정 `a17cb22b79f`가 만든 공통 `complete_row_boundary_band`를 보완한다.
현재 조각의 이미 수용한 물리 높이는 줄이지 않고, 다음 한 행의 원본 최소 높이와 실제 내용 높이를
먼저 남긴 뒤 현재 조각의 가용 밴드를 정한다. 과거 source frame 판정과 내부 내용 컷은 유지한다.
완전 행 경계 판정은 같은 원본 RowBreak 계약을 사용하는 HWPX 저장 경로에도 공유한다.

소비 연결은 `cell_units → RowBlockQuery::fragment_height → complete_row_boundary_band의
요구 높이 → scan_rowspan_block_step의 예약/끝 행 override → emit의 같은 컷·수용 원장 →
table_partial의 실제 행 상자`다. 완전 행 경계에서 추가 글줄을 소비하지 않고 빈 띠를 예약한다.
기존 수용 분기가 컷 표현을 완전 행 경계로 바꾼 뒤 다음 조각의 시작·소유도 실제 출력으로 대조한다.
예산이 모자라면 스캔에서 수용하지 않으며 paint 뒤에 상자만 늘리지 않는다. 적용되는 경계의 재출력은
240·241쪽과 정상 완료 쪽, 원 PR의 8쪽 심사지표를 우선 대조한다. 아직 보정 후 검증 전이다.

`c1ca8a7091` Native에서 raw 100%로 표시된 260쪽은 동일 마스크·반경으로 별도 계산한 결과
**260쪽 전부 미대응 0**이었다. 새 후보의 결과로 재사용하지 않는다.
해당 코드의 fresh WASM은 최적화까지 exit 0(21분 42초)이며 전체 실제 실행 중이다.

### 최초 밴드 후보의 후속 쪽 실패와 원장 보정

`4b58101963`의 Native에서 240쪽은 99.99884%로 개선됐지만 241쪽이 87.02060%로
하락했다. 새 후보는 채택하지 않았다. 240쪽 6행은 독립 Print처럼 약 80.8px가 됐으나,
241쪽 7행이 원래 90.2px 대신 초기 전체 측정값 95.8px를 다시 쓰면서 뒤 표까지 밀었다.

스캔의 완전 행 밴드 수용은 빈 컷과 마지막 행 높이로 표현된다. 원장 소비자는 그 빈 컷을
다시 셀 유닛 선택으로 질의해 완료 prefix로 인정하지 않았고, 앞 조각에서 실제 수용한
5·6행을 기록하지 못했다. 이 명시적 스캔 결과를 원장에서 완료 행으로 소비하도록 수정한다.
다음 조각은 기존 `stored_fragment_row_plan`에서 실제 수용한 상자를 차감해 닫는 걸침 행을
다시 계산한다. 별도 좌표 덮어쓰기나 다음 쪽의 강제 높이 보정은 추가하지 않는다.
이 보정 후 240·241쪽과 심사지표 대조군을 다시 실행하며, 아직 후속 쪽 개선은 미검증이다.

직전 `c1ca8a7091`의 fresh WASM 전체 766쪽도 exit 0으로 완료했다. raw 100%는
HWPX 190쪽·HWP 150쪽이며, 340쪽 모두 별도 감사에서 미대응 0이었다.
따라서 직전 후보 네 조건의 100%·미대응 0은 총 600/1,532건이며 932건이 남았다.
같은 코드의 두 browser catalog는 이전과 213개 font program·face·family SHA가 모두 같고,
수정된 임시 폰트의 보존 경로 46개만 달랐다. 새 밴드 후보의 증거로 이 결과를 사용하지 않는다.

## 사양 기반 보정으로 구현 근거 재정리

작업지시자의 추가 조건에 따라 점수 개선만으로 인쇄 동작을 추정한 상수를 채택하지 않는다.
100% 목표는 유지하며, 사양에 없는 수치를 공식 수치라고 기술하거나 독립 Print 관측값으로
파일 형식의 수치 계약을 대신하지 않는다. 앞 절의 후보 결과는 해당 SHA의 실험 기록이다.

직접 대조한 원문은 `pdf/한글문서파일형식_5.0_revision1.3-hancom-official.pdf`와
`pdf/hwpspec-2024.pdf`다. 전자의 PDF 11쪽(표 1, 자료형 설명)은 HWPUNIT을
1/7200인치로 정의하고 출력 장치의 해상도가 가변임을 명시한다. 후자의 HWPML 표 120
`LINESHAPE`는 `Width` 단위를 hwpunit으로 명시한다. 따라서 양수 그림 테두리는
저장 두께 × 실제 DPI / 7200으로 환산한다. 기존 0 두께 처리와 도형 전용 구현은 이번
수정 이전의 동작을 보존하며, 이번 조사로 그 동작의 사양 적합성까지 입증한 것은 아니다.

| 후보 변경 | 사양 대조와 현재 처리 |
| --- | --- |
| `913317779c` 그림 테두리 600dpi 양자화 | 고정 장치 해상도·최소 1도트·추가 outline 도트의 공식 근거를 확보하지 못했다. 이번 작업에서 추가한 그림 보정을 제거한다. |
| `c1ca8a7091` JPEG 색 확장 | HWP5 그림 정보는 밝기·명암·그림 효과·BinItem 참조를 정의한다. JPEG의 가시 색 수로 `c+(c>>5)`를 적용한다는 근거는 확인한 사양에 없다. 이번 JPEG 확장을 제거한다. 독립 진단 자료는 사양 근거와 구분해 보존한다. |
| `16f9131921` 바탕쪽 벡터 물리 종이 투영 | 원문 개체 폭·높이·오프셋은 HWPUNIT이다. 출력 PDF의 정수 point 크기로 벡터 좌표까지 축소해야 한다는 공식 근거를 확보하지 못했다. 이번 벡터 투영 추가를 제거한다. |
| `9f868694ef` 묶음 그림 테두리 누락 수정 | 저장 그림의 테두리 속성을 기존 공통 그림 경로로 전달하는 수정이다. 새 수치·최소 선폭·문서별 상수를 추가하지 않는다. |
| `4b58101963`·`632f215366` 표 분할 후보 | 표 75의 CellSpacing, 표 80의 셀 높이·병합 수를 대조한다. 공식 필드의 존재만으로 특정 분할 배분식까지 공식 규칙이라고 부르지 않는다. 앞서 발생한 241쪽 하락과 소비 원장의 결함을 따로 확인한다. |

공식 필드와 계산을 구분한다. HWP5 원문 PDF 44쪽 표 75는 CellSpacing을 HWPUNIT16으로,
45쪽 표 80은 셀 높이를 HWPUNIT으로 정의한다. 병합 셀 높이 17,351 HU는 96dpi에서
231.346666…px, 다음 단일 셀 높이 6,766 HU는 90.213333…px, 셀 간격 255 HU는 3.4px다.
이 저장값은 추정 상수가 아니다. 다만 남은 높이를 어느 조각이 소유하는지는 별도 계약이며
공식 사양·원본 소유·수용 원장과 후속 출력이 모두 일치하기 전에는 완료로 채택하지 않는다.
이전 Native/fresh WASM의 600/1,532건 100% 결과를 보정 제거 후 head 결과로 재사용하지 않는다.

### 사양 오류 확인과 매뉴얼 정정

추가 지시대로 사양을 절대적인 정답으로 취급하지 않는다. 사양과 충돌하면 공식 문서,
참조 구현, 실제 원본 바이트와 독립 한컴 출력으로 확인한 뒤 정오표와 사양 설명본을 함께 고친다.
미기술·추정·확인된 오류를 구분한다.

이번 표 분할 조사에서 HWP5 표 76의 값 2가 `나누지 않음`으로 중복 기재된 오류를 확인했다.
선행 #1094(`20e532d170f`)의 바이트 대조를 읽고, 현재 편람 원본 Section7 문단 405의
TABLE prefix `06 00 00 00 09 00 03 00 ff 00`을 rhwp 모델과 독립적으로 읽었다.
하위 2비트 값 2가 HWPX `CELL`에 대응하고, 기존 독립 PDF 240~241쪽에서 표가 이어짐을 확인했다.
hwplib의 고정 SHA와 공식 HWPML의 TABLE 설명, 한컴 표 속성 도움말도 교차 확인했다.
[정오표 §35](../tech/hwp_spec_errata.md#35-table-쪽-경계-속성-값-2--나누지-않음-중복-기재),
사양 설명본 표 76과 내부 enum의 반대 의미 주석을 정정한다. 원문 PDF와 실제 enum 값은 보존한다.

임시 감사 코드는 `output/pr-review/issue8153-handbook-raw100-20261022/audit_table_policy_spec.py`,
원시 바이트 결과는 `spec-table-pagebreak-original-bytes.json`, 독립 PDF 대조는
`spec-pagebreak-independent-print.json`에 있다. 실행 기록은 `logs/spec-table-policy-audit.log`다.
이 정정이 표 높이 배분식이나 600dpi·JPEG 색 보정까지 공식화하는 것은 아니다.

### 사양 확인 후 실제 검증

런타임 후보 `b9739125d8ef4ed0a906f58eb43914d9384719e2`의 Native 빌드는
공용 target·잡 8개로 5분 48초, exit 0이다. 두 편람의 12·46·94·133·163·240·241·375쪽,
총 16쪽을 기존 동일 Print·글꼴·96dpi로 재출력하고 원본 TSV를 산출했다.
검증한 16쪽은 고정 base 대비 하락이 없다. 240쪽은 HWPX 99.58234%·HWP 99.58352%이며
241쪽은 각각 99.97617%·99.93829%로 기준을 유지한다. 전체 1,532건 100%를 뜻하지 않는다.

별도 선행 PR 대조군 심사지표 8쪽도 실제 재출력을 완료했다. C1과 7쪽은 RGB 동일하고
6쪽은 실제 수용 원장 보정으로 달라졌다. 이를 무회귀로 추정하지 않고 같은 독립 2020 PDF와
대조한 결과, 해당 6쪽 raw는 61.38979%에서 99.76441%로 개선됐다.
수정 후 review에서 위 표의 글줄·이어받은 중첩 표·마지막 외곽선 위치를 직접 판독했다.
이 결과는 대조군 8쪽 전체가 raw 98% 이상이라는 주장이 아니다.
`span-control-simsa/spec-audited-p006-delta.json`과 `c3/p006-score/review_006.png`를 보존한다.

사양·정오표·계획서 세 문서의 내부 상대 링크 검사는 exit 0이다.
현재 root `12d0938913f33ee50ce6a43adbcedce6de30b939`는 위 런타임에서 문서와 enum 주석만
달라진 상태다. 해당 SHA의 fresh WASM은 잡 8개·codegen-units 16으로 별도 빌드 중이며,
최적화 성공 뒤 영향 16쪽을 실제 실행하도록 controller를 연결했다. 이전 후보의 WASM 결과를
현재 결과로 사용하지 않는다. 로그는 `logs/spec-audited-fresh-wasm-build.log`와
`logs/spec-audited-focused-wasm.log`다.

## 일반 셀의 그림 보완 경로와 공식 글줄 원점

133쪽 실제 그림 `image378`은 HWPX section3 문단45, 표1179766190의 2행0열,
셀 문단7에 있다. 원본 LineSeg는 `vertpos=25376`, `vertsize=textheight=12488`,
`baseline=10615`이며 그림 `sz.height=12488`, 바깥 여백과 위치 offset은 0이다.
HWP5 사양 표62는 줄의 세로 위치·높이·베이스라인을 각각 정의한다. 문단 앞 간격까지
소비한 실제 TextLine의 원점은 약716.7px인데 보완 그림은 조판 전 문단 원점707.4px를
다시 사용했다. 독립 Print의 그림 상단537.4279785pt는 716.570638px다.
저장 글줄과 그림이 같은 원점을 가져야 하며, 9.3px를 고정해 더하는 방식으로 보정하지 않는다.

선행 `21d0f27bb1ca1c1aeffacf240c8b74537cbbe198`은 분할 셀의 같은 결함에
`rendered_picture_paragraph_origin`을 도입했다. `1715d321e2e`는 완전 셀·안내 글줄에서만
별도로 emitted line box를 소비해 일반 혼합 셀의 첫 그림 줄은 적용 대상이 아니었다.
또한 PR #7377(`c84a8f8e1a`)의 전체 저장 그림 줄·캡션·폭 초과 계약을 읽었다.
이번 후보는 기존 공통 원점 helper를 일반 셀 fallback에도 소비하고, 후속 줄 이동은 같은
원점과 저장 LineSeg 상대 위치를 사용한다. x·너비·높이·crop·베이스라인 계산은 바꾸지 않는다.
새 수치·장치 해상도·문서 식별 조건을 추가하지 않는다.

생산·소비 연결은 `layout_composed_paragraph_in_frame → 실제 TextLine 원점 →
rendered_picture_paragraph_origin → 일반/분할 셀 fallback 그림 원점 → layout_picture`다.
분할 셀은 기존 helper 경로를 유지한다. 뒤에서 완전 프레임/안내 글줄의 실제 상자를 쓰는 기존
분기도 유지한다. 후보는 아직 재빌드·시각 검증 전이며 133쪽 개선을 확정하지 않는다.
공용 target의 fresh WASM 검증이 끝난 뒤 Native를 순차 빌드하고, 두 형식133쪽·인접134/135쪽,
기존133이외 주요 쪽과 #7333·#6122의 그림/폭 초과 대조군을 실제 비교한다.

### 공통 원점 후보의 실제 실행 결과

`e4efb59b8c0541e226ad7d83c15a3b6b9c790969`의 Native 빌드는 공용 target·잡 8개로
6분 3초, exit 0이다. 직전 fresh WASM의 최적화까지 22분 16초·exit 0을 확인한 뒤 순차 시작했고,
producer를 별도로 보존했다. 원점 후보의 두 형식 11쪽씩 총 22쪽을 재출력해 TSV·대표 review를 만들었다.
고정 base 대비 하락은 0쪽이다. 기존 #7333의 10쪽과 #6122의 6쪽 1건은 직전 후보와
**11쪽 모두 RGB 동일**하고, 원본과 각 독립 PDF의 전체 쪽수도 50쪽/12쪽으로 일치했다.

원본 paraPr155의 `HwpUnitChar` case에서 `prev=700`HU(legacy default1400)가 실제
문단 앞 간격이며 96dpi에서 9.333…px다. 조판이 이미 소비한 이 원점을 공유한 결과,
- handbook-hwpx 133쪽: 96.09230% → 99.99947%.
- handbook-hwp 133쪽: 96.09214% → 99.99947%.

그림의 실제 위쪽과 인접 글줄·다이어그램 경계를 독립 Print review로 직접 판독했다.
HWPX 133쪽의 같은 232 임계값·2px 반경 마스크에서 남은 미대응은 **1픽셀**이다.
좌표(608,918)의 Native RGB 255와 PDF RGB 231이며 본문 그림이 아닌 바탕쪽 쪽번호 영역이다.
반올림한 review 표시 100.00%를 전쪽 100% 달성으로 해석하지 않는다. 마스크·반경을 바꾸거나
그 1픽셀에 맞춘 위치/색 상수로 보정하지 않는다.

검증 로그는 `logs/inline-origin-native-driver.log`, 전체 22쪽 결과는
`inline-origin-e4efb59b8c/summary.json`, 대조군은 `picture-origin-controls/after-summary.json`이다.
초기 대조군 집계의 multiprocessing 진입점 오류는 main guard로 수정했고, 같은 producer의
완성된 PNG를 재사용해 집계 재시도 exit 0을 확인했다. 실패 로그도 별도로 보존했다.
이제 같은 producer의 선행 11쪽을 재사용하며 두 원본의 전 766쪽 Native를 실행 중이다.
`logs/inline-origin-whole-native.log`를 따른다. 이 후보의 fresh WASM은 아직 빌드 전이며,
직전 12d0938913의 WASM 영향 16쪽(하락 0쪽)을 새 후보의 결과로 대신하지 않는다.

사용자 요청의 사양 우선·확인된 불일치의 문서 정정 의무는 `6d90a02bf9`에서
AGENTS·CLAUDE·공통 작업 지침·정오표 검증 원칙에 반영했다. 네 문서 링크 검사는 exit 0이다.

### Native 전쪽 종료와 도장 호스트의 저장 글줄

동일 `e4efb59b8c` producer의 Native 전 766쪽 검증은 exit 0으로 끝났다.
HWPX 최저 92.61726%, HWP 최저 92.61751%이며 고정 base와 직전 후보 대비 하락은 0쪽이다.
표시 raw 100%는 각각 83쪽·74쪽이다. 반올림을 포함한 수치이며 미대응 0 감사와 구분한다.
이 후보의 fresh WASM은 아직 없으며, 전쪽 100% 목표는 계속 진행한다.

86쪽 section2의 실제 TAC 표 1670013921, 첫 셀은 높이 3431HU·세로 CENTER다.
호스트 글줄은 vpos 800HU·높이 1500HU·baseline 1275HU이고, 글 앞으로 도장 표는
별도 PARA 원점과 signed offset -48HU를 사용한다. 한컴 Print에서 이름 baseline은
680.160px, 도장 상단은 654.336px다. Native의 도장 상단 654.1px는 유지할 위치지만
이름 글줄 상단 652.9px는 저장 위치를 소비하지 않았다. 원본 여백 141HU와 vpos를
사용한 이름 baseline은 약 680.05px다. 10px 상수를 더하는 근거로 쓰지 않는다.

공식 HWP5 표 62의 글줄 위치와 표 80의 셀 높이·여백을 소비한다. 실제 호출은
`layout_horizontal_cell_paragraphs → has_block_table_ctrl → layout_composed_paragraph`다.
비-flow 표도 블록 표로 분류되어 이 텍스트 경로가 저장 셀 원점을 전달하지 않는다.
뒤의 `stored_nested_float_placement → nested_y → layout_table`은 이미 도장 원점을
선택하므로 호스트 `para_y` 전체를 이동하지 않는다. 기존 `_in_frame`의 저장 셀 원점
인자를 글줄에만 전달하는 후보를 검증한다. 별도 문서 ID·offset·해상도를 추가하지 않는다.

수용 범위는 저장 HWP/HWPX의 미편집·미재조판 셀, 원본 높이를 유지한 완전 셀,
변경되지 않은 줄 수와 셀 안에 들어가는 단조 저장 글줄이다. 부분 조각·프레임 변경·캐시
무효 입력은 기존 경로를 유지한다. 대상 호스트의 비자리차지 표는 기존 overlay 분류로
식별한다. 저장 위치를 새로운 가운데 정렬 공식으로 일반화하지 않는다.
#5593의 합성 초기 vpos 800은 한컴 재저장 시 0으로 정정됐던 반례이므로, 같은 숫자만으로
캐시를 수용하지 않는다. 기존 개체 시각 높이의 정렬 참여와 별도 원점은 유지한다.
85·86·87·144·179쪽 및 #5593 독립 Print 대조군을 먼저 실제 출력한 뒤 전체 검증을 진행한다.

### 비자리차지 표 호스트의 첫 후보 검증

첫 런타임 후보 `1a95d10761a761e6c746d8e8f75387a5a2fb4088`의 Native 빌드는
6분 56초·exit 0이다. 초기 `7c37a78b68`의 메서드 호출 범위 오류는 별도 보정 커밋으로
수정했고 실패 로그를 보존했다. 바뀐 두 Rust 파일의 rustfmt 검사도 exit 0이다.
두 형식의 85·86·87·133·144·179·240·241·375쪽, 총 18쪽에서 base 대비 하락은 0쪽이다.
86쪽 raw는 HWPX 96.04151% → 98.51749%, HWP 96.04135% → 98.51743%다.
87쪽은 각각 98.61784% → 99.60381%, 98.61775% → 99.60379%다.
133쪽 그림 원점 개선과 240·241쪽 분할 개선도 유지한다.

86쪽 render tree의 이름은 y 652.9px → 663.0px이며 x·크기는 그대로다.
도장 표는 x 421.4px·y 654.1px·34.0×37.8px로 같고 독립 Print와 직접 review를 열어
글줄과 도장을 판독했다. 같은 마스크의 남은 미대응은 704픽셀이다. 그중 오른쪽 용지
가장자리의 1px 세로선 성분이 358픽셀이며, 이 잔여를 100%로 보고하지 않는다.
#5593 1쪽·#7333 10쪽·#6122 1쪽의 독립 PDF 대조를 끝냈다. 총 12쪽 모두 이전 RGB와 같고,
#5593은 raw 100%를 유지한다. 기대값·원본·독립 PDF를 바꾸지 않았다.
로그는 `logs/overlay-text-native-driver.log`, `overlay-text-controls/after-summary.json`이다.

코드 검토 중 `paragraph_with_frame_geometry`가 줄 수를 유지하면서도 새 좌표를 소유할 수 있는
호출을 확인했다. `e49286e6ba1f36d5f8559ae296da89153c2258f1`은 이러한 재조판 프레임을
원본 저장 줄의 인증 대상으로 사용하지 않게 한다. 현재 이 보완까지 포함한 Native 빌드·같은
18쪽·12쪽 대조군을 재실행 중이다. 아직 앞 후보의 통과를 최신 후보의 결과로 대신하지 않는다.
종료 후 실제 결과에 하락이 없을 때만 Native 전쪽과 최적화 포함 fresh WASM을 이어서 실행한다.

### 반올림 없는 감사와 후속 원인 추적

직전 `e4efb59b8c` 전 766쪽 PNG·PDF raster의 해시를 원 manifest와 대조하고, 고정 232 임계값과
2px 반경으로 미대응 수를 다시 직접 계산했다. HWPX 83쪽·HWP 74쪽이 정확히 0픽셀이다.
표시 raw 100%와 이번에는 같은 수지만, 남은 609쪽은 개선이 필요하다.
`inline-origin-e4efb59b8c/whole-native-exact-audit.json`과
`logs/inline-origin-whole-native-exact-audit.log`를 보존한다.

133쪽 쪽번호 PDF의 실제 `Tf`는 92이고 CTM은 0.120pt, `Tz`는 95.652176%다.
실제 세로 크기 11.04pt·가로 크기 약 10.56pt이며, 추출기의 단일 크기 10.79733pt를
원래 글꼴 높이로 취급하지 않는다. raw 연산자를 `footer-p133-pdf-operators.json`에 보존했다.
이 관측만으로 한컴의 일반 반올림 규칙을 확정하거나 수치를 맞춘 상수를 추가하지 않는다.

140쪽은 HWP/HWPX의 본문 글줄·그림 위치가 같지만 HWP의 최종 표 조각이 약 36.8px 짧다.
원본 HWPX section3 p64의 표 1179766215는 common 높이 55178HU·셀 높이 110374HU다.
Native HWPX 마지막 조각은 약 735.9px, HWP는 699.1px다. HWP 그림은 Square,
HWPX 그림은 TopAndBottom으로 관측되어, 원본 바이트와 선언 높이·사용한 물리 원장부터
추가 대조한다. 이 기록은 원인 확정이나 원본 속성의 임의 변경을 승인하는 근거가 아니다.

사용자는 2026-10-23에 현재 브랜치의 PR 준비가 완료되면 PR·CI 모니터링·머지·후속 처리까지
진행하도록 자동 승인했다. #8153의 전쪽 100%·중요 영역 회귀·필수 검증 완료 조건은 유지한다.

### 140쪽 선언 프레임 원시 대조와 공통 수용 경로

독립 CFB 리더로 HWP Section3 p64의 CTRL_HEADER와 LIST_HEADER를 직접 읽었다.
폭·표 높이는 39543/55178HU, 셀 높이는 110374HU로 HWPX와 같다.
원시 offset·hex는 `p140-original-records.json`에 보존한다. 따라서 선언 높이가
포맷마다 달라서 생긴 차이로 판단하지 않는다. 첫 55178HU를 수용한 뒤 잔여는
`110374 - 55178 = 55196HU`이며 마지막 내용 끝보다 큰 원래 물리 공간이다.

기존 해결 PR #4763(`f8c784235d7`)은 native HWP의 실제 fragment 소비를 HWPX 전용
object-height advance로 덮지 않도록 분리했다. `direct_hwpx_cell_has_declared_stored_frame`의
프로필 조건을 제거하는 방식으로 보정하지 않는다. 선행 공통 계획은 PR #7617 보정
`a9c25f39b`의 `saved_first_row_opening_frame`이며, 원본 선언·실제 CellUnit reset·줄
범위로 opening/remaining을 생산해 scan·emit·paint와 accepted ledger에 함께 전달한다.
이 계획은 현재 첫 프레임 문단에 개체가 있으면 거절하므로, 첫 프레임의 실제 TAC 그림
2개를 담은 편람 표는 이 경로를 쓰지 못한다. 둘의 common 높이는 각각 14526HU로
소유 LineSeg의 높이와 같으며 앞뒤 저장 글줄은 0에서 53532HU까지 진행한 뒤 재시작한다.

후보는 원래 미편집 줄 상자가 그림과 바깥여백을 수용하는 TAC 그림을 공통 프레임의
인증 대상에 포함한다. 그림 캡션·비-TAC·중첩 표 유닛 등 다른 공간 계약은 기존 경로다.
저장 그림의 원시 슬롯 소유와 점유 높이를 확인하며, 출력의 끝선 좌표를 고정하거나 clamp하지
않는다. 원래 줄들의 잉크 끝·뒤 간격과 선언 프레임, 현재 잔여 내용의 수용 검사는 유지한다.
같은 `(cut, opening, remaining)`이 스캔 요구 높이·실제 수용·원장·최종 paint까지 소비되는지
실행 trace와 독립 Print 139·140·141쪽으로 먼저 대조한다. 새 정식 회귀나 기대값은 아직 변경하지 않는다.

### 전쪽에서 검출한 116쪽 회귀와 원점 가정 정정

`e49286e6ba`의 전쪽 HWPX에서 116쪽이 99.22614% → 99.00841%로 0.21773%p
하락했다. 선별 18쪽·대조군 12쪽 통과로 이를 면제하지 않는다. 같은 원본의 이 하락은
현재 후보의 결함이며 기준 갱신·이관 없이 고친다. source table 2015664433/1980200104,
section2 p884/885의 `3㎝` 글줄 두 곳만 y 214.6/327.7px → 210.4/323.4px로 움직였다.
도장 그림·표의 원점과 크기는 같다.

두 셀은 CENTER·height 6761HU, 여백 141HU이며 **첫 저장 vpos가 0**이다.
마지막 줄은 4840+1000HU다. 원본 내부 높이 6479HU와 줄 사다리 5840HU의 차이
639HU는 가운데 정렬에서 319.5HU(4.26px)를 만든다. 이번 구현은 이 로컬 0 원점을
셀 내부의 절대 원점으로 잘못 보아 원래 정렬 공간을 제거했다. 문단 첫 0은 유효한
셀 상단과 로컬 프레임 시작을 구분할 독립 증거가 아니다. Center/Bottom의 로컬 0
사다리는 기존 공통 정렬 프레임이 계속 소유하도록 한다. 86·87쪽의 실제 양수 저장
시작과 단순히 같은 캐시로 취급하지 않는다. 문서 ID·4.26px 상수로 분기하지 않는다.
양수/0 원점의 해당 source·Print 반례를 후속 후보 검증 목록에 함께 포함한다.

### 재검증 큐와 독립 글꼴 수치 확인

`e49286e6ba`의 Native 전 766쪽은 exit 0으로 종료했다. HWPX 116쪽에만 위 0.21773%p
하락이 있으며 HWP 전쪽은 base·직전 후보 대비 하락 0쪽이다. 최저값은 HWPX
92.61726%·HWP 92.61751%다. 따라서 이 후보는 승인 대상이 아니다. fresh WASM 빌드는
컴파일 5분 24초·성공 뒤 최적화 중이며, 이 실행은 첫 후보의 backend별 영향 근거로 보존한다.
최신 보완 후보 `cad634797d`는 116쪽 로컬 원점 복구, 140쪽 TAC 그림 줄의 물리 프레임 인증,
공통 `tac_object_flow_height_hu` 소비를 포함한다. 세 변경 Rust 파일의 rustfmt 검사는
`logs/picture-frame-rustfmt-check.log` exit 0이다. 이 후보의 실제 Native 빌드는 앞서 시작한
WASM 최적화가 끝난 뒤에 실행한다. 검사 목록은 두 형식 115·116·117·139·140·141쪽 및
기존 주요 9쪽, 총 30쪽과 독립 대조군 12쪽이다. 아직 실제 개선 완료로 보고하지 않는다.
실행 전인 기존 빌드 대기 controller만 자식 프로세스가 없음을 확인하고 교체했다.
실제 렌더러·Cargo·wasm-opt와 해당 증적은 중단하거나 제거하지 않았다.

133쪽 쪽번호의 크기 차이는 임의 배율 대신 원본 글꼴과 Windows API로 확인했다.
[LOGFONTW](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-logfontw)의
음수 높이는 character em이며 폭은 평균 글자 폭의 논리 단위다.
[MulDiv](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-muldiv)는
가장 가까운 정수로 반올림한다. HWP가 그린 실제 Print의 CTM 0.120pt는 해당 출력의
600dpi 장치 좌표를 나타낸다. 이 장치 해상도를 HWP 형식 자체의 필수 해상도로 표기하지 않는다.

`win10-ted`(Windows PowerShell)의 실제 `CreateFontW → GetTextMetricsW/GetCharABCWidthsFloatW`
48개 조합을 실행했다. 9·10·11·15pt와 자연 폭·95·97·100% 요청을 대조했으며 exit 0이다.
선택된 프로그램을 Windows `GetFontData`에서 메모리로 읽어 SHA만 산출했다. Haansoft Batang,
KoPubBatang Light, KoPubDotum Light 세 프로그램은 Linux 공급본과 SHA-256이 각각 같다.
폰트 프로그램을 복사·배포하거나 원격 작업 파일을 만들지 않았다. 초기 긴 EncodedCommand는
Windows 명령 길이 제한에서 실패해 압축된 bootstrap으로 수정했다. 계정·토큰 정보는 사용하지 않았다.

Haansoft Batang의 실제 unitsPerEm/xAvgCharWidth는 1024/512이며, 11pt의 장치 em 92에서
Windows 자연 평균 폭은 46이다. 95% 요청의 평균 폭 44와 `44/46 = 95.6521739…%`는
PDF의 Tz 95.652176%와 수치상 일치한다. 공식 API·실제 글꼴·독립 Print를 교차한 **메커니즘
후보**이며, 한컴의 호출 자체를 계측했다는 뜻은 아니다. 다른 크기·장평·글꼴과 조판/paint 소비를
확인한 후 구현 여부를 판단한다. 현재 런타임에 600dpi 글꼴 상수나 이 비율을 추가하지 않았다.
`gdi-font-metrics-win10-with-font-hashes.json`, `logs/gdi-font-metrics-win10-with-font-hashes.log`
및 `probe_gdi_font_metrics_with_hashes.py`에 실제 값·명령을 보존한다.

90쪽은 원본 section2 p634의 image365를 실제로 추적했다. 원본 JPEG 1195×1441과
Print JPEG 870×952의 양자화 표가 다르고, 원본 crop을 이미지 좌표로 환산한 끝점은
870.947568/952.692227px다. 동일 owner·입력/PDF SHA·JPEG 양자화·메타데이터를
`p90-source-print-image-program-audit.json`에 보존했다. 재압축 관측을 공식 HWP 색 변환으로
일반화하지 않으며 수치 맞춤용 tone/마스크 보정을 되살리지 않는다.

### 장치 글꼴 계산의 독립 반례와 수치 예측

`audit_print_font_device_math.py`는 원본 CharShape 높이·언어별 장평, 실제 동일 SHA의 TTF
unitsPerEm/xAvgCharWidth, 현재 Print CTM에서 읽은 장치 해상도만으로 font Tf/Tz를 예측했다.
133쪽 바탕쪽 style649, 본문 style220, 86쪽 직인 style697과 이름 style576의 네 조합이
PDF 원시 연산자와 일치했다. Tz는 PDF에서 관측한 float32의 소수 6자리 표현으로 대조했다.
점수나 PNG 픽셀을 인자로 쓰지 않았다.

144쪽 source section3 p71 style166(11pt·100% KoPub바탕 Light)을 추가하자 처음 식은
98.823532%를 예측했지만 PDF는 100%였다. 원본의 100%가 요청한 정상 장평을 무조건
평균 폭 재계산으로 바꾸는 가정이 틀렸다. 실패 로그 `print-font-device-math-expanded-before.log`
(exit 1)를 유지한다. 정상 장평은 LOGFONT의 width 0에 대응하는 자연 종횡비 계약으로
보존하며, 폭 변경을 요청한 경우만 원본 논리 크기와 평균 폭을 정수 장치 폭으로 환산한다.
이 구분으로 5건을 통과하고, source section3 p72의 실제 문장에 대응하는 145쪽
style20(11pt·99%)까지 추가해 **6/6·exit 0**을 확인했다.

양수 수치에서 `H = source HU × 언어별 상대 크기`, `E = H × D / 7200`,
`em = nearest(E)`, `naturalAvg = nearest(em × xAvg / upem)`이다.
장평 100%는 원래 종횡비를 유지한다. 그 외의 요청 폭은
`requestedAvg = nearest(E × xAvg / upem × sourceRatio)`이며 출력 장평 후보는
`requestedAvg / naturalAvg`다. 이 모델은 10pt·98% 바탕 75/76, 11pt·95% 한컴바탕 44/46,
6pt·98% 돋움 43/44, 11pt·99% 바탕 83/85를 독립 PDF와 일치시켰다.
한컴의 내부 호출을 직접 계측한 증거는 아니며, 아직 렌더러에 반영하지 않았다.
결과는 `print-font-device-math-six-cases.json`, 실행은
`logs/print-font-device-math-six-cases.log`에 있다. 전 383쪽 PDF의 235개 고유 font/Tf/Tz
관측은 `pdf-font-device-pairs-all383.json`에 별도로 보존하며, 같은 쪽 원본 스타일과 연결하지 않은
관측까지 위 6건의 정답 증거로 세지 않는다.

실제 소비 경로는 `resolve_single_char_style → font_sizes/ratios → composed run.text_style →
paragraph_layout의 TextRun/글리프 위치 → LayerBuilder의 text_run_ops 및 글리프 sidecar →
Native SkiaTextReplay/Canvas`다. 새 구현은 논리 줄 상자·기준선·개체 흐름과 장치의 글리프
실현 크기를 구분하고, 실제 선택된 프로그램과 같은 글리프 위치 결과를 소비해야 한다.
backend별 임의 배율이나 glyph source가 다른 fallback의 수치로 대신하지 않는다.
현재 수정한 물리 프레임 후보의 실제 검증을 먼저 확인하고 이 원인의 보정 범위를 결정한다.

직전 `e49286e6ba` fresh WASM은 최적화 포함 30분 34초·exit 0으로 종료했다.
현재 `cad634797d` Native 빌드가 공유 target에서 순차 실행 중이고, 앞 후보의 전쪽 WASM은
별도 고정 package로 실행 중이다. 오래 걸렸다는 이유로 최적화를 생략하거나 기존 package를
최신 후보의 결과로 사용하지 않았다.


### 140쪽 원본 빈 줄의 물리 프레임 인증 — 실제 유닛 진단 후 보정

`cad634797d` Native 빌드와 두 형식 30쪽, 정상 대조군 12쪽은 모두 exit 0으로 종료했다.
116쪽 HWPX는 앞선 정상 출력과 RGB가 같고 86·87쪽 개선도 유지한다. 정상 대조군 12쪽은
모두 기존 PNG와 RGB가 같다. 다만 140쪽은 HWP 96.02275%·HWPX 98.21693%로
불변이다. 새 helper가 존재한다는 이유로 물리 프레임 보정을 완료 판정하지 않는다.

`logs/p140-cad-cell-unit-trace.log`의 실제 유닛 15는 원본 문단 15의
`vpos=53532, line_height=1000, line_spacing=600HU`를 가지지만 내용 범위는 0..0,
높이는 0이다. 유닛 16에서 원본 vpos 0으로 되감긴다. 기존 인증은 내용 범위가 비었다는
이유로 원본 줄까지 거부했다. 원본 표 62의 물리 줄 높이와 내용의 전진량은 서로 다른 값이다.
원본 상하 여백 282HU를 포함하면 빈 줄의 끝 54814HU와 다음 전진 끝 55414HU 사이에
원본 opening 55178HU가 들어간다. 원본 셀 전체 110374HU의 남은 프레임은 55196HU다.
이는 문서별 보정 상수가 아니라 원시 HWP/HWPX 속성·LineSeg와 독립 Print의 대응이다.

공통 `source_physical_lines`는 실제 저장 줄이 한 개이고, 텍스트·control이 없으며,
내용 유닛에서 접힌 빈 spacer만 원본 물리 인증에 돌려준다. 개체 소유·중첩 유닛·재조판 슬롯은
제외한다. 기존 내용 유닛 높이·컷은 바꾸지 않는다. opening 인증과 컷의 저장 reset 인증이
같은 helper를 소비하고, 기존 예약 → emit의 남은 높이 → 다음 행 예산 → partial paint의
같은 프레임 계획을 유지한다. 기본 크기·본문 예산·성장한 내용 검사와 원본 LineSeg 유효성
검사는 그대로다. 이 보정의 실제 빌드·139/140/141쪽·정상 대조군 결과는 아직 검증 전이다.

직전 `e49286e6ba` 전쪽 fresh WASM도 종료 코드 0이다. HWPX 383쪽 최저 92.60426%,
표시 100% 113쪽, HWP 383쪽 최저 92.60462%, 표시 100% 87쪽이다. HWPX 116쪽의
0.19537%p 하락은 위 cad Native에서 복구했으나 새 WASM 검증은 별도로 필요하다.
248쪽의 양쪽 0.00001%p 감소는 앞선 9f 후보부터 있던 변화이며 이번 새 회귀와 구분한다.
전쪽 100%와 정확한 미대응 픽셀 0은 아직 달성하지 않았다.


### fresh WASM 정확한 미대응 수 확인과 원본 글자 소유 대조 정정

`audit_exact_wasm_overlay_text.py`는 e492 전 766쪽의 PNG·PDF PNG SHA와 반경 2를
확인하고 고정 마스크 232로 정확한 미대응 수를 계산했다. HWPX 113쪽·HWP 87쪽이
실제 미대응 0이며, 남은 566쪽의 합계는 각각 114550/116184픽셀이다. 결과는
`overlay-text-e49286e6ba/whole-wasm-exact-audit.json`과
`logs/overlay-text-whole-wasm-exact-audit.log`에 보존한다. 최신 런타임 결과로
대신 보고하지 않으며, 새 head의 116쪽 복구와 물리 프레임 보정은 별도 검증한다.

앞선 글꼴 수치 6건은 같은 쪽에 font/Tf/Tz 쌍이 존재하는지를 확인한 결과였다.
실제 텍스트와의 소유 대응을 더 강하게 확인하려고 PDF의 ToUnicode와 Tj/TJ를 직접
해독했다. 4쪽에서 764개 텍스트 연산자를 읽었고, 원본은 run의 직계 `t`만 대조했다.
이 과정에서 앞서 기록한 section3 p45의 **style220 소유 대응이 잘못됐음**을 확인했다.
호스트 run의 후손 표·도형 안 텍스트를 호스트 글자모양으로 오인한 것이며, 해당 바탕체
설명 본문의 실제 글자모양은 **53**이다. `업무관리시스템…은 과제분류체계를`는
style481의 돋움체 문장이고, 바탕체 style53은 `업무관리시스템…을 활용하려면`이다.
이전 style220 매핑을 구현이나 사양의 독립 증거로 사용하지 않는다.

style53의 원본 10pt·98%로 다시 계산한 Tf83/Tz98.684212는 해당 문장의 PDF 연산자와
일치한다. 144쪽 style166, 145쪽 style20, 86쪽 style697/576도 실제 직계 원문을
해당 PDF 프로그램의 해독 텍스트에서 확인해 5/5 통과했다. 직인·이름은 같은 스타일의
원본 사용이 여러 곳에 있으므로 이 텍스트 검사만으로 특정 표·문단 소유까지 증명하지 않는다.
쪽번호는 field 치환이므로 이 5건에 섞지 않으며 기존 바탕쪽 소유 확인을 별도로 유지한다.
수치 6/6 재검증은 `print-font-device-math-six-cases-source-bound.json`, 텍스트 5/5는
`print-font-device-text-linked-audit.json`과 `logs/print-font-device-text-linked.log`다.
원시 PDF 해독은 `decode_print_text_programs.py`에 보존한다. 같은 숫자 쌍의 존재를
원본 run 소유 증명으로 대신하지 않았고, 실제 렌더러에는 글꼴 수치 변경을 추가하지 않았다.


### 원본 빈 줄 인증 후보의 실제 Native 개선과 전쪽 재검증 시작

`32c679fc12`는 고정 `target/pr-review`, jobs 8의 Native 빌드 6분 50초·exit 0이다.
선행 두 형식 각 15쪽에서 cad 대비 하락은 0쪽이며 **HWP 140쪽만 96.02275% →
98.21670%(+2.19395%p)**로 개선됐다. HWPX 140쪽은 98.21693%를 유지하고,
139·141쪽을 포함한 다른 29쪽은 cad와 RGB가 같다. 두 형식 모두 실제 383쪽이다.
12쪽 정상 대조군도 모두 RGB가 같고 driver는 exit 0이다. 관련 실행은
`logs/physical-empty-native-build.log`, `logs/physical-empty-native-driver.log`다.

140쪽 review PNG를 직접 읽어 표 외곽과 하단 괘선이 Print와 대응함을 확인했다.
렌더 트리에서 변경된 것은 Table·Cell 높이 및 아래/양옆 괘선 5개 노드뿐이다.
모든 자식 수·글자·그림의 기하는 같고, 원래 셀 내부 (118,152)-(640,846)의 RGB도 같다.
독립 Print 왼쪽 괘선은 96dpi 환산 y149.2653→884.8734px이며 후보 하단은 약885.0px다.
독립 PDF의 실제 path, 원본 HU, 두 source SHA와 변경 노드를
`physical-empty-32c679fc12/p140-physical-frame-evidence.json`에 연결했다.
스크린샷 색·작은 글자 및 바탕쪽 오른쪽 선은 남아 있어 raw 100% 완료는 아니다.

실제 선행 검증이 끝난 뒤 `post_focus_physical_empty.py`는 전쪽 Native와 fresh WASM을
시작했다. Native는 같은 producer의 15쪽만 재사용하고 나머지 368쪽을 형식별로 새로
출력한다. fresh WASM은 같은 32c source·공유 target·jobs 8·codegen 16으로 별도 package를
빌드하고 최적화 후 766쪽을 출력한다. 다른 source의 PNG·package를 최신 결과로 섞지 않는다.
이 문단 시점의 두 작업은 실행 중이며, 종료 코드·전쪽 결과·정확한 미대응 0 확인은 아직
남아 있다. 로그는 `logs/physical-empty-whole-native.log`,
`logs/physical-empty-fresh-wasm-build.log`, `logs/physical-empty-whole-wasm.log`다.


### 다음 원인의 분리 진단: 원본 대체 글꼴과 실제 측정 face

원본 Latin face `Dinbol`은 `한컴바탕`을 대체 TTF로 저장하고 있고, 독립 Print의 133쪽
번호 125는 실제 HBATANG 프로그램을 사용한다. 기본 후보의 번호 bbox는 x608.9/w20.8px,
실제 공급을 확인한 `Haansoft Batang`을 기존 FontEnvironment로 명시한 진단은
x607.2/w24.3px다. 독립 PDF 첫 글자 원점은 x607.19999px다. 원본·PDF·DPI·장평·마스크나
device 크기를 변경하지 않았으며, 진단에서 Native 133쪽은 실제 미대응 0(raw100)이 됐다.
기본 실행에서의 실제 paint 선택 프로그램이 동일하다는 증명으로 확대하지 않는다.

이는 **별도 font environment 진단**이다. 기본 설정의 전쪽 TSV와 섞거나 기본 경로가
해결됐다고 판정하지 않는다. 기본 style resolver는 원본 chain을 넘기지만 실제 host의
공급 결과를 일반 글자 측정에 확정하는 계약이 아직 없다. 임의 문서별 치환이나
`메트릭 표 없음 = 글꼴 설치 안 됨` 추정 대신, 공급자가 실제로 선택한 원본/대체 프로그램과
논리 측정의 face가 같도록 하는 공통 처리가 필요하다. 기존 `FontEnvironment`, Native의
`NativePngExporter::prepare_document`/Skia 선택, WASM의 실제 FontFace 설치를 소비 지점으로
확인한다. 현재 루트에 새 글꼴 런타임 코드는 추가하지 않았다.

같은 진단의 Native 375쪽은 raw100 유지, 86쪽은 98.51749→98.51702로 0.00047%p
감소했다. 기존 e492 package를 사용한 fresh WASM 진단 6쪽도 383쪽을 유지했고,
133·375쪽은 두 형식 모두 raw100, 86쪽은 둘 다 98.47191%다. 이는 HWPX 기본
98.47227/HWP 기본98.47246보다 각각 0.00036/0.00055%p 낮다. 이 반례를 생략하지 않으며
해당 설정을 그대로 전쪽 개선이나 PR 완료로 채택하지 않는다. source 고정과 조건 차이는
`footer-fallback-diagnostic/`의 명령·TSV·producer 기록, 로그
`logs/footer-fallback-diagnostic.log`, `logs/footer-fallback-wasm-diagnostic.log`에 있다.
Native 133쪽의 정확한 미대응 확인은 `p133-exact-audit.json`에 보존한다.

32c 전쪽 HWPX Native 383쪽은 최저 92.61726%, 표시100 83쪽, base·직전 후보 대비
하락 0쪽으로 종료했다. HWP 전쪽은 실행 중이다. fresh WASM 컴파일은 7분47초·성공이며
wasm-opt가 실제 실행 중이다. 전쪽 Native/WASM 완료 후 PNG 해시·정확한 미대응 수를
검사하는 `after_physical_empty_exact.py`를 같은 실행의 종료 코드 확인 뒤 순차 연결했다.


### 회귀 등록 시 기존 검사 동시 정리 지침과 편람 검사 선행 점검

작업지시자의 추가 지시에 따라 새 회귀를 등록할 때 같은 원본·동작의 기존 검사를 함께
점검하고, 잘못된 기대값·단위·전제는 수정하거나 근거를 남겨 제거하도록 AGENTS·CLAUDE와
`visual_fixture_evidence.md`를 보완했다. 정확한 검사·시각 선행 조건은 유지하고, 유효한 다른
보호 범위를 함께 지우지 않는다. 별도 이슈 이관은 기존 원본 코드 전체 보존 절차를 따른다.
변경한 3개 문서의 상대 링크와 새 heading은 정본 checker의 `collect_broken_links`로 확인했고
`logs/regression-registration-guidance-links.log` exit 0, `git diff --check`도 통과했다.

직접 편람 이름을 참조하는 Rust 테스트 파일은 32개다. 파일 전체가 편람만 사용하는 것은
아니므로 파일 수를 검사 건수로 보고하지 않는다. `existing-regression-review-inventory.json`에
원본 참조 경로·검사 선언·첫 수동 점검 4개 파일을 보존했다. 144쪽 원본 4251HU 정사각형과
공통 y, 130쪽 그림-종이 포함 관계, 그림/외곽선의 동일 프레임은 의미 있는 원본·관계 계약이다.
좁은 수치 허용 오차만 보고 제거하지 않는다. 기존 ±2px 위치를 둔 3·130·144·163·241·289·290·
297·375쪽은 직전 Native/WASM 전쪽 자료에서 모두 양쪽 raw98% 이상이다. 최종 코드에서도
그 조건을 유지하는지 다시 확인하며, 새 전쪽 검사 등록 전에 나머지 기존 검사도 같은 기준으로
판정한다. 아직 잘못됐다는 독립 근거가 없는 검사까지 수정·제거했다고 보고하지 않는다.

32c Native 전 766쪽도 exit 0으로 종료했다. HWPX 최저92.61726%/표시100 83쪽,
HWP 최저92.61751%/표시100 74쪽이며 base 대비 하락은 양쪽 0쪽이다. 직전 e492와의 비교에는
HWP 116쪽 99.24234→99.23389(-0.00845%p)가 남는다. 이는 8e/cad에서 원본 CENTER 로컬0
프레임을 복구한 결과이며 base99.23389와 같다. e492의 점수상 상승만으로 잘못된 프레임을
다시 수용하지 않는다. 독립 Print 원점·116쪽 내용과 최종 WASM에서 계속 대조한다.


### 실제 공급 기반 공통 대체 face 전달 계획

다음 구현은 HWP5 표 19·21 대체 글꼴과 HWPX `substFont`의 원본 부재 계약을 따른다.
정적 메트릭 표가 없다는 이유로 원 글꼴을 없다고 판단하지 않는다. 원본의 TTF·비임베드
선언과 명시 대체 TTF를 대상으로 실제 공급자가 원 face 부재와 대체 프로그램 선택을 확인한
경우에만 공통 FontEnvironment로 측정·paint face를 확정한다. 이미 적용한 정적 치환·HFT·
내장 글꼴·사용자 명시 환경은 보존한다. 같은 source family의 사용 style마다 선택이 일치해야 한다.

생산/소비는 `원본 font/charShape 요청 → 같은 renderer의 실제 공급 snapshot → 공통 host
font environment → resolve_render_styles의 font family/metric face → composer/줄 폭 →
paragraph TextRun → 공통 paint/각 backend`다. 사용자 환경과 공급자 관측 환경은 분리하고,
공급 변경·문서 교체·사용자 환경 변경에서 이전 관측을 재사용하지 않는다. 원본 IR은 수정하지
않는다. 선행 반례는 원 face가 설치된 경우, 공급 불명, 서로 다른 style 결과, 문서/공급 교체다.
Native 실제 선택부터 연결하고 WASM도 같은 공급 근거를 전달하도록 이어서 구현·재검증한다.
현재 32c의 전쪽 검증은 고정 producer/package로 그대로 완료하며 새 소스의 증거로 바꾸지 않는다.


실제 구현 후보는 사용자 선언과 host 관측을 별도로 저장한다. 원본 요청이 바뀌면 이전
관측이 조판에 들어가지 않고, 공급자가 다음 준비에서 원 face 존재/불명(None)을 반환하면
관측 환경을 제거한다. 문서 교체·사용자 환경 변경도 관측을 폐기한다. 실제 family와 원본
프로그램 SHA-256·collection face index를 보존하고, Native 출력 JSON의 원본 입력 SHA와
함께 제공한다. 원 face 부재를 검증하지 않은 플랫폼의 불완전 설치 목록은 사용하지 않는다.

Native PNG는 실제 Skia family/postscript 후보와 공급 SFNT에서 선택한다. generic/legacy
fallback이 발견됐다는 이유로 원 face가 설치됐다고 판정하지 않는다. 모든 사용 style의 target
family가 같아야 공통 환경에 반영한다. 내장/HFT·정적 치환·사용자 명시 선택은 건드리지 않는다.
WASM은 같은 관측을 공통 prepare 함수로 전달하는 API를 사용하며, 원본·공급 프로그램 SHA를
대조하는 Visual Sweep 어댑터를 이어서 연결한다. 아직 후보 빌드·시각·반례 실행 전이다.

### 실제 공급 관측의 backend 연결과 32c 전쪽 검증 완료

`017e4dee18`의 첫 Native 빌드는 digest 배열에 `LowerHex`가 구현되지 않아 exit101로
종료했다. 글꼴·입력 SHA를 바이트별 16진수로 직렬화하는 `9fa523e5e3`으로 수정하고 같은
worktree·공용 target·jobs8에서 다시 빌드 중이다. 실패한 빌드 결과를 시각 통과로 세지 않는다.
로그는 `logs/host-fonts-native-build.log`와 `logs/host-fonts-native-build-retry.log`다.

WASM 어댑터는 Native JSON의 입력 SHA·사용자 환경을 확인하고, 같은 원본 요청·실제
글꼴 프로그램 SHA·collection face를 공급할 때만 공통 측정 환경에 관측을 적용한다.
원 face가 WASM 공급 목록에 있거나 Canvas가 다른 프로그램을 우선하면 실패한다.
공급 사본의 실제 바이트도 SHA로 확인한다. 선택적 `--observed-font-receipt`와 관측 파일
해시를 provenance에 추가했으며 관측 유무는 실행 조건으로 구분한다. 기본 경로의 불완전한
글꼴 목록으로 설치 부재를 추정하는 처리는 추가하지 않았다.

원본/사용자 환경 불일치·collection face 불일치·바이트 변경·선택 순서 충돌·중복/없는 요청·
동일 프로그램의 설치 후 환경 확정을 포함한 Node17건과 기존 Python 어댑터33건이 통과했다.
로그는 `logs/host-fonts-adapter-tests.log`, `logs/host-fonts-python-adapter-tests.log`다.
이는 어댑터 계약 검사이며 실제 fresh WASM·시각 결과로 대신 보고하지 않는다.

이전 32c Native/fresh WASM의 전1532쪽과 PNG 해시·정확한 미대응 검사는 모두 종료했다.
Native HWPX83/HWP74쪽, WASM HWPX113/HWP87쪽이 실제 미대응0이다. 남은 쪽은 각각
300/309/270/296쪽으로 총1175쪽이다. WASM248쪽의 base 대비 -0.00001%p와 HWP116쪽의
직전 후보 대비 -0.01661%p도 원장에 보존했다. 116쪽은 base 값으로 복구된 결과다.
정확한 원문은 `physical-empty-32c679fc12/whole-{native,wasm}-exact-audit.json`과
`whole-{native,wasm}-summary.json`, 종료 로그는 `logs/physical-empty-exact-driver.log`다.
32c 검증을 새 글꼴 후보의 검증으로 바꾸지 않으며 전쪽100% 목표는 계속 미완료다.

공급 helper 변경도 재사용 판정에 포함하도록 `font_registration_sha256`을 provenance에
추가했다. 동일 package라도 helper 또는 관측 파일이 바뀌면 기존 resume이 거부되는 검사를
기존 generation 계약 파일에 보완했고, 같은 Python 묶음은 34건 통과했다. 새 Native 빌드와
fresh WASM 실제 검증은 별도 진행 중이다.
