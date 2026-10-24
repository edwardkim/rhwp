# #7445 리베이스 보정 63단계: HFT 내부 수식 원점의 중복 기준선 정렬

## 기존 범위와 독립 근거

기존 C429 보정 `878627ddea738957026888af0778a05b9a973acf`와 해당 보고서·6개 편집 상태를
대조했다. HWP 보정 대상을 확대하지 않았다. 예전 경로의 PDF가 없어 첫 준비가 실행 전에
실패했으며, 사용자의 임시 산출물 이동 지시에 따라 이미 `temporary-artifacts-20261020/pdf/`에
보존된 원본을 찾아 사용했다. 새 기준 PDF를 만들거나 입력을 바꾸지 않았다.
정확한 6개 HWP 및 PDF의 SHA는 C429 정식 증거와 모두 같다.

독립 Print의 inline67/80은 drawing stream과 같은 raster가 동일하다. 기존/현재 cell90/95도
같은 drawing stream 및 raster다. 기준선 값은 호스트의 줄 프레임이 소비하며 내부 프로그램에
다시 더하는 변환이 아니라는 기존 관계 검사의 독립 근거다. 원본의 글리프 내용·소유·순서와
셀 내부 포함, 편집한 저장 LineSeg의 기준선 소비 검사도 유지했다.

## 공통 원인과 보정

`EqLayout::layout`은 versionless `HYhwpEQ`의 지원하는 HFT 논리 글줄·분수 프레임과 내부
원점을 이미 생산한다. `layout_for_paint`가 이 결과에 저장 기준선 비율을 다시 적용해, 똑같은
내부 프로그램이 기준선 편집만으로 이동했다. 그 원점이 SVG와 실제 Native/WASM paint로 전달됐다.

같은 공통 `legacy_logical_frame`이 증명하는 HFT 결과는 그 내부 원점을 보존한다. 저장 흐름
프레임·원본 크기·편집 LineSeg의 기준선 갱신은 유지한다. 현대 수식과 지원되지 않는 논리
프레임은 기존 경로를 보존한다. 문서 ID·쪽 번호·독립 수치를 구현 분기로 넣지 않았다.

## 실제 집중 검사와 시각 검증

- 직전 source의 기존 두 기준선 편집 검사 FAIL을 유지하고 구현을 보정했다.
- 현재 수식 계약 모듈 9건은 모두 PASS다. 기존 분수 괘선 독립 ±2 CSS px 검사와 현대 수식의
  intrinsic 경로 검사도 통과했다. 집중 36건은 24 PASS / 12 FAIL, 6.245초다.
- 변경 전 정확한 6개 상태는 양쪽 backend에서 기준선 67만 100%이며 나머지 5개 상태가 미달이었다.
  Native 최저 52.90698%다. 새 Native의 6개 상태는 각 전 1쪽 100%다.
- Native inline80/current cell95 review와 overlay를 직접 판독해 분자·분모·괘선과 내용 소유를
  확인했다. 100%를 RGB byte 동일성으로 해석하지 않는다.
- fresh WASM의 정확한 6개 상태도 각 전 1쪽 100%다. 같은 두 RGB raster 그룹이 두 backend의
  모든 상태를 포함하며 각 backend의 inline80/cell95 review·overlay를 직접 판독했다.
  Mac locked web/pkg/no-opt 빌드 230.189초, exit 0. root pkg/Studio public/frozen JS·WASM SHA가 같다.
  실제 Studio 초기화 응답은 아직 미검증이다.

기존 CJK 진단 대조군의 Native raw 수치는 전후 같다. 원래 미달이던 frame p3(84.60691%)와
parenthesis p3(76.18441%)의 PNG는 바이트가 동일하다. 나머지 일부 쪽은 50/28/67개 픽셀에서
RGB 채널 최댓값 2~3의 색값 변화가 있고 공개 bbox 트리는 같다. 미달 진단 쪽을 새 완료 페이지로
인용하거나 새 보정 대상으로 확대하지 않는다. 정상 baseline67 대조군은 PNG 바이트 동일·100%다.

기존 sample16 3종 64쪽의 전쪽 RenderTree에서 수식 노드는 각각 6쪽의 2개다. 현재 source의
변경 경계인 6쪽을 before/after Native·WASM으로 추가 대조한다. 전체 64쪽 최신 head 검증을
대신한다고 보고하지 않는다. Native는 99.94633/99.94867/99.94867%이며 최대 하락 0.00476%p로
0.1%p 이상 후퇴가 없다. 원본 HWP3의 6쪽 review·overlay에서 괘선·두 수식·동그라미 기호와
앞뒤 본문·쪽 번호 소속을 직접 판독했다.

## 잔여

남은 12건, 최신 전체 nextest, 필수 lint/build/Skia/manifest/unit tiers, 승인된 문서의 최신
Native/fresh WASM 전쪽 및 실제 Studio 초기화·소비 경로 검증을 계속한다. 이번 국소 통과를
전체 PR 준비 완료로 보고하지 않는다. 이전 PR 품질 래칫의 자동 강제 연결도 복구하지 않는다.

WASM sample16 6쪽은 99.94980/99.95200/99.95200%다. Native/WASM의 원본·HWP5 review와
overlay를 직접 판독했고 같은 backend HWPX의 RGB는 HWP5와 동일해 hash로 연결했다.
WASM 진단 frame p3는 86.35004% 유지, parenthesis p3는 76.36706→76.37202%다.
사용자 승인에 따라 raw 하락 기준 2%p는 별도 로컬 커밋 ea3022a4f0에 반영했다.

정식 TSV·PNG·6개 입력: `mydocs/pr/assets/issue7445/completed-20261009/rebase-605ccfd9-20261022/legacy-inner-paint-origin`.
필수 기준 PDF 6개는 `pdf/issue7445-legacy-baseline-edit/`에 같은 SHA로 보존했다.
생산 출처·진단·전체 로그는 ignored `stage63-legacy-inner-paint-frame/`다.
