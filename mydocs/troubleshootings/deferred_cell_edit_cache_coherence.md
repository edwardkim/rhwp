---
kind: reference
status: active
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-17
---

# 지연 셀 편집의 캐시·페이지네이션 정합성 진단

## 목적과 적용 범위

표 셀 편집은 모델, 문단 조판, 셀 레이아웃 캐시, 페이지 트리, cursor와 Canvas를 서로 다른
시점에 갱신할 수 있다. 이 문서는 지연 페이지네이션 또는 page-local redraw를 사용하는 편집에서
“입력은 모델에 들어갔지만 화면·캐럿·표 경계 중 일부만 이전 상태로 남는” 문제를 층별로 분리하는
재발 진단 절차다.

다음 증상에 우선 적용한다.

- 셀에 입력한 글자가 즉시 표시되지 않거나 다음 입력·전체 재조판 뒤에 나타난다.
- 모델의 cursor offset은 증가했지만 cursor 조회가 느려지거나 엉뚱한 위치를 반환한다.
- 줄은 늘었는데 행 높이, 다음 문단 위치 또는 페이지 분할 cut이 이전 값이다.
- 문서를 새로 열면 정상인데, 같은 세션에서 미리 렌더한 뒤 편집할 때만 실패한다.
- 일반 입력은 빠르지만 특정 줄·셀·페이지 경계에서만 큰 지연이 발생한다.

이 절차는 한컴의 line-break semantic, 폰트 metric 또는 파서 문제를 처음부터 전제하지 않는다.
먼저 각 층의 최신 여부를 확인하고, 증거가 그 층을 가리킬 때만 조사 범위를 넓힌다.

## 1. 상태 층과 고장 서명

한 번의 셀 입력은 최소한 다음 상태 층을 통과한다.

| 층 | 확인할 상태 | 오래되었을 때의 대표 서명 |
|----|-------------|---------------------------|
| 문서 모델 | text, char offset, cell path | 저장·재로드에도 글자가 없거나 다음 cursor offset 자체가 증가하지 않음 |
| 문단 조판 | `LINE_SEG`, line start, `vpos`, 높이 | 모델 text는 최신이지만 줄 구조가 입력 전과 같거나 잘못된 줄 나눔을 가짐 |
| 셀 레이아웃 캐시 | `cell_units`와 owner-table 상태 | cold는 정상, warm만 실패하며 동일 셀 page tree가 이전 text 끝에서 멈춤 |
| 페이지 트리 | fragment cut, bounds, 다음 항목 위치, page count | text/tree는 최신이지만 행 높이·cut·다음 문단 위치가 full flush 전 값임 |
| cursor 조회 | exact offset, cell path, rect, fallback 여부 | 최신 offset을 tree에서 못 찾아 전체 페이지 scan 또는 near fallback으로 느려짐 |
| 표시 | Canvas 합성, DOM caret, IME 조합창 | 구조와 cursor는 최신인데 화면 픽셀 또는 overlay만 이전 상태임 |

한 층이 최신이라는 사실로 아래 층까지 최신이라고 추정하면 안 된다. 예를 들어 모델과
`LINE_SEG`가 최신이어도 warm `cell_units`가 남으면 page tree는 이전 text 끝을 가질 수 있다.
반대로 cell cache만 바로잡아 tree/cursor가 최신이 되어도 flow 경계를 넘은 표의 cut과 bounds는
full pagination 전 값일 수 있다.

### 빠른 분기표

| 관찰 | 우선 조사 |
|------|-----------|
| 모델 text부터 틀림 | edit command, offset 변환, IME/raw input, undo/redo |
| 모델은 맞고 `LINE_SEG`가 틀림 | reflow, break unit, font/문단 metric |
| cold는 맞고 warm만 틀림 | layout-cache invalidation과 cache key/owner 범위 |
| cache-only 무효화로 tree/cursor만 회복 | cache coherence 확정, pagination geometry는 별도 판정 |
| explicit full flush에서 cut/bounds까지 회복 | flow 경계 신호와 flush 순서 |
| 구조·cursor는 맞고 Canvas만 틀림 | display invalidation, static layer reuse, overlay 갱신 |
| full flush 뒤에도 구조가 틀림 | paginator 또는 layout semantic 자체 |

