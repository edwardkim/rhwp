# 7445 리베이스 보정19: 그림 프레임·줄 점유·외곽선 소유 체크포인트

Issue: #7445

브랜치 `fix/7445-completed-20261009`, parent `08e020231d8e1c15584ee16dab004cc4e46b19dd`,
base `2e3b4f880bd11d35398e6ea0100846611f13a72d` 위의 로컬 보정이다. 전체 PR 준비 완료가
아니다. 원격 push·PR·게시·merge는 하지 않았고 새 사용자 승인 전 수행하지 않는다.

## 원 계약과 실제 소비 경로

- HWP3 사양 표44의 그림 정보14바이트 위치 bit0을 IR의 테두리 속성으로 보존한다.
  `issue_265.hwp`의 꼬리말 로고만 bit가 있고 본문4그림에는 없다. bit 하나만 끈 독립
  Print638에서 첫 본문 이전의 장치 정밀도 전환도 대조했다. 원본·저장본을 혼동하지 않는다.
- 원PR1279/메인터너658868407c의 HWP 회전 행렬 생성 계약을 HWPX에도 공유한다.
  유효한 기존 raw/group/명시 affine은 유지하며 `rotate_image`와 flip bit19는 바꾸지 않는다.
  독립 Print661/676에서 identity로 잃던30도 회전을 확인했고, 기존#5595→PR5608→5617의
  90도 프레임 계약도 대조했다.
- 공개 삽입 그림1980HU와 위아래4000HU의 독립 한컴 재저장666은 줄 높이5980HU,
  기준선5083HU, 다음 시작6580HU다. 측정·저장 줄·setter의 후속 vpos가 같은 점유를
  소비한다. 여백 제거 시 같은4000HU를 반환하며 본문 내용은 보존한다.
- `common`의 회전 후 점유와 `current`의 회전 전 표시 크기를 공통 transform에 보존한다.
  RenderTree의 점유 bbox는 유지하고 실제 paint frame을 Native/SVG/WebCanvas/Canvas 명령에
  전달한다. 전체·흐름 이미지 JSON의 `imagePaintBBox`를 Studio DOM/CanvasKit이 소비한다.
- 독립 Print676/687의 외곽선은 그림 안쪽 약1CSSpx를 제외한 굵기가 바깥으로 자란다.
  공통 그림+테두리 발행에서 그리는 중심선을 정하고 ROUND와FLAT/SQUARE 모서리를
  Native/SVG/WebCanvas/JSON/CanvasKit에 전달한다. JSON의 추가 계약은 schema1.28이다.
  CanvasKit 사각형도 이미 발행된 회전 transform을 소비한다.

초기의 일반 그림 경로에 테두리가 없다는 가설은 잘못이었다. 일반 `layout_picture_full`에는
캡션 뒤의 기존 테두리 발행이 있어 공통 append 추가 후 두 번 그려졌다. 실제 누락은
LineSeg 없는 InlineFlowPlan 문단 경로였다. 일반 경로는 앞 공통 append 하나를 유지하고
뒤 중복 발행을 제거했다. 독립 셀 그림 대조군700에서 실제2회→1회를 확인했다.

## 실행 결과와 정식 회귀

- Native689/fresh696의697 전쪽: 각18조건243쪽, 총486쪽,90% 미달0이다.
  이는 아래 schema·Studio 표면 보정 전 source의 결과이며 최종713/714 전체로 재인용하지 않는다.
- 최종713 Native/fresh714의715 전쪽: 각6조건51쪽, 총102쪽, 쪽수·누락·중복0,
  Native 최저98.99647%, WASM98.84560%다.102 PNG는697의 대응 PNG와 모두 바이트 동일하다.
  PNG hash·동일 입력/글꼴·생산 경로를 남겨 기존 대표 판독을 연결했다.
- 같은 최종 source의 신규6함수 직접 실행은6 PASS다. Native656에는5 FAIL/1 PASS,
  테두리 보정 전607에는 마지막 원본 비트 함수가 실제FAIL이다. 처음5함수도705에서
  같은 source로5 FAIL→5 PASS를 확인했다.717 최초 테스트 컴파일의 Header/Footer 타입
  오류는 제품의 실패로 집계하지 않았으며 정정 후 결과만 사용한다.
