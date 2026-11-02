# #8309 최신 devel 검증 증적

## source와 보정

제출 직전 devel은 `04338dc242ff1bd9fab01290b0d90bafcddc5aab`이다. #8447의 검토 기록·
시각 증적·신규 기준 입력만 추가되어 `f72fba39d5651673a6da74a10bc07406a358d269`로 병합했다. 제품·테스트·빌드·workflow와
실측 입력 35개의 바이트가 같음을 [최종 base 보존 기록](final-base-sync.json)에서 확인했다.
아래 `0c1e5a6…`는 실제 빌드·전체 검증을 실행한 base이며 검증 결과를 그대로 유지한다.
새 제출 head의 GitHub CI는 별도로 완료한다.

최신 base는 `0c1e5a6fc20f4ac3e950b7377f831928ad52f5e0`, 후보 제품 source는
`c4686b913cb13ed7a161f941b8e6eadb2be37171`이다. 검증 중 #8438과 후속 #8445가 반영되어
기존 base `c88657d9c3`의 결과와 분리해 다시 실행했다. 기존 결과는
`../pr8309-maintainer-20261102/README.md`와 commit `67501ea78b`에 동결했다.

원 기여 `a9836a959015e9f4709d1b659994b7316f812289`를 보존했다. 최신 base와 제품 차이는
새 글상자 raw 기본값 0과 공통 `write_text_box_header`의 semantic 정렬 비트 생성뿐이다
(두 파일, 9줄 추가·2줄 삭제). 원 회귀 assertion/helper는 그대로이며 첫머리 문서 주석만 보강했다.

생성/속성 편집의 `vertical_align` → 공통 payload writer의 bit 5–6 → HWP 파서의 정렬 →
`shape_layout`의 자식 텍스트 정렬이 실제 경로다. 다른 raw 비트·추가 바이트·측정과 쓰기의
동일 writer·Storage 한도·I/O 오류 전달을 유지한다. 내부 offset 4의 bit 5–6을 셀/캡션의
u32 필드 offset 2 기준 bit 21–22와 구분한다. 새 사양 정오표를 주장하지 않는다.
측정·배치·쪽 분할·이어받기의 알고리즘을 바꾸거나 문서 ID/수치 예외를 추가하지 않았다.

최신 base의 실제 Native에서 원 회귀 3건은 모두 FAIL, 후보는 같은 assertion으로 3/3 PASS다.
`storage-regression-results.json`이 해당 결과다. `base-probe.json`은 최신 base의 최적화 WASM에서도
기본 Top·편집 Bottom·재편집 Top이 HWP 저장 후 Center로 바뀜을 보여 준다. HWPX는 보존된다.
빈 글상자의 API/저장 검사와 다음 실제 글자가 있는 시각 입력을 구분한다.

base 빌드는 독립 검토 worktree에서 제품 파일 두 개만 base 바이트로 바꾸어 전체 runtime
source가 base와 같음을 확인한 뒤 실행했다. Git HEAD는 후보 merge 상태였다. `finally`에서
후보 바이트를 복원했으며 이 차이는 `producer.json`의 `base_build_snapshot`에 명시했다.
기준/후보의 실행 파일과 package를 분리하고 각 파일의 SHA-256을 기록했다.

## 입력과 독립 Print

`tests/fixtures/pr8309-latest/`는 실제 API로 만든 공개 합성 입력이다. 18000×12000 HWPUNIT
글상자에 `Text box alignment ABC 123`을 넣고 기본 Top → Center → Bottom → Top으로 편집했다.
한컴 생성 원본이라고 표시하지 않는다. 재현은 `reproduce.mjs`를 사용한다.

- `textbox-reference-{top,center,bottom}.hwpx`: 최신 base의 올바른 HWPX 저장 경로에서 생성.
- `textbox-saved-{default,center,bottom,top}.{hwp,hwpx}`: fresh 후보 WASM의 실제 저장 바이트.
  HWPX는 각 기준 입력과 바이트가 같고, 마지막 Top은 default와 두 형식 모두 같다.
- `pdf/pr8309-latest/`: 각 기준 HWPX 3개와 후보 HWP 3개의 독립 한컴 2024 Print PDF.
  HWP 자체의 PDF를 확보했으며 HWPX PDF로 대신하지 않았다.

