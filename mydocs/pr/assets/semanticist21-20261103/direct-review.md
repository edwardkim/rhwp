# 직접 판독과 최종 재출력의 대응

확인 시각 `2026-11-04T00:25:05.934988+09:00`. 최종 Native와 fresh WASM은 각각 31개 입력·71쪽을 새로 출력했다. 직접 판독한 이전 출력과 각 페이지의 RGB 픽셀을 대조해 62개 입력/backend 조합·142쪽 모두 차이가 0이다. 이전 build head는 `25607fc28d0555caf0ef9b053e181c7236569c18`, 최종 검증 구현 head는 `c6ad3eea59`다.

최종 출력 실행과 대표 패널 재사용을 구분한다. 아래 패널은 이전 출력으로 합성해 직접 읽었으며, 최종 출력 픽셀이 같아 같은 판독 근거를 유지한다. 원시 raster와 render-tree·run JSON을 중복 커밋하지 않는다.

| 대표 패널 | 직접 확인한 의미 |
| --- | --- |
| [form-qualified.png](zoom/form-qualified.png) | 명령·선택 상자·계절 선택·라디오 단추의 바탕 글꼴, 배치와 경계를 같은 프로그램으로 확인했다. 작은 AA/획 두께 차이는 별도 raw 측정에 남긴다. |
| [forms-edited.png](zoom/forms-edited.png) | 본문 5+3개, 셀 4+4개 라디오 단추가 네 물리 행에 한 번씩 보인다. 부모 테두리·각 줄 순서·양식 내부의 원을 독립 Print와 대조했다. |
| [polygon-90.png](zoom/polygon-90.png) | 열린 3점 다각선의 방향·꼭짓점·중심과 열린 변을 독립 Print와 대조했다. |
| [polygon-resize.png](zoom/polygon-resize.png) | 30도·너비 변경의 긴 두 선과 만나는 꼭짓점, 본문과의 상대 위치를 확인했다. |
| [spacing.png](zoom/spacing.png) | 중간 빈 문단과 후속 문단의 세로 간격·앞뒤 문단 순서를 확인했다. |
| [header.png](zoom/header.png) | 70자 두 줄과 빈 문단 뒤의 마지막 머리말이 본문 첫 줄 위에 겹치는 원본 의도와 내용 보존을 확인했다. |
| [split-row-p16.png](zoom/split-row-p16.png) | 분할 칸 끝의 공공데이터 포털 관련 두 줄과 칸 경계, 다음 내용의 순서·누락·중복을 확인했다. |
| [notice-p12.png](zoom/notice-p12.png) | 공통 적용사항 제목·본문 줄바꿈·작은 괄호 글자와 별표 간격, 다음 동시수행과제 제한 제목을 확인했다. |
| [notice-p13.png](zoom/notice-p13.png) | 공통 사항과 지원 기준 사이의 본문 줄바꿈·번호·문단 간격·내용 순서를 확인했다. |
| [notice-tail-p21.png](zoom/notice-tail-p21.png) | 가점 사항 표의 산·학·연 본문과 작은 별표 행·다음 본문의 순서·셀 안 표시를 대조했다. |
| [grow-after.png](zoom/grow-after.png) | 가운데 행만 성장하고 위·아래 행과 표 외곽이 독립 Print와 같은 관계를 유지한다. |
| [grow-nested.png](zoom/grow-nested.png) | 중첩 표의 늘어난 행·다음 아래 행과 부모 clip 경계를 독립 Print와 대조했다. |
| [nested-wide.png](zoom/nested-wide.png) | 선언 폭을 줄이지 않은 여러 겹 표의 가로 경계와 조상 셀에 의한 clipping을 확인했다. |
| [no-lineseg-wrap.png](zoom/no-lineseg-wrap.png) | 그림 단독 마지막 줄의 세로 밴드와 본문 순서, Print의 계단형 sampling과 backend 보간을 구분했다. 끝 캐럿은 문단 글자 모양과 같은 높이를 소비한다. |

변경 영역에 수식은 없다. 작은 문자·괄호·별표·숫자·줄 사이·후속 본문 간격을 같은 배율에서 읽었다. 100% raw도 같은 픽셀이라는 뜻은 아니므로 독립 PDF와의 육안 판독을 점수와 별도로 기록했다. 위 0픽셀 차이는 기준 PDF와의 차이가 아니라, 판독한 후보 출력과 최종 재출력 사이의 차이다.

## 실제 저장 입력

| 입력 | Native/WASM SHA-256 | 연결한 독립 Print |
| --- | --- | --- |
| [notice-edited.hwpx](saved/notice-edited.hwpx) | `567ec4aabba351a3936285479ff31d5e8d11570a033e770e1662347f7b7ce3c0` | 기존 table-insertion-flow/notice-hancom-print.pdf, 24쪽 |
| [grow-nested-edited.hwpx](saved/grow-nested-edited.hwpx) | `4eb9db7414cd35336ec1a965e62b45783af477bcb150fae05fe7ce86b0e5717c` | grow-nested-2024.pdf, 1쪽 |