- 자동 prepare 뒤 실제 nextest718:5모듈39건 PASS/0 FAIL, 실행2.307초,
  컴파일 포함284.183초다. 나머지1213건은 해당 필터에서 미선택이며 전체 skip45와 다르다.
  전체 nextest 통과가 아니다. 신규6함수·기존 회전/저장 비트·번호 소유·레이어 계약을 포함한다.
- Studio의 인쇄 표면 크기 실패를 실제 화면 및 새 단위 검사로 확인했다. CanvasKit만 원본
  조판 크기1122.5를 사용해1123px, Canvas2D는 발행된 인쇄 종이841pt에서1122px였다.
  `PageRenderer → PageLayerTree.pageWidth/pageHeight → surface`를 공유하게 했다.
  원본 pageInfo는 편집/여백 정보로 유지한다.711은6 PASS/2 FAIL→8 PASS, tsc exit0이다.

Native all-target Clippy723도 종료 코드0이다. 전체 lint 묶음의 완료를 뜻하지 않는다.

Native713 빌드와 Mac locked wrapper `--target web --out-dir pkg --no-opt` fresh714는
모두 exit0이다.714는 실제 root package 재컴파일205.712초이며 Docker 최적화 통과가 아니다.
Root pkg/Studio public/frozen package의 JS·WASM hash가 같다.
JS `63c587e4b072f0a65416e65a0e86bf00b2971cdfb0f46cfda7f150449dd30524`,
WASM `86508a76268bb5cf34d26d698fd8a81248bf383d97b2bcda618f6cf2fc252d1e`다.

[입력·정본 TSV·대표 review/overlay](../pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/picture-source-frame/README.md),
[정식 fixture와 독립 PDF](../../tests/fixtures/issue7445_picture_source_frame/README.md)를 연결한다.
로그·진단·중간 JSON·미채택 입력은 ignored output의 같은 rebase 폴더 repair633–722에 보존한다.

## 실제 Studio와 아직 수용하지 않은 범위

실제 제품의 `open-document-bytes → initializeDocument → CanvasView`로 두 완료 입력을 열었다.
Canvas2D·CanvasKit compat/default software에서 Print/Screen 각각6개 화면을 수집했다.
각 프로필의12개 실제 초기 JS/WASM 응답이 frozen714와 일치하고 backend fallback은 없다.
Print는 인쇄 표면 크기 보정 후794×1122로 같으며 backend 비교4건 PASS다.

다만716/719/720/721 프로세스는 글꼴 중복 메모리 캐시 알림·중단 fetch의 본문 증적 수집
오류로 exit1이다. 매 navigation의 실제 OldHangul GET 본문/hash는 별도 남았지만, 그 사실로
읽지 못한 응답이나 실제 CSS 선택 face의 모든 바이트를 입증했다고 쓰지 않는다.
Page.getResourceContent 재조회도 빈 캐시 알림을 해결하지 못했다. HTTP403으로 문서 열기
이전에 중단된707은 제외하고, 이후 동일 입력 바이트를 제품 열기 이벤트로 전달했다.
서버 접근 제한·글꼴/CanvasKit 준비 guard·픽셀 임계값은 바꾸지 않았다.

Screen의4개 report-only 픽셀 비교는 기본0.5% 예산을 초과한다.722의 올바르게 이름 붙인
backend 대조에서 그림·회전·모서리·표 소유를 읽었으며 이미지 샘플링·글꼴·편집 가이드
차이는 남아 있다. backend 대조의98.97525/99.24138%를 독립 PDF gate로 사용하지 않는다.
이 결과는 actual36 완료나 모든 소비 경로 수용이 아니다.

PR 보류는 계속 유지한다. picture-start7 8쪽의 기존 고정4a07 base99.79893→현재99.40163%
(0.39730%p 하락)은703에서 그림이 아니라 하단 표 가로선의1068개 원시 불일치로 좁혔다.
source 선 중심942.08CSSpx와 독립 Print941.05067CSSpx의 물리 프레임을704에서 대조했다.
일반 규칙·반례를 더 확인하고 공통 소비 경로에서 보정한다. 과학3쪽의0.32810%p 하락,
최신 전체 corpus·전체 nextest·범위별 lint/build/Skia/manifest/unit tiers·actual36도 남는다.
#7445 전체를 닫거나 일부 높은 점수로 이 잔여를 면제하지 않는다.
