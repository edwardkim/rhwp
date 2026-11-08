# 개인정보 저장 프레임: fresh WASM과 재발 방지 검사

Issue: #7445. Runtime source는 Stage53 `a340511936499170626bef6408a16b419b4466ac`이며, 이 후속 단계는 기존 검사 함수의 assertion과 증적만 추가한다. 새 HWP/HWPX 보정 대상과 새 fixture는 없다.

## 전체 출력과 직접 판독

- 원본/PDF는 [Stage53 출처](../stage53-pii-stored-child-frame-20261022/verification.md)와 같은 SHA다.
- locked wrapper `--target web --out-dir pkg --no-opt`, Mac 로컬 대체 빌드가 297.937초에 성공했다. Docker 최적화 빌드 결과가 아니다.
- Native 전 66쪽 최저 96.57165%, fresh WASM Canvas2D 전 66쪽 최저 96.56850%, 각각 미달 0쪽. 독립 Print66쪽과 일치한다.
- **65·66쪽은 Native/fresh WASM 모두 100.00000% raw 실루엣**이며 양쪽 review/overlay를 같은 배율로 직접 판독했다. 첫 줄과 뒤 네 줄의 소속, 원본 작은따옴표, 첫 조각 상하 여백과 종료 테두리를 확인했다. 2px 내용 실루엣이며 완전한 byte 일치가 아니다.
- 루트 pkg·Studio public·보존 패키지의 JS SHA-256 `aea5f6639676442e0628b640b75d0f70eb300481421112d23b661a082f695a68`, WASM SHA-256 `8da73e66e1f6609af639bcd5d752d6398e568a5dcf02c2d6155ab0ba6bd44cb6`가 각각 일치한다. 실제 초기화 응답 hash와 Studio 소비 경로 검증은 아직 별도 미완료다. CanvasKit 통과로 보고하지 않는다.

[Native 전 66쪽 TSV](../stage53-pii-stored-child-frame-20261022/after-native-silhouette.tsv) · [fresh WASM 전 66쪽 TSV](after-wasm-silhouette.tsv)

| 쪽 | Native | fresh WASM |
| --- | --- | --- |
| 65 | [review](../stage53-pii-stored-child-frame-20261022/after-native-p065-review.png) / [overlay](../stage53-pii-stored-child-frame-20261022/after-native-p065-overlay.png) | [review](after-wasm-p065-review.png) / [overlay](after-wasm-p065-overlay.png) |
| 66 | [review](../stage53-pii-stored-child-frame-20261022/after-native-p066-review.png) / [overlay](../stage53-pii-stored-child-frame-20261022/after-native-p066-overlay.png) | [review](after-wasm-p066-review.png) / [overlay](after-wasm-p066-overlay.png) |

## 추가 회귀의 의미

기존 `issue_5846_cut_fragment_defers_partial_nested_tail_row` 안에 두 검사를 보완한다.

1. 첫 부모 셀 조각이 원본 자식 첫 프레임과 부모 상하 안 여백을 한 번 소유하는지 관계로 확인한다. 예상값은 원본 1794+223+223 HU에서 계산한 29.8667px이며 수정 helper의 반환값을 재인용하지 않는다. 기존 쪽 소속·내용 순서·누락·중복 관계는 유지한다.
2. 독립 Print의 실제 벡터 선(1011.37337→1041.26131 CSSpx)이 정한 행 높이 29.88794px를 ±2px로 보호한다. 양쪽 raw 100%·직접 판독·전체 문서 90% 선행 조건을 충족한 65쪽의 중요한 첫/이어받기 경계다.

수정 전 정확한 P47 runtime(기존 실제 출력26.8933px)을 지정 작업장의 같은 검사로 실행해 새 관계 assertion에서 **1 FAIL, 0.430초**를 확인했다. P48 runtime으로 복원·hash 확인 후 같은 검사에서 **1 PASS, 0.595초**를 확인했다. 초기 frozen rlib SDK 연결의 의존 metadata 불일치 컴파일 오류는 무효이며 결함 FAIL로 세지 않았다. 실제 Cargo의 기존 manifest→suite014 경로에서 검증했다. 최종 source 주석·형식 정리 뒤에도 같은 검사 1 PASS / 0 FAIL, 테스트 0.576초를 확인했다.

원래 P48 관련 123/123 PASS와 이 새 assertion 한 건의 재실행을 구분하며, 전체 nextest 통과로 확대하지 않는다. Runtime source는 Stage53과 동일하므로 새 WASM을 중복 빌드하지 않는다.

## 같은 기준 대비 보류

SHA 고정 `f7abae4a8e70c062c378db06ad64d2d2eacb88cd`의 Native CLI를 source archive에서 순차 빌드하고 같은 713개 실제 글꼴 파일·선언 환경·입력/PDF·DPI·compat·PNG/PDFium·2px 조건으로 전 66쪽 TSV를 다시 산출했다. 비교 기준은 과거 PR 수치가 아니라 [이번 fixed-base 전쪽 TSV](fixed-base-native-silhouette.tsv)다.

**41쪽은 기준 99.23712→후보 97.93657%,1.30055%p 하락**으로 보류한다. Native 다른 65쪽에는 0.1%p 이상 하락이 없다. Canvas2D의 fixed-base 비교, 전체 nextest·lint/build/Skia·manifest/unit tiers, 원래 제출 범위의 나머지 전체 시각·소비 경로는 아직 미완료다. 65·66쪽 개선이나 문서 전체90% 이상으로 이 보류를 상쇄하지 않는다. 같은 개인정보 문서 안에서 원인을 확인하며 보정 대상을 새 문서로 확대하지 않는다.

현재 41쪽 위치 추적은 논리 render-tree 상자와 실제 Print border가 다를 수 있음을 확인했다. 후보 하단 선은 raster y 1038, 기준은 1039~1040, 독립 PDF는 1041~1042이며 이 차이가 미일치 픽셀 대부분을 차지한다. 글꼴 문제로 단정하거나 공통 종이 CTM을 문서별 예외로 우회하지 않는다. 독립 기준에 따라 물리 컷/끝 상자와 공통 paint 소비를 추적해야 한다.

로그·source/producer·새 기준 archive hash·전체 PNG와 초기 컴파일 무효 기록은 ignored `output/pr-review/7445-completed-20261009/rebase-8155-20261022/`에 보존한다. PR 생성·push·외부 게시·merge는 하지 않았다.