## 2. 재현을 고정하는 방법

문서 전체의 육안 비교만으로는 어느 층이 어긋났는지 판정하기 어렵다. 먼저 한 셀과 한 입력
경계를 고정하고 다음 값을 같은 체크포인트에서 수집한다.

1. 문서 형식과 샘플 SHA 또는 고정된 fixture 이름
2. section, parent paragraph, control/cell path, cell paragraph와 편집 전 offset
3. 입력 후 model text suffix와 최종 offset
4. 대상 문단의 line start, `vpos`, line height/spacing
5. 대상 page tree의 TextRun 최대 offset과 stable source key
6. 표 fragment의 cut, bounds, 다음 항목 위치와 page count
7. direct/path cursor의 cell path, rect와 조회 시간
8. pagination pending, flush 횟수와 mutation/flush/cursor 호출 순서
9. 동일 crop의 입력 전후 및 지연 체크포인트 픽셀 비교

시간은 원인 후보를 찾는 관찰값으로만 남긴다. 머신, 브라우저, 폰트와 캐시 상태에 따라 달라지므로
정확성 hard gate는 text/offset, cut/bounds, 호출 횟수·순서와 화면 상태로 만든다.

## 3. cold/warm·direct/path 통제 실험

각 case는 새 문서 인스턴스에서 시작한다. 앞 case의 page tree 또는 cursor 조회가 다음 case의
cache 상태를 바꾸지 않게 해야 한다.

### 3.1 cache 상태

- **cold**: 문서를 연 뒤 대상 page tree나 cursor를 미리 조회하지 않고 편집한다.
- **pre-warm**: 편집 전에 대상 page tree를 만들고 대상 cell cursor를 한 번 조회한다.
- **mid-warm**: 연속 입력 도중 정해진 offset에서 tree/cursor를 조회한 뒤 입력을 계속한다.
- **every-edit warm**: 매 입력 뒤 조회해 실제 Studio의 반복적인 렌더·caret 조회에 가까운 상태를
  만든다.

cold 성공과 pre-warm 실패의 차이는 입력 내용보다 “편집 전에 무엇이 캐시되었는가”를 가리킨다.
warm case만 느리거나 stale이면 parser와 최초 layout보다 cache invalidation을 먼저 조사한다.

### 3.2 편집 방식

- **batch**: 같은 문자열을 한 번의 mutation으로 삽입한다.
- **sequential**: 실제 키 입력처럼 한 글자씩 삽입한다.

batch만 검사하면 중간 경계 신호, pending 누적과 history merge 결함을 놓칠 수 있다. sequential
case에서는 입력마다 mutation result를 기록하고, 경계 앞·경계·경계 뒤를 분리한다.

### 3.3 cursor 방식

- **direct**: section/paragraph/cell 좌표를 직접 지정하는 exact cursor 조회
- **path**: nested cell path를 따라가는 path/near cursor 조회

두 API가 같은 모델 offset과 cell path를 대상으로 같은 geometry를 반환해야 한다. direct만
성공하면 path 해석 또는 fallback을, path만 성공하면 direct 좌표/셀 문맥 전달을 의심한다.
둘 다 warm에서만 느리면 공통 입력인 page tree/cache를 우선 조사한다.

### 3.4 최소 매트릭스

| 축 | 최소 case | 판정 목적 |
|----|-----------|-----------|
| 형식 | HWP, HWPX | 공통 IR 이후 결함인지 포맷별 파서 결함인지 분리 |
| cache | cold, pre-warm, mid-warm | 캐시 생성 시점 의존성 검출 |
| mutation | batch, sequential | 중간 경계·pending 누적 검출 |
| cursor | tree-only, direct, path | tree 생성과 cursor fallback 비용 분리 |
| flow | stable, 첫 flow 경계, 경계 다음 입력 | 0회/1회 flush 계약 판정 |