이번 입력 6개는 이전 검증의 입력과 바이트가 같아 해당 Print를 해시 검증 후 재사용했다.
새 Print를 실행한 것으로 표시하지 않는다. `reference-print-execution.json`과
`saved-print-execution.json`에 원 job·원 입력/PDF·동일 입력 해시·재사용 상태를 보존했다.
실제 원 실행은 한컴 13.0.0.3901, PrintMethod 0, `frame_print_to_pdf_ex_one_up`이며
`font_scope=verified`다. 파일 이동 후에도 PDF 해시가 같다.

다음은 이미 커밋된 공개 대조군과 독립 Print다. 원본/PDF SHA-256과 출처는 `inputs.json`에 있다.

- #8134/#8393의 `samples/issue8122/textbox_valign_no_lineseg.hwp`와
  `pdf/issue8122/textbox_valign_no_lineseg-webhwp-print.pdf`.
- 새 base #8399/#8405/#8438의 `samples/issue8399/{A,D,E,P}.{hwp,hwpx}` 및
  `pdf/issue8399/<이름>-<형식>-2024.pdf` 8쌍.

이들 대조군의 기존 PDF를 다시 출력한 것으로 표시하지 않는다. 입력·폰트 프로그램을 원격에
추가 게시하지 않았으며 비공개 corpus를 사용하지 않았다. 폰트 프로그램·WASM·실행 로그·서버
주소·변환 capability·인증 설정은 커밋하지 않는다.

## 전쪽 시각 결과

2026-11-02에 Native Skia와 **새로 최적화한 기준/후보 WASM Canvas2D**에서 직접 PNG를 출력했다.
print profile·96dpi·raw 2px 실루엣(`threshold_boundary_color_support_v1`)을 사용했다.
새 정렬 입력과 #8399는 compat 2024, 기존 #8122는 compat 2022다.

| 입력 (각 1/1쪽) | base Native → 후보 | base WASM → 후보 |
| --- | ---: | ---: |
| 기준 HWPX Top·Center·Bottom, 3개 | 각 100.00% → 100.00% | 각 100.00% → 100.00% |
| 후보 저장 HWP Top·Center·Bottom, 3개 | 각 100.00% → 100.00% | 각 100.00% → 100.00% |
| 기존 #8122, 1개 | 99.55% → 99.55% | 99.42% → 99.42% |
| #8399 A·D·E HWP/HWPX, 6개 | 각 100.00% → 100.00% | 각 100.00% → 100.00% |
| #8399 P HWP/HWPX, 2개 | 각 99.98% → 99.98% | 각 99.98% → 99.98% |

고유 입력 15개, source/backend 4조합의 전쪽 TSV 60개를 `tsv/`에 원문으로 보존했다.
`tsv-files.json`에 원문별 바이트 수와 SHA-256이 있다. 관련 원본·독립 PDF는 각각 1쪽이며
출력·누락·제외 영역·90% 미만 쪽은 모두 없다. 최저는 Native 99.55%·WASM 99.42%다.
폰트 예외를 사용하지 않았다. `visual-results.json`과 `manifests/`에 각 실제 source·PNG·TSV를
연결했다. TSV의 `not_evaluated`를 승인 판정으로 쓰지 않았다.

동일 입력의 base/후보 PNG는 Native 15쌍·WASM 15쌍 모두 바이트가 같다
(`same-input-preservation.json`). 이는 최신 base의 렌더링 기능이 보존된 증거이며 잘못된 이전
저장 바이트가 개선됐다는 증거와 구분한다. 저장 개선은 음성/양성 회귀·실제 저장 재열기·후보
HWP 자체의 독립 Print로 별도 확인했다.

raw 100%를 완전 pixel 일치로 표현하지 않는다. 새 정렬 문서의 pixel match는 Native
99.88517–99.90223%, WASM 99.91458%이며 내용 픽셀 중심 proxy는 각각
59.35638–64.42114%, 68.37074%다. 입력별 일반 pixel/proxy 값은 `metrics/`에 별도로 보존했다.

## 확대 판독과 새 base 기능 보존

