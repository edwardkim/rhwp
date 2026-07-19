---
kind: decision
status: active
canonical: mydocs/tech/thorvg_decision.md
last_verified: 2026-07-16
---

# ThorVG 기술 검토 결정 기록

## 문서 정보

| 항목 | 내용 |
|------|------|
| 작성일 | 2026-02-19 |
| 검토 기간 | 타스크 112~115 (2026-02-18 ~ 2026-02-19) |
| 결정 | ThorVG를 rhwp 렌더링 백엔드로 **채택하지 않음** |
| 코드 롤백 | 타스크 111 완료 커밋(`46d417a`)으로 main/devel 복원 |

## 요약

ThorVG(Samsung 오픈소스 벡터 그래픽 엔진)를 HWP 웹 에디터(rhwp-studio)의
Canvas 2D 대안 렌더링 백엔드로 검증하기 위해 4개 타스크에 걸친 POC를 수행했다.
결론적으로 **실시간 편집 용도에 부적합**하여 채택하지 않기로 결정했다.

## POC 수행 내역

| 타스크 | 내용 | 결과 |
|--------|------|------|
| 112 | Rust FFI 바인딩 + 네이티브 PNG 렌더링 | ThorVG C API → Rust FFI 30개 함수, HWP→PNG 출력 성공 |
| 113 | Emscripten WASM 빌드 + WebGL 렌더링 | Docker 빌드 파이프라인, JS 브릿지, WebGL 2.0 GPU 직접 렌더링 |
| 114 | rhwp-studio 통합 + 편집 기능 검증 | 렌더러 전환 UI, 캐럿/선택/IME 모두 GL 위에서 동작 확인 |
| 115 | TTF 폰트 메트릭 + 글자별 개별 배치 | ttf-parser 기반 글리프 측정, charPositions 배열, 장평 변환 |

## 불채택 사유

### 1. 편집 후 재렌더링 지연 (구조적 한계)

Canvas 2D의 `fillText()`는 브라우저 내부에서 동기적으로 GPU 가속 처리된다.
ThorVG GL은 비동기 다층 파이프라인을 거쳐야 한다:

```
Canvas 2D: 키 입력 → WASM → fillText() → 즉시 반영 (1 hop)

ThorVG GL: 키 입력 → WASM → JSON 직렬화 → JS JSON.parse()
  → setupCanvas() → preloadFonts() → renderNode()
  → GL 렌더링 → drawImage(GL→2D) → 화면 반영 (7+ hops)
```

이 지연은 최적화로 줄일 수 있지만 **WASM ↔ JS ↔ WebGL 경계**를 넘는 한
근본적으로 제거할 수 없다. 한글 입력 시 체감 가능한 지연이 발생했다.

### 2. 폰트 처리의 비효율성

| 비교 항목 | Canvas 2D | ThorVG |
|-----------|-----------|--------|
| 폰트 로딩 | CSS `@font-face` → 브라우저 자동 처리 | TTF fetch → WASM 힙 복사 → `tvg_font_load_data()` |
| 새 폰트 발견 시 | 투명하게 처리 | 렌더링 블로킹 (핫 패스에 위치) |
| 폰트 폴백 | 브라우저 내장 체인 | 직접 구현 필요 |
| 메모리 관리 | 브라우저 관리 | WASM 힙에 상주 |

ThorVG는 **고정된 1~2개 폰트**를 사용하는 임베디드 환경(Tizen TV, Lottie)에 최적화되어 있다.
워드프로세서처럼 **문서마다 다양한 폰트를 실시간 바인딩**하는 시나리오는 설계 범위 밖이다.

### 3. 공유 GL 캔버스 레이스 컨디션

ThorVG는 단일 GL 캔버스를 사용하는데, 가상 스크롤 환경에서 여러 페이지가
동시에 비동기 렌더링을 시작하면 `setupCanvas()`가 이전 페이지의 ThorVG 캔버스를 파괴한다.
직렬화 큐로 해결 가능하지만, 이는 다중 페이지 렌더링 시 추가 지연을 발생시킨다.

### 4. 네이티브 빌드 의존성

ThorVG C 라이브러리(`libthorvg-1`)가 네이티브 빌드에 필요하여
Docker 개발 환경에 추가 설치가 필요하다. 기존 순수 Rust 빌드 체인이 복잡해진다.

## POC에서 확인된 긍정적 사항

### 아키텍처 검증 성공

- **렌더러 독립적 편집 인프라**: 캐럿, 히트테스트, IME가 DOM 오버레이와 WASM API 기반이므로 렌더링 백엔드와 완전 독립적으로 동작함을 확인
- **렌더 트리(JSON) 추상화**: 중간 표현을 통한 렌더러 교체가 실제로 동작함을 확인
- **Google Docs와 동일한 패턴**: Canvas + DOM 오버레이 + 숨겨진 textarea 구조가 올바른 설계임을 재확인

이 아키텍처 검증은 향후 다른 렌더링 백엔드를 검토할 때 유용한 기반이 된다.

## 결정 사항

1. **ThorVG 코드 제거**: 타스크 112~115에서 추가된 모든 ThorVG 관련 코드를 롤백
2. **Canvas 2D 유지**: 실시간 편집의 기본 렌더링 백엔드로 Canvas 2D를 계속 사용
3. **렌더 트리 구조 유지**: 렌더 트리(JSON) 추상화는 SVG 내보내기 등에서 이미 활용 중이므로 유지
4. **POC 이력 보존**: 제거된 ThorVG 구현은 특정 로컬 브랜치의 존속을 전제로 하지 않는다. 재검토 시 이
   결정 기록과 Git commit 이력을 기준으로 당시 POC를 확인한다.

## 향후 ThorVG 재검토 조건

다음 조건이 충족될 때 ThorVG 재도입을 검토할 수 있다:

- 서버사이드 PNG/PDF 내보내기 기능이 필요할 때 (네이티브 환경, 시간 제약 없음)
- 읽기 전용 뷰어에서 GPU 가속 렌더링이 필요할 때
- ThorVG의 웹 폰트 지원이 개선될 때

## 참조 문서

- [ThorVG PoC 인사이트](investigations/issue-112/thorvg_poc_insights.md) — 타스크 112~115의 보존된 POC 요약과 기술 근거
- [mydocs/feedback/font-metrics.md](../feedback/font-metrics.md) — 폰트 메트릭 관련 피드백
