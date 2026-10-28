# 개인정보 원본 두 프레임과 공유 수평선 paint 보정 증적

Issue: #7445 / 관련 #5846. 단계 보고서 C440·C441을 따른다. 이 자료는 개인정보 문서의
해당 보정·회귀 추가 범위다. **18·43쪽 devel 대비 후퇴와 전체 PR 필수 검증은 남아 있다.**

## 입력과 빌드

- 원본: `samples/task2097/75544_pii_bunseok.hwpx`, SHA256
  `f429768ec02e5a849d08c06678cbee029fdc8618ca9e2bdcdbba485087fe71f1`.
- 독립 기준: `pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf`, 한컴2020 Print66쪽, SHA256
  `0fef38f44b3825f1c97869c2720112536504eaf30cec2a2d2af6d4399ffe8770`.
- Native202와 fresh WASM204의 원본 source ledger는 같은 runtime2201파일/hash다.
  Native CLI SHA256: `807b7912cd969e0e7fcdaf06f584ee033d13740c4f543989aec43cf786830cc0`.
  WASM SHA256: `f8c3c6e3786959d8f713e42dc3b93023944819ac84b4ca4215eba232b3a0f26b`.
  JS SHA256: `23f085e36c78d952ddac0d91faed9752f33ee1124db78e290382687410917591`.
- Mac locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다. Docker 최적화
  검증이 아니다. root pkg/Studio public과 실제7736 초기화 응답207의 패키지 신원을 확인했다.
  문서 paint/CanvasKit 준비/actual36까지 검증했다는 뜻은 아니다.
- 192DPI·compat2022·Print·정본 PDFium·내용 원값 이웃2px·90% 문턱을 유지했다.
  pdftoppm 비교는 별도 감사 열이며 정본 값과 섞지 않는다.
- 세 원본 TrueType 공급 경로와 선언/기본 환경을 유지했다. HFT 부분 검토용 프로그램은
  이 원본의 선언과 맞지 않아 공급하지 않는다. 이전 잘못된137 공급 결과는 사용하지 않았다.
  [정정 전후 공급 근거](../pii-declared-font-scope/README.md)를 따른다.

## 전쪽 TSV

네 조건264쪽을 모두 새로 렌더하고 TSV를 산출했다. 각 파일에는1~66쪽이 한 번씩 있다.
같은 backend의 선언/기본 환경 PNG는 전66쪽 byte동일하다. 네 조건의 table/cell/문단 소유
별로 합친 공백 제외 Unicode 본문 순서도 전쪽 일치한다. 이는 출력 경로 간 일치이며
독립 PDF의 텍스트 정답을 대신하지 않는다. 기존 #5846 본문/소유/종료 관계 검사는 유지했다.

| backend | 환경 | 정본 최저 raw | 90% 미만 | 쪽수 | pdftoppm 감사 최저 |
| --- | --- | --- | ---: | --- | --- |
| native | pii66-declared | [94.58132%](after/pii66-declared-native-canonical.tsv) | 0 | 66/66 | [94.12869%](after/pii66-declared-native-pdftoppm-audit.tsv) |
| native | pii66-default | [94.58132%](after/pii66-default-native-canonical.tsv) | 0 | 66/66 | [94.12869%](after/pii66-default-native-pdftoppm-audit.tsv) |
| wasm | pii66-declared | [94.60117%](after/pii66-declared-wasm-canonical.tsv) | 0 | 66/66 | [93.78715%](after/pii66-declared-wasm-pdftoppm-audit.tsv) |
| wasm | pii66-default | [94.60117%](after/pii66-default-wasm-canonical.tsv) | 0 | 66/66 | [93.78715%](after/pii66-default-wasm-pdftoppm-audit.tsv) |

수정 전171/174의 전쪽 TSV는 [기존 공급 범위 증적](../pii-declared-font-scope/README.md)의
`pii66-*-canonical.tsv`·`*-pdftoppm-audit.tsv` 원본을 재사용한다. 같은 입력/PDF/공급 조건이다.
수정 전후 전264쪽 [비교 TSV](before-after.tsv)와 고정4a07 base 전쪽
[비교 TSV](devel-after.tsv), [base 원본](base4a07/)을 함께 보존했다.
색 경계 보정 전 `silhouette_raw_match_percent`를 Decimal로 대조했다.

| 경계 | Native 수정 전 → 후 | fresh WASM 수정 전 → 후 |
| --- | --- | --- |
| 9쪽 수평선 | 99.76631 →100% | 99.77626 →100% |
| 28쪽 수평선 | 99.76699 →100% | 99.77688 →100% |
| 52쪽 수평선 | 99.76689 →100% | 99.77679 →100% |

100%는2px 이웃 관용의 내용 실루엣이다. RGB 완전 일치나 확대 판독 면제를 뜻하지 않는다.
수정 전후 PNG가 바뀐 쪽은 양쪽 모두9/28/52/56/57쪽이다. 나머지61쪽은 같은 backend에서
byte동일하다.32쪽 최저·기존 알려진 글자/획 차이는 byte/hash가 같은 앞 단계 판독과 연결한다.

## 원본 기하와 실제 소비

첫 단일 칸의 원본 common42265HU·총 cell83771HU·후속 원점42072HU가 두 상자를 닫는다.
일반 가용 공간42266HU를 먼저 수용하면 뒤 조각이1HU 작아진다. 원본 두 프레임 인증을
scan 예약과 emit·후속 차감에 공유해 앞42265·뒤41506HU를 보존했다. 검사 공차를 넓히지 않았다.

