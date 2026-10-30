# seo-rii Native PR 통합 검증

## 검증 상태

로컬 검증 진행 중. 기준 `96ccd557592a711f215e7ad28822f4c7a3f00c15`, 통합 제품 소스
`c3f5cbd6d7beecdaa231cd3a40080f2e7f3c8c1e`. 이후 테스트 첫머리 주석은 입력·계약과
시각 증거의 관계를 설명하며 제품 코드나 assertion은 바꾸지 않는다.

[입력·기준 PDF](inputs.json)는 기존 저장소 자원을 그대로 사용한다. PDF의 해당 한컴 세대는
본문·글꼴 대조군 2020→compat 2022, 이미지 crop 2022, 편람 2024다. 임시 실행·전체 PNG는
`output/pr-review/seo-rii-20261030/`에서 만들고 최종 근거만 이 경로에 보존한다.
개인 글꼴 프로그램은 커밋하지 않는다.

## 원인과 실제 소비 경로

| PR | 생산 → 실제 소비 | 검토한 계약 |
| --- | --- | --- |
| #8270 | `NativeDocumentSession::render_region` → `render_region_cached` → `PreparedSkiaPage::plan_region/render_region` → RGBA frame | revision/font generation/page/scale 키, 최대 4개 LRU, 전체 retained byte 한도와 요청 peak를 할당 전에 확인. 편집·undo·IME·font 교체 무효화 |
| #8275 | classifier와 trusted CI workflow → `run-rust-test.mjs` → Native cfg suite | region cache/planner/host surface가 release와 release-test에서 실행되며 최신 devel의 glyph·이미지 검사를 유지 |
| #8276 | `SkiaTilePlan`/공통 `SkiaRegionPlan::for_tiles` → 고정 512px 격자와 2px guard → `render_prepared_raster` → 실제 겹침 영역 copy | viewport와 무관한 tile origin/clip, 한 번에 최대 516²×4 working bitmap. full-page exact crop와 별도 raster 계약 |
| #8279 | `renderer::arrow::{line_heads,connector_heads}` → `validate_direct_pdf_tree` → 기존 Skia Line/Path replay → PDF canvas | 요청한 화살촉 수, 유한 scalar/outline, 몸통·endpoint metadata·cubic tangent 검사. geometry 생산을 PDF에서 재구현하지 않음 |

문단·줄 소속, 측정/배치, pagination·rowspan·이어받기·저장 LineSeg는 변경하지 않는다.
네이티브 모듈은 wasm32에서 제외되며 PDF admission도 native-skia+non-wasm cfg다.
기준 devel의 #7697/#7688 공통 화살촉, #7964의 기존 Direct PDF 제한, #8389의 편람 글꼴·이미지,
#8393의 caption 통합을 확인했다. 새 CI 호출은 Native cfg가 default suite에서 실행되지 않는 범위를
보완하며 기존 호출을 중복시키지 않는다.

## 합성 계약과 실물 증거 구분

- `issue_536_native_region_cache`: 합성 도형·빈 문서의 exact crop, 타일 overlap, LRU/예산/무효화.
- `skia_region_plan`: 합성 페이지 크기와 요청의 격자·stride·바이트 예산, 실제 버퍼와의 일치.
- `issue_536_native_session`: new_blank 저장·재열기, 소유 수명·revision·feature-off 오류.
- `render_p37_direct_pdf_export`: re-03-latin 실제 입력의 API/PDF 구조와 합성 이미지·화살촉
  vector stream 계약. 합성 vector 좌표의 기대값은 #7688 독립 Print의 화살촉 크기·끝점과 PDF CTM에서
  정하며 현재 arrow helper의 반환값을 기대값으로 부르지 않는다.

위 합성 assertion에는 다른 문서의 raw 수치를 붙이지 않는다. 실물 Visual Sweep은 아래 별도 검증이다.
`consumer_probe.rs`는 네 실물 문서를 scale 0.75/1/1.25로 실행하는 검토 도구이며 자동 suite는 아니다.
두 페이지 cache 왕복은 각 실물 입력의 section을 메모리에서 복제한 진단 입력이다. 정상 한컴의
실제 두 페이지 문서로 오인하지 않는다. 편람은 원본 383쪽을 그대로 사용한다.

## 현재 확보한 결과

- 집중 Native 81건, Native lib 4,040건(ignored 12), placeholder 2건 통과.
- Node CI 계약 95건, Python CI 계약 95건, actionlint·workflow shell 구문 검사 통과.
- 실제 네 문서의 prepared/host는 full-page 픽셀과 동일. 후보 왕복 0→1→0은 replay 2/hit 1.
- 후보 Native의 다섯 문서 전쪽 최저 99.66%, 편람 383쪽 최저 99.96%.
- 편람 63쪽·289쪽 Direct PDF 생성과 기준 Print 실루엣 100.00%.
- 최종 baseline 대조·fresh WASM·전체 회귀·lint는 진행 중이며 아직 완료 판정하지 않는다.
