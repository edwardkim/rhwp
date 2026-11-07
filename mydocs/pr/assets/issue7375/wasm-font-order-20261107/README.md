# WASM Visual Sweep 글꼴 프로그램 순서 보정

2026-11-07. 사용자 지정 원본 공급 순서 `TTF → TTC의 일치 face → OTF → 대체`를
Native·Studio와 동일하게 Visual Sweep의 WASM 공급에도 적용한다. 원본과 대체 중 무엇을
선택할지는 기존 Rust 공통 API가 소유하고, 호스트 도구는 실제 형식과 준비된 프로그램을 전달한다.

기존 WASM 환경 준비·Canvas 등록·글리프 공급은 같은 family/style에서 CFF를 먼저 골랐다.
스타일 우선순위는 유지하면서 실제 프로그램 형식의 순서를 공통 함수로 맞췄다. CFF/HFT 요청에도
실제 선택한 TrueType 프로그램을 전달하며, 명시적인 source TTF 질의의 형식 보호는 유지한다.
TTC를 개별 SFNT로 추출해도 원래 컨테이너 순서를 metadata와 공급 snapshot에 보존한다.
FontFace 등록은 CSS 우선순위에 맞춰 역순으로 수행한다. 원본 글꼴·PDF·판정 문턱은 바꾸지 않는다.

형식·스타일 관계 검사는 수정 전 실제 실패를 확인했고 수정 후 Node 16건, 실제 브라우저 공급
2건, Python PNG 계약 38건을 통과했다. Python 검사는 Brotli가 준비된 저장소 venv에서 실행했다.

[대조 TSV](comparison.tsv)는 고정된 같은 코드의 Native/WASM producer와 같은 입력·PDF를
사용한 공급 도구 보정 전후 3쪽 비교다. 이 코드는 새 devel 리베이스 전 producer이며, 최신 HEAD의
전쪽 Native/fresh WASM 통과로 인용하지 않는다. 최신 head와 고정 base는 새 도구로 전쪽 재검증한다.

- 전기안전 8쪽: [review](electrical7728-p008-review.png) · [overlay](electrical7728-p008-overlay.png)
- 전기안전 11쪽: [review](electrical7728-p011-review.png) · [overlay](electrical7728-p011-overlay.png)
- LPG 5쪽: [review](lpg71352-p005-review.png) · [overlay](lpg71352-p005-overlay.png)

8쪽의 정부개입 단락과 대안 표, 11쪽 및 LPG 5쪽의 글줄 소속·문장부호·앞뒤 내용을 직접 확인한다.
producer·source hash·로그 위치 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7375/wasm-font-order-20261107/validation.json`)를 보존한다. 글꼴 파일 바이너리는 포함하지 않는다.