전쪽 PNG와 `saved-roundtrip-zoom.png`를 직접 대조했다. Top·Center·Bottom의 내부 여백·
기준선·문자 간격·테두리와 `ABC 123`의 보존을 같은 2배 배율로 확인했다. 잘림·누락·중복은
없다. 이 입력에는 수식·첨자가 없다. 빈 입력의 API 검사를 시각 측정으로 대신하지 않았다.

`flow_A_hwp-detail.png`, `flow_D_hwpx-detail.png`, `flow_E_hwp-detail.png`,
`flow_P_hwpx-detail.png`를 같은 2배 배율로 판독했다. 줄 안 그림의 앞뒤 글자, 어울림 글상자의
외곽과 본문 줄, 그림 옆 줄바꿈, 뒤의 `다음 문단`을 확인했다. 그림 비율·테두리·본문 위치가
같고 내용 누락·겹침이 없다. HWP/HWPX는 전쪽 TSV를 각각 별도로 비교했다.

새 base의 `0a2e306754` (#8399) 도형 밴드 기하·저장 줄 갱신은 현재 생성/속성 setter 안에서
그대로 유지했다. 정렬 속성만 바꿀 때 공통 기하는 바뀌지 않는다. `1903cdf1d2` (#8335)의
번호 범주 bit 26–27은 CommonObjAttr writer이며 TextBox LIST_HEADER 정렬과 다른 payload다.
해당 enum/mask·#8302 HWPX lineage를 원복하지 않았다. 각 실제 기존 회귀도 Native에서 확인한다.
별도 페이지 분할·이어받기 로직은 이번 정렬 보정의 변경 경로가 아니다.

기존 #8122는 `issue8122-detail.png`와 전쪽 PNG에서 제목·본문 줄·도형·후속 설명의 소속·
간격을 대조했다. Print와의 기존 획 굵기·래스터 차이는 남지만 최신 base/후보는 같은 출력이며
이번 보정으로 차이가 늘지 않았다. 대표 review·standalone overlay의 목록과 해시는
`representative-images.json`에 있다.

## 글꼴 환경과 실제 Studio

시각 비교에는 함초롬바탕, #8399의 함초롬돋움, 기존 #8122의 HBATANG/굴림 파일을 명시했다.
HCR 두 파일은 1.980 Build 20140819이며 파일·출처·SHA-256은 `producer.json`에 있다.
글꼴 프로그램을 포함한 SVG는 공개 증적으로 쓰지 않았다.

Rust 회귀는 bubblewrap에서 시스템/개인 폰트와 이전/현재 fidelity 폰트 폴더를 모두 숨겼다.
실제 노출한 전역 fallback은 저장소의 공개 프로그램 5개이며 이번 실행의 목록·해시를
`font-isolation.json`에 기록했다. 시각 비교와 회귀 환경은 서로 다르다.

별도 Studio 7706에서 초기 로딩이 실제 소비한 WASM 응답을 CDP로 관측해 후보 해시
`4734ed402f46493c92ccc107a6a29d78b2e3b66cee22a39b0b237ef38832f255`와 같음을 확인했다.
실제 WasmBridge 속성 변경·export와 파일 입력 재열기를 8회 완료했고 글꼴 대화창에 응답한 뒤
로딩/busy 종료·Canvas·글자·정렬·1쪽 상태를 확인했다. page error는 0이다.
`studio-roundtrip.json`과 스크린샷은 이 UI 검증이며 대체 폰트 스크린샷을 Print 측정으로
대신하지 않았다. TypeScript·전체 npm 검사·제품 build는 `studio-gates.json`에 있다.

필수 Rust gate 20개를 모두 완료했다. 전체 nextest 12,369 PASS(기존 skip 45개), Native lib
4,040 PASS(기존 ignored 12개), Native 집중 57 PASS다. Studio npm 2,716 PASS(기존 skip 2개),
TypeScript·제품 build도 통과했다. 실제 명령·종료 코드·시간은 `rust-gates.json`, 요약은
`gate-summary.json`에 있다. source·test assertion·기대값·ignore·timeout을 보정해 실패를 회피하지 않았다.
재현 스크립트 재실행도 저장 입력 8개와 바이트가 같았다(`reproduction.json`).
