# #6040 — 첫 진입·스크롤 A/B 준비

- 날짜: 2026-09-09 KST
- 단계: B의 잔여 수용 조사 준비. 새 구조 실험이나 Stage 4 전체 완료가 아니다.
- 사용자 승인: 비교 서버·동일 계측 준비 후 Firefox에서 직접 조작할 링크 제공.
- 결과: 서버와 DEV 계측 준비 완료, 실제 Firefox A/B 수집·성능 판정 대기.

## 1. 비교 제품과 환경

| 항목 | 기준선 | 후보 |
| --- | --- | --- |
| 제품 SHA | `56706247f4950286117496c41f5b2c4b1cdbddc5` | `0345107c586c2088802d6629ba13ac6a66e3694c` |
| 로컬 worktree | `tmp/issue6040-ab-before` | `tmp/issue6040-ab-after` |
| loopback 포트 | 4201 | 4202 |
| 제품 변경 | 없음 | 없음 |

작업용 4200과 별도의 detached worktree를 사용한다. 계측 파일 3개만 같은 내용으로 적용하고,
Vite cacheDir를 각 worktree의 tmp로 분리했다. 설치된 node_modules는 같은 것을 연결했다.
양쪽 Rust source/crates/Cargo.lock/폰트/package-lock의 Git 차이가 없음을 확인했다.

동일 파일 검증 SHA-256:

| 항목 | hash |
| --- | --- |
| `pkg/rhwp_bg.wasm` | `24d3d2ffe0c1b43d4f53c23762820112b8298c2a081c9dd4842406819f4130fc` |
| `pkg/rhwp.js` | `5f85f63c23f6bf19688d31ac432080eb75f4c77c48c344a05d46820cddb57e7e` |
| `samples/exam_kor.hwp` | `0315576fb25dd29ad3b6b188ee2539d0e8d31c15b74847be801c2186a97aac69` |
| Studio package-lock | `2857a04491b9bc789d4e229ec0101cf21b99c421fe2d8eff462189bf710217d1` |
| `assets/fonts` 38파일 tree | `617196ca56ea222ba8f497a22ce3fe9354855c44d47e8611c806be0a5591c2e5` |
| `page-scroll-probe.ts` | `1325d1523d6641e57e05d4b3b56f38fc3a17bd90cbad0bb9e4c8aaec3c6422d9` |
| `zoom-session-observation.ts` | `7c4951fb2b1e957edc1dda3697f7235e799d29cda8eb050043cd480dd9b936fc` |
| `probe-zoom-compat.ts` | `d3378dab728da4b8d0166669f31c41e16d6251d07a6f1e5114833a3c9926d4a6` |

폰트 tree는 이름순 재귀 `[relativePath, fileSHA256]` 배열의 JSON을 SHA-256한 값이다.
동일 로컬 폰트 파일이 실제 런타임의 외부 웹폰트·fallback 선택까지 같다는 보증은 아니다.
문서 줄바꿈·글꼴·page dimensions가 다르면 시간 비교 전에 환경을 다시 확인한다.
WASM은 기존 진단용 `--no-opt` 결과다. 이 비교로 최적화된 확장 배포판의 성능 개선율을 주장하지 않는다.

## 2. 기록 경계

일반 URL autoload 뒤에 사용자가 시작 버튼을 누르면 첫 진입의 짧은 비용을 놓칠 수 있다.
따라서 패널의 새 문서 버튼으로 cache/surface를 새 문서 범위로 교체하고 초기 동기 화면 구성 뒤
`document-view-loaded`에서 20초 bounded 기록을 시작한다. 초기 파일 fetch·파싱·초기 동기 렌더
전체가 기록됐다고 주장하지 않는다. 이 이전 비용은 native profile 등 별도 수집이 필요하다.

이후 첫 축소·스크롤을 사용자가 즉시 수행한다. `warm-manual`은 현재 문서를 그대로 두고 수동으로
시작한다. `JSON 저장`은 이미 표시한 결과를 다운로드하며 다음 기록 시작 시 이전 export를 비운다.
제품 줌·화질·cache·prefetch 정책은 바꾸지 않는다. 양쪽에 없는 observer 메서드를 주입하지 않는다.

추가 rAF 표본의 미생성 bitmap/current raster/queue는 DOM bounds나 readback 없이 읽는다.
화면상 빈 픽셀 면적·최종 paint·DPR 화질 판정을 직접 대체하지 않는다. 기존 rAF+wheel trace와
추가 표본의 관찰 오버헤드는 아직 계측하지 않았다.

## 3. 실행 검증

- focused observation/compat tests: **20 pass**.
- 최종 Studio `npx tsc --noEmit`, `npm test`: **1,558 pass, 1 skip, 0 fail**.
- 구 기준선에 동일 observer를 적용한 TypeScript 검사도 통과.
- 인앱 브라우저 1280×720, DPR 2에서 양쪽 exam_kor 20쪽 로드와 초기 기록 배율 100% 확인.
- 양쪽 cold 시작 → 34% 버튼 → 다음 행 → 종료: `stopped`, observer errors 0, span/frame 기록 확인.
  34%에서 이 viewport는 3열이었다. 사용자의 4페이지 배치와 동일 조건인 성능 표본이 아니다.
- 후보 warm 시작/종료의 별도 metadata와 JSON 기록 확인.
- 다운로드 이벤트 대기는 인앱 도구에서 timeout됐으나 실제 Downloads의 JSON 파일 생성·파싱 및
  before/after 식별 필드·cold·stopped·errors 0을 확인했다. 파일은 smoke이며 성능 증적으로 채택하지 않는다.
- 마지막 export 초기화 보정 뒤 양쪽 reload에서 패널 준비 정상, 타입 검사·전체 테스트 재통과.

브라우저 테스트는 기능 smoke이며 실제 핀치·스크롤 성능이나 Firefox 지원의 실측을 대신하지 않는다.
agent가 만든 두 smoke 탭은 닫고 기존 사용자 탭은 보존했다. remote push·PR·이슈 생성 없음.

## 4. 사용자 수집과 판정

1. Firefox에서 같은 창 크기·브라우저 자체 배율을 사용하고 다른 문서 편집 탭의 부하를 줄인다.
2. 기준선/후보 각각 `exam_kor`, 동일 배치를 선택한다. 우선 자동을 가정하되 사용자 답변에 따라
   네 열 여부를 확정한다. 제품 문서 배율도 동일하게 기록한다.
3. cold 버튼 → 첫 화면 직후 축소·스크롤 → 20초 안에 종료·저장.
4. 이미 지나간 구간으로 복귀·정착 → 현재 구간 기록 → 같은 구간 왕복 → 종료·저장.
5. 첫 표본은 총 4개 JSON. 다음 반복에서는 A/B 순서를 반대로 하고 입력·관찰 비용 차이를 평가한다.

예비 판정은 frame/input p50/p95/max, 공통 raster 개수/총 inclusive/max 시간, 캐시·queue 및
미생성 visible 상태 지속을 함께 본다. 타이밍이 좋아도 빈 화면 지속/화질 회복이 나빠지면 개선으로
채택하지 않는다. timeout·중단·이미지 실패도 보존한다. 비교 반복 전 수치 경보선은 따로 확정한다.
한 쌍의 수동 표본만으로 통계적 개선율이나 회귀 부재를 선언하지 않는다.