중첩 대조군의 원래 편집 입력은 잘못된 child 선언 높이 8502 HU를 가진 상태로 보존했다. 독립 한컴 Print는 그 내용의 실제 성장 배치를 보여 준다. 최종 저장본은 같은 내용·배치를 유지하면서 부모·자식 common height를 각각 11748 HU로 기록한다. source cellSz와 무편집 8502 HU는 유지한다. 실제 저장본을 양쪽 backend로 교차 재열기해 같은 독립 Print와 전쪽 비교했다.

## 수치의 범위

이 검토는 silhouette-only TSV의 전쪽 raw와 쪽수·직접 확대 판독을 사용했다. 표준 이미지 전체 비교의 `pixel_match`, `visual_accuracy_proxy_percent`, 일반 `flagged` 분류는 이 실행에서 산출하지 않았다. raw 90% 미만은 양쪽 모두 0쪽이며, 일반 flagged 0으로 바꾸어 보고하지 않는다.

## 최신 devel 통합 후의 직접 판독

# 새 base 통합 후보의 직접 판독 메모

구현 head: `ef6d3029371262204f7b5c3ecf44820c66e54345`. Native 새 출력과 독립 Print를 canonical review·standalone overlay로 생성해 직접 읽었다. WASM과 실제 live·저장 판독은 아직 이 기록에 포함하지 않는다.

- 공고문 physical 12쪽: 공통 적용사항·동시수행과제 제한 제목, 두 본문 상자와 법령 표, 작은 별표·괄호·줄바꿈·뒤 본문 순서를 대조했다. 글꼴 획·자간의 차이는 보이지만 문장·줄 순서와 표 안 포함, 후속 본문은 보존됐다. raw 97.69193%.
- rowbreak physical 16쪽: 앞 조각 마지막 목록 4–8, 공공데이터베이스 표·두 본문 문단·뒤 안내 상자의 순서와 내부 표시를 확인했다. 아래 두 줄을 숨겨야 한다는 옛 전제는 이미 #8187에서 해소된 범위이며, 현재 보이는 꼬리·후속 내용과 hit 소유 계약을 보존한다. serif/sans 획 차이와 AA를 raw 일치율과 구분한다. raw 98.28574%.
- 이 두 변경 영역에 수식은 없다. 최종 완료 판정은 나머지 필수 검사와 fresh WASM의 전쪽·동일 영역 판독 후 작성한다.

- 편람 physical 130쪽·272쪽: 독립 Print와 제목·항목 번호·괄호·각주 별표·줄바꿈·그림/후속 본문을 직접 대조했다. 130쪽의 업무관리 카드 그림과 본문의 상대 배치, 272쪽의 원문자 ①–④·장평·들여쓰기·정책 항목 간격을 보존했다. 각각 raw 99.96465% / 100.00000%이며 전쪽 383쪽 gate도 기준 대비 하락 0쪽으로 통과했다.
- Native 추가 경계 9개를 실제 PNG에서 확대 판독했다. forms-edited는 5+3·4+4개의 원과 검은 바탕·부모 테두리가 네 물리 행에 한 번씩 표시된다. PDF에도 같은 검은 바탕이 있으며 원의 AA·얇은 개체 경계 차이를 배치 누락으로 혼동하지 않는다. Polygon90·30도 resize는 열린 두 선·꼭짓점·본문과의 상대 원점을 보존한다. spacing의 앞/뒤 본문 사이 빈 공간이 Print와 같고 header의 마지막 문단과 본문 겹침은 Print의 원본 의도다. grow-after·grow-nested는 성장한 중간 행과 아래 행, 부모 clip/끝을 유지한다. nested-wide는 선언 폭을 줄이지 않고 조상 셀의 clip을 따른다. no-lineseg-wrap은 여섯 글자 순서와 같은 세로 밴드의 그림을 보존하며 PDF의 계단형 샘플링과 Native의 부드러운 보간을 구분했다(raw 92.89641%). 이 자료는 캐럿 API 검증을 대체하지 않는다.

- 원 PR #8235가 명시한 `samples/hwpx/form-01.hwpx` physical 1쪽도 새로 출력해 명령 단추·선택 상자·계절 선택·라디오 단추·빈 Edit의 위치·글자·3D 테두리·우하단 원 외곽선을 직접 읽었다. Native raw 99.60303%; 이전 HWP 원문 대조와 구분한다. 한국어 캡션이 표시되며 Print의 같은 컨트롤/기준선/간격을 유지한다.
- 이전 검증 c6ad의 같은 71쪽과 새 Native raw를 페이지별 비교했다. 최악 하락은 rowbreak physical 5쪽 −0.71082%p이며 2%p 초과 하락은 0쪽이다. 현재 32개 입력·72쪽 전쪽 최저는 92.16858%다. 새 source의 픽셀이 이전 출력과 같다는 주장은 하지 않는다.

## 원본 TTF 휴먼명조를 명시한 Native 전쪽의 직접 판독