수평선은 아래 셀의 흰 상단이 논리적으로 한 번 소유한다. 독립 PDF의 원본 graphics 연산은
검은 하단 → 흰 상단 순서다. 색만 흰색으로 합치면 아래 획의 회색 가장자리를 잃는다.
공통 `LineNode.paint_strokes`가 두 paint op를 만들고, 기존 compound component provenance로
둘의 원본 겹침 기하를 보존한다. Native/WASM/CanvasKit JSON/SVG가 같은 결과를 소비한다.
중간199처럼 두 획을 각각 픽셀 중심으로 옮기면 Native 잉크가244로 옅어져 사라지므로 제외했다.
추가 소비 점검에서 `renderPageToCanvasLegacy`와 header/footer 편집 미리보기의 직접
WebCanvas tree 경로는 이 확장을 아직 소비하지 않음을 확인했다. 일반 WASM layer paint와
구분하며, 다음 보정·실제 API 검증에 포함한다. 현재 자료로 해당 경로 완료를 주장하지 않는다.
동일 굵기·종류의 인접 수평 변에 적용하며 수직/다른 굵기·종류·None의 기존 우선순위는 유지한다.

## 회귀와 직접 판독

- 기존 #5846은 수정 전1FAIL → 보정 후1PASS다. 원본 내용·쪽 소속·두 조각과 후속 본문
  관계를 유지하고, 1HU 손실을 검사 제거/완화로 숨기지 않았다.
- 최종 source에서 기존5846 1·1133 3·6860 2·수평 소유1, 총7PASS를 확인했다.
  border grid unit9PASS, fmt와 manifest exit0이다. 다른 library의 필터0건은 더하지 않았다.
- 동일한 새 검사 원문으로 frozen198(수평선 수정 전)2FAIL,199(색 순서만 전달)2FAIL,
  최종202 2PASS를 확인했다. 최종 suite010도 실제2PASS/283선택 제외(0.734초)다.
- 기존 수평선 소유 검사에 실제 공통 paint의 순서·복합 기하·역순 셀 배열과 옆의 정상
  검은 단일 경계를 보강했다. 새 Native PNG 검사는 독립28쪽의 연속 회색 잉크를
  ±2CSSpx에서 확인한다. 소유 노드만 검사하는 상태로 끝내지 않는다.
- 원본 전66쪽의 두 backend 최저90% 이상·28쪽 양쪽 raw100%·독립 Print와 review/overlay
  직접 판독을 먼저 확인했다. 새/수정된 검사 뒤 manifest prepare와 현재 suite 배정을 다시 확인했다.
- 수정 전 PNG는 이미 검증된186의 같은 원문을 hash로 확인한 재사용/합성이다. 수정 후는
  205의 최신 실제 렌더 PNG이며 별도 판독203/209와 byte동일한 것을 확인해 복사했다.
  입력·PDF·비교기·색 임계값232·관용2px를 바꾸지 않았다.

| 직접 판독 | review | overlay |
| --- | --- | --- |
| 수정 전 Native28쪽 | [review](before-native-p028-review.png) | [overlay](before-native-p028-overlay.png) |
| 수정 전 WASM28쪽 | [review](before-wasm-p028-review.png) | [overlay](before-wasm-p028-overlay.png) |
| 수정 후 Native28쪽 | [review](after-native-p028-review.png) | [overlay](after-native-p028-overlay.png) |
| 수정 후 WASM28쪽 | [review](after-wasm-p028-review.png) | [overlay](after-wasm-p028-overlay.png) |
| 수정 후 Native56쪽 | [review](after-native-p056-review.png) | [overlay](after-native-p056-overlay.png) |
| 수정 후 WASM56쪽 | [review](after-wasm-p056-review.png) | [overlay](after-wasm-p056-overlay.png) |
| 수정 후 Native57쪽 | [review](after-native-p057-review.png) | [overlay](after-native-p057-overlay.png) |
| 수정 후 WASM57쪽 | [review](after-wasm-p057-review.png) | [overlay](after-wasm-p057-overlay.png) |

짧은 수평선과 셀·내용 소유가 복원되고56쪽의 첫 상자와57쪽의 이어받기·표 종료·뒤 표/본문 소속도 보존된다.
얇은 외곽선과 일부 굵은 글자 기준선의 작은 차이는 남는다.

## 보류 항목과 출처

고정4a07 대비 Native 두 환경은0.1%p 하락0쪽이다. WASM 두 환경에는18쪽
99.97662→99.79255%(0.18407%p),43쪽99.89407→99.78159%(0.11248%p)가 남는다.
두 곳은 TTF Palatino의0.5em 꺾쇠를 HFT 부분 표0.605em으로 재는 별도 소유 문제로 조사 중이다.
현재90% 통과/세 쪽 개선으로 이 후퇴를 면제하지 않는다. 전체 nextest·다른 문서 전쪽/범위 게이트·
최종 실제 소비 검증도 남아 있어 PR 준비 완료를 주장하지 않는다.

실행 source/patch·입력/PDF/글꼴 hash와 전체 명령/exit는 ignored
`rebase-4a07d3ae-20261019/repair197~213`의 producer·audit·로그에 보존했다.
로그·중간 JSON·HFT 표본 PDF·글꼴 바이너리·삭제한 SVG는 이 제출 asset에 포함하지 않는다.