진단용 확장 매트릭스는 `tests/issue_2214_cache_matrix_probe.rs`, 빠른 영구 계약은
`tests/issue_2214_page_local_repaint.rs`를 참고한다. ignored 진단 probe의 실행 완료를 correctness
GREEN으로 과장하지 말고, 기대값을 assertion으로 고정한 non-ignored test를 최종 게이트로 둔다.

## 4. cache coherence와 pagination을 분리하는 oracle

같은 fresh document와 같은 입력을 사용해 다음 세 상태를 비교한다.

### A. 지연 편집 직후

모델/`LINE_SEG`, warm tree, cursor, cut/bounds를 수집한다. 여기서 tree의 최대 text offset이 모델
offset보다 작다면 page-local invalidation 아래에 살아남은 layout cache가 있는지 확인한다.

### B. cache-only 무효화 뒤

pagination을 수행하지 않고 편집 셀의 layout cache만 비운 뒤 tree/cursor를 다시 조회한다.

- tree/cursor가 최신 offset으로 회복하면 cache coherence 누락의 직접 증거다.
- cut/bounds가 그대로라면 실패가 아니다. 이는 셀 내부 내용과 페이지 geometry가 서로 다른
  갱신 경계를 가진다는 증거다.

### C. explicit full pagination 뒤

full flush 뒤 cut, bounds, 다음 문단 위치와 page count를 다시 수집한다.

- B에서 text/tree가 회복되고 C에서 geometry가 회복되면 해결책도 두 계약으로 분리한다.
  1. 모든 지연 mutation의 scoped layout-cache coherence
  2. 실제 flow 경계의 pagination 완료
- C에서도 틀리면 cache 무효화 순서가 아니라 paginator/layout semantic을 조사한다.

이 oracle에서 explicit flush는 원인 분리 도구다. 모든 키 입력에 full flush를 넣어 증상을 숨기는
것은 해결책이 아니다.

## 5. scoped invalidation 불변식

지연 편집은 문서 전체 캐시를 지우지 않고, 편집으로 의미가 바뀐 최소 범위를 무효화해야 한다.

1. 편집한 cell의 `cell_units`는 다음 tree/cursor 조회 전에 다시 계산된다.
2. owner table의 집계 플래그가 불변이면 unrelated cell과 table cache identity는 보존된다.
3. nested content contribution 때문에 cached owner 상태가 `Some(false)`에서 `Some(true)`로
   전환될 때만 owner table의 직접 cell cache를 함께 무효화한다. cache가 cold(`None`)이면 local
   witness로 `true`를 기록하고 edited cell만 무효화한다.
4. unrelated table 및 무관한 nested table cache는 보존한다.
5. global cache clear는 문서 교체·전체 재조판처럼 실제 전역 경계에서만 사용한다.
6. 캐시 key가 object identity에 의존하면 다른 IR의 포인터 재사용이 섞이지 않도록 문서 수명
   경계에서 반드시 정리한다.

캐시 무효화가 정확한지는 “화면이 고쳐졌다”뿐 아니라 stable 입력에서 unrelated cache hit가
유지되는지로도 검증한다. 이것이 #1949의 메모이제이션 이득을 보존하는 회귀 계약이다.

## 6. stable 입력과 flow 경계

입력마다 전체 pagination이 필요한 것은 아니다.

- **stable deferred edit**: 대상 문단의 상대 flow advance가 바뀌지 않는다. scoped cache
  invalidation과 page-local redraw를 수행하고 동기 pre-cursor full flush는 0회다. deferred
  pending은 남으며 기존 idle/manual/full-edit 정책으로 나중에 마감할 수 있다.
- **flow boundary edit**: 줄/셀의 downstream geometry에 영향을 주는 상대 flow advance가
  바뀐다. cursor 조회 전에 pagination을 정확히 한 번 시도한다.

줄 수만으로 경계를 판정하지 않는다. 줄 수가 같아도 높이·spacing 변화가 downstream flow를
바꿀 수 있고, 반대로 표현상의 line 객체 수 변화가 실제 trailing advance를 바꾸지 않을 수 있다.
현재 계약은 편집 전후 target paragraph의 상대 trailing advance를 비교해 `cellFlowChanged`를
전달한다.

