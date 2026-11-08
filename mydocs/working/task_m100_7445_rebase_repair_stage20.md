# #7445 리베이스 보정 Stage20 — 저장 줄 흐름과 Native 글리프 폭

- 브랜치: `fix/7445-completed-20261009`
- 부모 HEAD: `c07d208e2b996b254d5484df88d741849928e3da`
- 고정 base: `2e3b4f880bd11d35398e6ea0100846611f13a72d`
- 상태: 로컬 보정 체크포인트. PR 준비 전체 완료가 아니다. 기존 전체 결과를 이번 소스 통과로 재인용하지 않는다.
- ignored 원장: `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/`

## 원인과 공통 소비

1. Print 표 괘선은 원본 논리 상자를 유지하면서 독립 PDF의 물리 종이 CTM을 사용한다. `PagePaintContext::publish_table_border_geometry` → paint builder → 실제 Native/Canvas line op를 연결했다. 표가 직접 발행한 선만 대상으로 하며 셀 내부 도형에는 상속하지 않는다. 임의 pen 확장·device 반올림을 더하지 않았다.
2. 저장 글줄이 있는 텍스트/인라인 수식 셀은 줄 상자와 활성 패딩을 모두 수용한다. 기존 줄-선언 높이 차이가1.5px보다 작다는 조건이 실제 패딩을 버렸다. `stored_text_line_cell_minimum`을 행 측정·rowspan·실제 배치가 함께 소비한다. 편집·합성·독립 개체 셀은 기존 경로를 유지한다.
3. 저장 `column_start`가 이미 문단 왼쪽 여백을 소유할 때 그 여백을 다시 더하지 않는다. 별도 indent는 유지한다. 소유 인증을 TAC 배정 존재 여부와 연결하던 잘못된 조건을 제거했다.
4. Native 본문은 Canvas와 같은 `glyph_fit_positions`·`canvas_cluster_fit_scale`을 소비한다. 이전 통합 `84764fa21c96cd5f8a0e158658e4162a4fb3ce9e`(#7902/#7941)는 Native 적용을 생성 라벨에 제한했다. sample11의 원본 전진 폭과 공급 Dotum의 자연 윤곽 폭이 달라도 Native만 자연 폭으로 그렸다. 얼굴 경로·TTC 공급·bitmap strike 유무를 바꾼 대조군의 PNG가 동일했고 폭 소비 변경에서 개선됐다. 원점·양수 자간·굵기 tracking과 exact GlyphRun/복잡 문자 shaping 경로는 보존한다.
5. 다단의 원본 저장 줄 배치와 typeset의 단순 높이 합이 달랐다. 과학 p118은 중간 줄 상자가 다음 원점과 겹치는데, 조판은 높이를 합하고 각 문단 마지막 줄간격을 버렸다. p119 표 높이는8002HU로 맞았지만 생산 원점245.1733CSS가 실제 저장241.4667CSS보다 낮았다. 기존 `stored_content_bottom`이 인증한 같은 단 축·현재 원본 메트릭의 최대 점유 끝 + 마지막 간격 + 뒤 간격을 후속 흐름으로 공유했다. `stored_successor_flow` → 확정 inline placement → layout table 소비까지 확인했다. fit 높이와 후속 전진을 구분하며 미확립 축·재조판·독립 float는 기존 경로다. 문서/쪽 예외나 paint 오프셋은 없다.

독립 Hancom2020 재저장에서도 과학 p118/119/120의 줄 원점·높이·표 크기가 보존됐다(`repair757`). 원본 prefix를 수식/일반 글자/표 선두로 바꾼 독립 출력(`repair768`)은 일괄 baseline 이동 가설을 반증했다. 이 자료는 로컬 진단이며 새 fixture로 추가하지 않았다.

## 시각 결과와 판독

과학: 같은 원본 `samples/exam_science.hwp` SHA `22d29786a80d68a9b2ad9294c2dab4915e0eced941e790e37390b14312b8b6a8`, 독립 Hancom2020 Print PDF SHA `1c828fef29d7761d8ef46c8da50cc59936a95128920453ca5cc0d6440ac82d14`,96DPI/compat2022/사용자 Fonts, canonical PDFium smooth_rgb_v1/raw2px90%.

| 쪽 | Native 767(흐름 보정 전) | Native 772 | fresh WASM 775 |
|---|---:|---:|---:|
|1|98.62957|98.62957|98.61966|
|2|98.97662|98.97662|98.91532|
|3|95.37494|97.80906|97.79035|
|4|93.72715|96.14438|96.06833|

Native/fresh WASM 각각4/4쪽,90% 미만·누락0. 767은 직전 후보이며 upstream 기준이 아니다. 최신 고정 base 전체 비교는 남아 있다. 2·3·4쪽 review/overlay를 판독했다. 표와 후속 문단의 큰 세로 drift는 복구됐고,3·4쪽 인라인 수식 주변 본문 baseline의 미세 차이는 남아 있다. 이 두 쪽을 px 완료 페이지로 고정하지 않았다.

[과학4쪽 Native review](../pr/assets/issue7445/completed-20261009/stage20-20261021/science-native-p004-review.png) · [overlay](../pr/assets/issue7445/completed-20261009/stage20-20261021/science-native-p004-overlay.png)

sample11: `repair769` Native765/fresh WASM751 각각151/151쪽, 최저90.66264/90.66313%, 미달0. Native 2쪽93.49501→98.10425%,50쪽89.13433→90.66264%로 개선했다. 이는 Native 글리프 수정의 전쪽 증거이며, 이후 다단 변경771을 포함한 최신 소스 전체 검증으로 대신 쓰지 않는다. 2·50쪽 양 backend 판독에서 본문 폭 개선을 확인했다. PDF의 PC/라우터 아래 회색·갈색 면은 overlay에 그대로 남아 있으며 마스킹하지 않았다.

[sample11 2쪽 Native review](../pr/assets/issue7445/completed-20261009/stage20-20261021/sample11-native-p002-review.png) · [overlay](../pr/assets/issue7445/completed-20261009/stage20-20261021/sample11-native-p002-overlay.png)

## 회귀 검출

- `native_skia_public_paint_contract`의 기존 폭 검사를 작은 MIT 합성 글꼴 `RHWPHostFixture-Regular.ttf`로 바꿨다. Noto/system 설치에 의존하지 않는다. 자연 전진900/1000em의 A에 좁은 원본 전진을 주어 실제 Native raster가 다음 글자를 침범하는지 검사한다. 동일 테스트/글꼴에서 보정 전748 FAIL(ink10px), 보정 후771 PASS(`repair773`). 양수 배분 간격 분리와 음수 자간 반례도 유지했다.
- 새 `issue_7445_stored_multicolumn_flow`: 분수 셀 글줄 포함, 문단 왼쪽 여백 단일 소비, 후속 표·본문의 원본 상대 간격과 줄 순서/누락/중복, 완료2쪽의 실제 Print 구분선 좌표를 검사한다. 이전713에 실제4FAIL → 현재 자동 nextest4PASS(`repair779`).
- px 페이지는 두 backend raw 최솟값이 높은 과학2쪽(98.91532%)을 선택했다. 독립 PDF 원시 선 `(422.88,249.172)→(570.72,249.172)pt`,MediaBox1116pt를 CSS96으로 변환한 끝점을 ±2px로 보호한다. 후보 bbox에서 역산하지 않았다. 원문 소유를 통해 찾은 최종 Print paint의 line op도 검사하고 관계 검사도 유지한다.
- 정상7049/7150, 문단 margin7407/7816, 그림 source frame, 기존 Native raster와 새 관계3건의 자동 nextest24PASS(`repair778`,160.866초). 네 번째 px 검사 추가/prepare 뒤 해당 최종 case4PASS(`repair779`). 중복3건을 합산해 전체 회귀 건수로 쓰지 않는다.

`.log`는 ignored output에만 보존했다. assets의 전쪽 TSV·독립 좌표·producer·대표 PNG와 `evidence.json`이 출처다. Native771 빌드108초, Mac locked WASM774211.160초 exit0이며 web/pkg/no-opt 로컬 대체 빌드다. root pkg·Studio public·동결 pkg JS/WASM hash가 같고 실제 Studio 초기화 응답 확인은 남았다.

## 남은 범위

- 최신 고정 upstream/devel과 후보의 전체 대상 Native/fresh WASM TSV·0.1%p 하락 검토.
- 새 소스의 전체 nextest·Clippy/build/Skia 등 범위 게이트 및 실제 소비36건.
- 과학3·4쪽에 남은 인라인 본문 baseline 차이의 원인·독립 출력 대조.
- latest head/base 확인과 최종 증적 정리 뒤 사용자 PR 승인 요청. 현재 remote push/PR 생성은 하지 않았다.

종료된 진단 라이브러리23개는 gzip 해제 SHA 동일성을 확인한 뒤 무손실 압축해1.357GiB를 확보했다(`repair777-closed-library-compression/result.json`). 원본·PDF·PNG·TSV·로그·현재 producer·공유 target은 변경하지 않았다. 이전 rlib 원문은 같은 폴더의 `library-archive.json`과 `.rlib.gz`에서 정확히 복원할 수 있다.
