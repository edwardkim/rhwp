# 추가 공통 보호·SVG 소비 검증

PR #9140의 code head `c6996300203fec553a57b0c16b2e48329ebc083a`에서 필수 항목을 재대조한 뒤 실행했습니다. 공통 네 문서 검증이 최초 제출 기록에서 누락됐음을 확인해 추가 실행했으며, 제출 전 완료한 것으로 소급하지 않습니다. 제품 코드·판정 문턱·정본 baseline 변경은 없습니다.

## 실제 결과

- 공개 공통 보호 네 문서 × Native/fresh WASM: 8건 전부 공식 두 자리 raw 100.00%, 하락·누락 0. 영문 세 줄·좁은 문단 개행·그림 crop 두 개·그룹 도형을 각각 같은 쪽의 review/overlay로 직접 확인했습니다.
- SVG 소비 대표 검사: HFT 기호 탐침 3쪽 Native 99.68% / fresh WASM 98.44%, nested 5쪽 Native 99.99% / fresh WASM 100.00%. 기호와 뒤따르는 글자의 전진·표 내용·주석 ※·문단 소유를 직접 확인했습니다. HFT WASM SVG에는 미세한 글리프·간격 잔차가 있으며 100% 일치로 주장하지 않습니다.
- SVG는 대표 두 쪽의 소비 경로 확인입니다. 전체 문서 수용을 대신하지 않으며, 별도 canonical PNG 전쪽 TSV와 정본 보호 18문서·경로당1,771쪽 검증을 함께 사용합니다.
- 생성했던 기호 하나용 `RHWPIndependentHFTSymbols.otf`는 삭제했고 실제 검증의 자원 참조에도 없습니다. 원본 `H2GTRM.TTF`는 보존했습니다. 비공개 글꼴 바이너리는 포함하지 않습니다.

[입력·생산자·원문 TSV·이미지 해시·판독](validation.json)에 범위와 실행 시점을 기록했습니다. 이미지의 100.00%는 색상 경계 보정 전2px 실루엣 표시값이며 전체 색상 픽셀 동일성을 뜻하지 않습니다. PDF는 기존 독립 데스크톱 한컴 Print를 재사용했고 pdftoppm으로 변환했습니다.

Native/fresh WASM 생산 source는 `0d587554b804183b5cc6d090f396acc47525c089`입니다. 최신 head까지 일반 렌더 소비 경로·Cargo의 동일성과 별도 유입 GPU 검사 근거는 상위 rebase-validation에 보존했습니다. Mac WASM은 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며 Docker 최적화 빌드가 아닙니다.