## 7. 호출 순서 계약

경계 입력의 필수 순서는 다음과 같다.

```text
mutation
→ deferred pending 등록
→ flow boundary이면 pending pagination flush 1회 시도
→ exact cursor 조회
→ page/display refresh
```

세부 불변식은 다음과 같다.

- pending은 cursor 조회 전에 등록한다.
- stable effect는 동기 flush 없이 cursor/refresh로 진행하고 pending은 기존 마감 정책에 맡긴다.
- boundary effect는 cursor 전에 flush를 한 번 시도한다. 성공하면 최신 pagination에서 cursor를
  조회한다. 실패하면 현재 구현은 pending을 보존한 채 cursor 조회를 계속하며, 30쪽 초과 문서의
  retry/error UX는 아직 후속 범위다.
- 한 mutation의 effect는 one-shot으로 소비하고, explicit control이 두 번째 flush를 만들지 않는다.
- undo/redo에서 오래된 effect를 재사용하지 않는다. redo mutation의 실제 결과로 다시 계산한다.
- IME/iOS raw 입력처럼 여러 event가 한 편집을 구성하면 effect를 누적하되 boundary 신호를 잃지
  않고, commit/cancel/document switch에서 상태를 초기화한다.
- immediate mutation이나 fallback 경로는 남아 있던 deferred pending을 정리하고 자기 완료 상태를
  명시한다.
- 문서 전환·입력 handler 비활성화 시 pending, timer, raw/IME/iOS accumulator를 모두 초기화한다.

`mutation → cursor → flush` 순서는 잘못된 순서다. stale tree에 대한 cursor exact miss와 전체
페이지 fallback 비용을 먼저 발생시키며, 반환된 caret도 이미 잘못된 geometry를 사용한다.

## 8. 영구 회귀 매트릭스

다음 계약을 서로 다른 테스트 층에 배치한다.

| 테스트 층 | 필수 확인 |
|-----------|-----------|
| Rust 구조 | model/tree 최대 offset, exact direct/path cursor, fragment cut 연속성, bounds, page count |
| cache scope | stable edit의 unrelated cache hit 보존, owner 상태 전환 시 필요한 범위만 재계산 |
| Studio unit | mutation effect 전달, one-shot 소비, redo 재계산, immediate/pending 대체, lifecycle reset |
| 입력 router | normal, IME, iOS raw의 stable 동기 0회·boundary 동기 1회 flush 시도 |
| 브라우저 E2E | HWP/HWPX, mutation→flush→cursor 순서, 최신 TextRun/caret, 지연 뒤 픽셀 안정성 |
| 저장 회귀 | 저장·재로드 뒤 text, line geometry, page count와 cell path 유지 |

브라우저 E2E는 경계 직전, 동기 입력 직후, 1~2 rAF, 짧은 지연과 기존 verification timer 이후를
비교한다. caret blink의 opacity는 판정값으로 쓰지 않고 표시 여부와 위치를 검사한다. 픽셀 hash는
같은 실행의 시간축 안정성에는 유용하지만 OS·폰트·DPR이 다른 머신 사이의 golden으로 사용하지
않는다.

브라우저 E2E를 실행하려면 먼저 최신 WASM package를 준비하고 별도 터미널에서 Vite를 실행한다.

```bash
cd rhwp-studio
npm run dev -- --host 127.0.0.1 --port 7714 --strictPort
```

그 다음 테스트 터미널에서 `VITE_URL`과 Chrome 실행 경로를 지정한다.

```bash
cargo test --profile release-test --test issue_2214_page_local_repaint
cargo test --profile release-test --test issue_2214_cache_matrix_probe -- --ignored --nocapture

cd rhwp-studio
VITE_URL=http://127.0.0.1:7714 \
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
npm run e2e:issue-2214 -- --runs=1
```

## 9. 금지 패턴

- **매 키 global clear**: correctness는 우연히 회복해도 #1949의 거대 셀 메모이제이션을
  무력화하고 지연 원인을 다른 비용으로 바꾼다.