정확한 HMKMM.TTF 경로를 공급한 5쪽(raw 92.93123%)·16쪽(raw 98.51046%)의 canonical review/overlay를 다시 읽었다. 5쪽의 표 2.1–3.1 행·후속 본문 세 줄·각주/쪽 번호가 한 번씩 같은 순서로 표시된다. 본문의 serif 글꼴과 세 줄 나눔이 원본과 맞으며 기존 세로 메트릭·획 두께 차이는 남아 있어 완전한 픽셀 일치로 주장하지 않는다. 16쪽의 상단 마지막 목록 4–8, 두 참고 표, 장 제목·본문 두 문단·아래 인용 상자의 시작과 끝·마지막 두 강조 줄을 대조했다. 원문자 ③과 괄호·페이지 인용·줄바꿈이 보존되며, 문서의 조각 끝을 임의로 숨기지 않았다. 글꼴/AA 차이는 원 PR의 hit 소유·clip 판정과 구분한다. 이 두 출력 producer는 ef6이며, 출처 선언·소비 문서만 달라진 후보에 같은 렌더링 source라는 근거로 재사용한다.

## 새 최적화 WASM 및 실제 live 21쪽 직접 판독

새 WASM producer `696240b004`의 휴먼명조 명시 5쪽(raw 92.88059%)·16쪽(raw 98.49239%)을 같은 Native/Print 영역과 대조했다. 5쪽의 2.1–3.1 표 행, 후속 본문 세 줄·쪽 번호와 16쪽의 마지막 목록·참고 표·장 제목·인용 상자·마지막 강조 두 줄이 한 번씩 같은 순서로 보인다. 글꼴 굵기·기준 메트릭·AA 차이를 raw 완전 일치로 표시하지 않는다.

원 PR #8211 본문의 `notice-live-021` 주장과 같은 입력·실제 53호출·physical 21쪽을 Native/WASM에서 각각 다시 캡처했다. 단순 재열기 fixture의 21쪽으로 대신하지 않았다. 가점 본문 제목·앞의 두 별표 줄·본문 항목의 정렬·상자 포함·작은 URL·감점 상자의 세 항목·앞뒤 줄바꿈과 마지막 꼬리를 읽었다. Native 99.98051% / fresh WASM 99.99883%이며 글꼴 획·박스 경계의 작은 차이와 실제 본문 소유/내용 보존은 구분한다. 쪽 아래 인쇄 쪽 번호 20과 physical 21을 혼동하지 않는다. 새 Native API 캡처도 producer는 696이며 ef6의 재열기 전쪽 raster와 구분한다.

## 새 WASM의 변경 경계와 원 PR 직접 대응

원본 Form01 HWPX의 다섯 양식(명령 단추·체크·선택 상자·라디오·빈 Edit)을 실제 Print와 같은 배율로 읽었다. 한국어 캡션·3D 상자·체크 상태와 라디오 우하단 외곽선이 보인다. 얇은 테두리·AA 차이는 남아 있다. 편집 대조군 16개 라디오는 본문 5+3·셀 4+4개, 네 물리 행과 부모 상자 안에서 한 번씩 보인다. 검은 바탕은 독립 PDF에도 있으며 세로 경계의 작은 차이를 배치 누락과 구분했다.

90도 열린 다각선의 꼭짓점/두 변과 resize의 30도 긴 두 선·본문 원점을 대조했다. spacing의 앞 문단·빈 간격·뒤 문단, header의 70자 두 줄·빈 줄·본문과 겹치는 마지막 머리말, grow-after와 grow-nested의 위/성장한 네 줄/아래·부모 clip·표 끝, nested-wide의 선언 폭과 조상 clip을 확인했다. no-lineseg-wrap은 여섯 글자의 순서·기준선과 가로로 늘어난 작은 bitmap의 같은 세로 밴드가 보존됐다. Print의 계단형 sampling과 backend 보간 차이는 남는다. 새 WASM notice 12·13쪽의 법령 표·앞뒤 본문·작은 별표/괄호·항목 번호·문단 간격을 독립 PDF와 직접 대조했다.

원 PR #8211의 측정 소유 4개 상태도 같은 입력/쪽의 WASM/Print를 읽었다. 새 표의 작은 두 행, 삽입 전 두 큰 표, 빈 host에 삽입 후 새 표와 두 기존 표, 기존 표 다음에 삽입한 작은 두 행의 순서·간격·외곽/내부 선·머리 문단을 보존했다. 네 상태 모두 raw 100%이나 테두리의 작은 획 차이를 픽셀 동일성으로 바꾸어 주장하지 않는다.

제품 source를 바꾸지 않고 위 판독과 전쪽 SHA/metric·API identity를 연결했다. PNG-pair 측정 형식은 자체 provenance 필드가 없으므로 실제 캡처의 생산 head는 post API 실행 기록과 입력 PNG SHA에서 확인했다. 새 임의 renderer 실행으로 provenance를 꾸미지 않았다. 14개 actual/cross 역할 175쪽도 모두 90% 이상·쪽수 일치·누락 0이며 실제 저장 높이와 앞뒤 내용 관계를 확인했다.