- **cursor 조회 뒤 flush**: stale cursor/fallback을 이미 실행한 뒤라 정확성과 지연을 해결하지
  못한다.
- **line count만 경계 신호로 사용**: 높이·spacing·ordered geometry 변화를 놓친다.
- **고정 밀리초를 correctness gate로 사용**: 실행 환경 차이로 false failure/false success를
  만든다. 횟수·순서와 exact state를 gate로 사용한다.
- **Canvas만 강제 redraw**: stale page tree를 새로 그릴 뿐이며 model/tree 불일치를 숨긴다.
- **explicit full flush 성공을 cache fix로 간주**: cache-only와 pagination oracle을 분리하지 않으면
  일반 입력까지 불필요한 full pagination으로 보낼 수 있다.
- **cold case만 검증**: 편집 전에 생성된 캐시가 원인인 결함을 놓친다.
- **direct 또는 path 한쪽만 검증**: 중첩 셀 path와 fallback 차이를 놓친다.
- **진단 PNG/JSON을 전역 golden으로 커밋**: 브라우저·폰트·DPR 종속 산출물은 실행 내 비교 자료로
  두고, 구조적 기대값을 코드 assertion으로 승격한다.

## 10. 혼합 서식 문단의 한계

현재 `cellFlowChanged` 판정은 target paragraph의 편집 전후 상대 trailing advance를 비교한다.
따라서 총 advance가 같지만 중간 줄의 ordered geometry가 달라지는 혼합 글꼴 크기, baseline,
줄간격 또는 인라인 개체 조합은 아직 완전히 대표하지 못한다.

이 경우 aggregate 값이 같다는 이유로 stable로 분류하면 중간 line/cell-unit 위치 변화가 page
tree나 cursor에 영향을 줄 수 있다. 재현이 확인되면 기존 boolean을 무조건 넓히기 전에 다음을
별도 범위에서 비교한다.

1. line별 `(start, vpos, line_height, text_height, baseline, spacing)`의 ordered signature
2. cell-unit의 순서·높이와 downstream contribution
3. signature를 안정적으로 만들 수 없을 때의 보수적 dirty 신호
4. false positive flush 비용과 false negative geometry 오류의 trade-off

이 제한은 한컴 line-break semantic 전체를 복제해야 한다는 결론과 동일하지 않다. 먼저 rhwp 내부
편집 전후 geometry가 실제로 달라졌는지를 관찰 가능한 signature로 고정한다.

## 11. 관련 이슈의 경계

- [#1949](https://github.com/edwardkim/rhwp/issues/1949): 거대 셀의 `cell_units` 반복 계산을
  메모이제이션해 `O(pages × cell)` 렌더 비용을 제거했다. 지연 편집 fix는 이 cache 이득을
  보존하면서 필요한 scope만 무효화해야 한다.
- [#1951](https://github.com/edwardkim/rhwp/issues/1951): 셀 bbox를 넘는 caret/IME 위치를 제한하고
  overflow 시 pagination하는 안전망 및 one-depth path 정합을 다룬다. cursor clamp는 stale tree의
  근본 원인을 대신하지 않는다.
- [#2214](https://github.com/edwardkim/rhwp/issues/2214): warm cell cache coherence와 실제 flow
  경계의 pre-cursor 동기 flush 1회 시도 계약을 고정한 사례다. 이 문서의 cold/warm oracle과
  호출 순서의 직접 근거다.
- [#2193](https://github.com/edwardkim/rhwp/issues/2193): 경계에서 남은 full pagination, page tree와
  Canvas 갱신 비용을 분해하는 종합 성능 범위다. bounded/partial paginator는 정확성 계약을 먼저
  보존한 뒤 이 범위에서 설계한다.

네 이슈를 한 수정으로 합치지 않는다. #1949의 cache 성능, #1951의 overflow 안전망, #2214의
cache/flow 정확성과 #2193의 paginator 성능은 서로 연결되지만 각각 독립적으로 회귀해야 한다.
