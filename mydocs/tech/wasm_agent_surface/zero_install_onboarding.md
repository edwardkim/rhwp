---
kind: guide
status: active
canonical: mydocs/tech/wasm_agent_surface/zero_install_onboarding.md
last_verified: 2026-08-03
---

# 설치 0 온보딩 — 오프라인 데모 페이지 설계

> 사용자가 **아무것도 설치하지 않고** rhwp 를 써보는 경로를 확정한다.
> 단일 HTML 인가, CDN 인가, 정적 사이트인가. 크기 예산은 얼마인가. 오프라인에서 도는가.
> 로드맵 [#3608](https://github.com/edwardkim/rhwp/issues/3608) M24 셋째 줄,
> [#3869](https://github.com/edwardkim/rhwp/issues/3869) §1("실제 첫 관문 — 바이너리")에
> 대응한다.

이 문서의 모든 기술 주장에는 코드 경로(`파일:줄`) 또는 실측이 붙는다.
**크기는 지어내지 않는다** — 잰 것과 안 잰 것을 §2 에서 명확히 가른다.
축 전체의 지도는 [README.md](README.md).

---

## 0. 결론 먼저

1. **"설치 0"의 절반은 이미 존재한다.** rhwp 는 GitHub Pages 로 자동 배포된다
   (`.github/workflows/deploy-pages.yml`). 브라우저로 URL 을 열면 HWP 가 열린다.
   #3869 가 말하는 "바이너리 확보"라는 첫 관문은 **브라우저 경로에서는 이미 없다.**
2. **없는 절반은 에이전트 축이다.** 그 페이지는 **뷰어/에디터**이고, 보여주는 것이
   렌더링이다. `digest`·`fields`·`inspect` 같은 에이전트 동사를 보여주지 않는다 —
   애초에 WASM 에 노출돼 있지 않기 때문이다
   ([self_description.md §1.3](self_description.md)).
3. **크기 예산을 지금 세울 수 없다.** `.wasm` 크기가 **측정되지 않았다.**
   저장소가 `*.wasm` 을 제외하고(`.gitignore:12`) 이 작업 트리에 산출물이 없다.
   유일한 숫자는 소스 주석 `WASM (~12 MB)`(`rhwp-studio/vite.config.ts:132`)이며
   **이건 측정치가 아니다.**
4. **단일 HTML 은 지금 불가능하다고 봐야 한다.** 근거는 §3.
   현실적 채택은 **기존 Pages 배포에 데모 경로를 추가**하는 것이다.
5. **오프라인은 "첫 로드 이후"만 성립한다.** 서비스워커가 `.wasm` 을 precache 에서
   **명시적으로 제외**하고 런타임 CacheFirst 로만 잡기 때문이다
   (`vite.config.ts:130-146`).

---

## 1. 지금 이미 있는 것

### 1.1 배포 파이프라인 (실측)

`.github/workflows/deploy-pages.yml` 전 단계:

| 단계 | 명령 | 줄 |
| --- | --- | --- |
| 툴체인 | `dtolnay/rust-toolchain`, `targets: wasm32-unknown-unknown` | 41·43 |
| wasm-pack | `./.github/actions/install-wasm-pack` (0.15.0 고정) | 46 |
| **WASM 빌드** | `wasm-pack build --target web --release` | 60 |
| 산출 복사 | `cp pkg/rhwp_bg.wasm rhwp-studio/public/` · `cp pkg/rhwp.js ...` | 74-75 |
| 프런트 빌드 | `npx vite build --base=/rhwp/` | 82 |
| 배포 | `actions/deploy-pages` | 100 |

트리거는 `main` 푸시이며 `mydocs/**`·`docs/**`·`samples/**` 변경은 제외한다(6-19행).
공개 URL 은 `https://edwardkim.github.io/rhwp/` (`npm/editor/package.json:32` `homepage`).

버전 고정 정책은 [../wasm_pack_version_policy.md](../wasm_pack_version_policy.md).

### 1.2 PWA·서비스워커 (실측)

`rhwp-studio/vite.config.ts` 의 `VitePWA` 설정:

```ts
// vite.config.ts:131-146
workbox: {
  // WASM (~12 MB) is kept out of precache to avoid blocking SW installation;
  // CacheFirst at runtime still gives offline access after the first load.
  globPatterns: ['**/*.{js,css,html,png,svg,ico,woff,woff2,ttf,otf}'],
  maximumFileSizeToCacheInBytes: 20 * 1024 * 1024,
  runtimeCaching: [
    { urlPattern: /\.wasm$/, handler: 'CacheFirst',
      options: { cacheName: 'wasm-cache',
                 expiration: { maxEntries: 5, maxAgeSeconds: 30*24*60*60 } } },
  ],
},
devOptions: { enabled: false },
```

읽어야 할 사실 넷:

- `globPatterns` 에 **`wasm` 확장자가 없다.** 의도적 제외이며 주석이 이유를 적는다.
- 상한이 20 MB 다(`vite.config.ts:135`). `.wasm` 이 이걸 넘으면 런타임 캐시도 실패한다 —
  **그래서 실제 크기를 재야 한다.**
- 캐시 만료 30일, 최대 5개.
- **dev 서버에서는 SW 가 꺼져 있다**(`devOptions.enabled: false`). 오프라인 검증은
  `vite build` + `preview` 로만 가능하다.

매니페스트(`vite.config.ts:102` 이후)는 `start_url`/`scope` 가 `/rhwp/` 이고(110행),
`file_handlers` 로 `.hwp`·`.hwpx`·`.hml` 을 등록한다(112행) — **설치형 PWA 로 OS 파일
연결까지 잡는다.**

### 1.3 무엇이 이미 "설치 0"이고 무엇이 아닌가

| | 상태 |
| --- | --- |
| 브라우저로 HWP **열기·보기** | ✅ 이미 된다 |
| **편집·내보내기**(HWP/HWPX/HML) | ✅ 이미 된다 (`rpc-router.ts:90-94`) |
| PWA 설치·파일 연결 | ✅ 이미 된다 |
| iframe 임베드 API | ✅ `@rhwp/editor` (`npm/editor/`) |
| **에이전트 동사 시연** | ❌ WASM 에 노출 없음 |
| **자기서술 조회** | ❌ [self_description.md §1.2](self_description.md) |
| **MCP-유사 호출** | ❌ [browser_bridge.md](browser_bridge.md) |
| 크기·성능 수치 공개 | ❌ 미측정 |

**즉 M24 셋째 줄의 실제 작업은 "페이지를 만드는 것"이 아니라 "이미 있는 페이지에
에이전트 축을 보이게 하는 것"이다.**

---

## 2. 크기 예산 — 잰 것과 안 잰 것

### 2.1 실측 (이 작업 트리, 2026-08-03)

```
$ ls -la rhwp-studio/public/
284769  rhwp.js              # wasm-bindgen 글루 (JS)
109086  rhwp.d.ts            # 타입 선언 (런타임 불필요)
 38196  rhwp_bg.wasm.d.ts    # 타입 선언 (런타임 불필요)
  1649  theme-init.js
   911  favicon.ico

$ du -sh assets/fonts
22M     assets/fonts
```

즉 **JS 글루만 278 KB** 다. 이건 잰 값이다.

### 2.2 미실측 — `.wasm` 본체

```
$ find . -maxdepth 5 -name "*_bg.wasm"
(결과 없음)

$ grep -n "wasm" .gitignore
11:/pkg/
12:*.wasm
```

**저장소에 `.wasm` 이 없다.** `pkg/` 도 `*.wasm` 도 제외돼 있다. 이 PC 는 rhwp 를
빌드하지 못하므로(별도 기록) 재려면 **CI 산출물이 필요하다.**

현재 인용 가능한 유일한 숫자는 소스 주석이다.

> `// WASM (~12 MB) is kept out of precache ...` — `rhwp-studio/vite.config.ts:132`

**이건 근거가 아니라 흔적이다.** 언제 잰 것인지, release 인지, wasm-opt 후인지,
gzip 전인지 알 수 없다. 설계 문서에서 이 숫자를 예산의 기준으로 쓰면 안 된다.

측정 절차(제안):

```bash
wasm-pack build --target web --release
ls -la pkg/rhwp_bg.wasm                    # 원본
gzip -c pkg/rhwp_bg.wasm | wc -c           # gzip 전송량
brotli -c pkg/rhwp_bg.wasm | wc -c         # brotli 전송량 (Pages 기본)
```

**세 숫자를 다 재야 한다.** 사용자가 실제로 내려받는 것은 압축본이고, 메모리에
올라가는 것은 원본이다. 하나만 적으면 다른 하나로 오해받는다.

### 2.3 폰트 22 MB — 진짜 큰 것

`assets/fonts` 는 22 MB 다(`du -sh`). 개별 최대치:

```
1579172  D2Coding-Bold.woff2
1485508  D2Coding-Regular.woff2
 765684  Cafe24Supermagic-Regular-v1.0.woff2
 613412  HappinessSansVF.woff2
```

`rhwp-studio/public/fonts` 는 `../../assets/fonts` 를 가리키는 링크 파일이다(18바이트).
그리고 SW `globPatterns` 는 `woff|woff2|ttf|otf` 를 **포함**한다
(`vite.config.ts:134`) — 즉 **폰트는 precache 대상**이다.

> **`.wasm` 을 12 MB 로 가정하더라도, 폰트가 그보다 크다.**
> 데모 페이지의 크기 예산은 WASM 이 아니라 폰트가 지배할 가능성이 높다.
> 다만 vite 빌드가 실제로 몇 개를 번들에 포함시키는지는 **확인되지 않음** —
> `dist/` 를 만들어 재지 않았다.

### 2.4 예산 표

| 구성요소 | 크기 | 출처 |
| --- | --- | --- |
| `rhwp.js` 글루 | 284,769 B | **실측** |
| `rhwp_bg.wasm` | **미측정** | 주석 `~12 MB` (근거 아님) |
| 폰트 전량 | 22 MB | **실측**(디렉터리 전체) |
| 데모용 폰트 부분집합 | 미측정 | 서브셋 정책 없음 |
| studio JS 번들 | 미측정 | `vite build` 미실행 |
| 샘플 문서 1건 | 8,704~33,792 B | **실측**(§5.2) |
| **합계** | **산출 불가** | — |

**합계를 지어내지 않는다.** 두 항목이 비어 있으면 예산은 없는 것이다.

### 2.5 최적화 여지 — wasm-opt 부재

```
$ grep -rn "wasm-opt" .github/workflows/ Dockerfile
(결과 없음)
```

워크플로우 어디에도 명시적 `wasm-opt` 단계가 없다. `Cargo.toml` 에도
`[package.metadata.wasm-pack]` 섹션이 없다(`grep` 확인). release 프로파일은
`lto = true`(`Cargo.toml:179-180`)다.

wasm-pack 이 release 빌드에서 wasm-opt 를 자동 실행하는지는 **확인되지 않음** —
버전(0.15.0)의 기본 동작을 문서로 확인하지 않았다. 이걸 확정하는 것이
크기 논의의 **선행 조건**이다.

---

## 3. 배포 형태 — 후보 4종

### A. 단일 HTML 파일

`.wasm` 을 base64 `data:` URI 로 인라인해 HTML 하나로 만든다.

- 매력: 진짜 "파일 하나". 이메일·이슈에 첨부 가능.
- 문제:
  - base64 는 **약 4/3 로 부푼다.** 12 MB 가정 시 16 MB HTML.
  - `wasm-pack --target web` 산출물은 `rhwp.js` + `rhwp_bg.wasm` **2파일 전제**이고
    글루가 `fetch`/`instantiateStreaming` 경로를 쓴다. 인라인하려면 로더를 바꿔야 한다 —
    **어느 정도 작업인지 확인되지 않음.**
  - `instantiateStreaming` 을 못 쓰면 컴파일이 느려진다(측정 안 함).
  - 폰트를 넣으면 파일이 감당 불가가 된다. 빼면 한글 렌더가 깨진다.

**판정: 지금은 채택하지 않는다.** 재검토 조건은 §2 의 실측이 나온 뒤다.

### B. 정적 사이트 (현행 GitHub Pages) — **채택**

이미 돌고 있는 것(§1.1)에 데모 경로를 추가한다.

- 장점: **추가 인프라 0.** brotli 압축·캐시·PWA·SW 가 이미 붙어 있다.
  URL 하나만 주면 되므로 온보딩 마찰이 실질적으로 0 이다.
- 단점: 첫 로드에 네트워크가 필요하다. 완전 오프라인 시작은 안 된다.

### C. CDN 배포 (npm 패키지 → jsDelivr/unpkg)

`@rhwp/editor` 는 이미 npm 에 나간다(`.github/workflows/npm-publish.yml`).

- 장점: `<script type="module">` 한 줄로 남의 페이지에 붙는다. `@rhwp/editor` 의
  iframe 임베드 모델과 잘 맞는다(`npm/editor/index.js`).
- 단점:
  - **제3자 origin 의존.** [browser_bridge.md §5](browser_bridge.md) 의 origin 경계
    논의와 직접 충돌한다. 문서 내용이 CDN 이 서빙한 코드를 지나간다.
  - CDN 이 죽으면 데모도 죽는다.
  - 저장소는 이미 **외부 CDN 폰트 로드를 끌 수 있게** 만들어 뒀다(§4.3) — 즉
    "외부 의존을 줄이자"는 방향이 이미 존재한다. C 는 그 방향의 반대다.

**판정: 데모의 기본 경로로는 쓰지 않는다.** 임베드용 배포로는 유지한다.

### D. 저장소 zip → `file:` 로 로컬 열기

- 장점: 네트워크 없이 배포 가능(USB·사내망).
- 문제: `file:` origin 은 불투명("null")이다. 그리고 embed 런타임은 **`"null"` origin 을
  명시적으로 거부**한다(`protocol.ts:96` — `if (!origin || origin === 'null') return false`).
  즉 **브리지가 동작하지 않는다.** SW 도 `file:` 에서 등록되지 않는다.

**판정: 브리지 없는 단순 뷰어 데모로만 가능.** 에이전트 축에는 부적합.

### 판정 요약

| | A 단일 HTML | **B 정적 사이트** | C CDN | D file: |
| --- | --- | --- | --- | --- |
| 추가 인프라 | 없음 | **없음(현행)** | 없음 | 없음 |
| 첫 로드 네트워크 | 필요 | 필요 | 필요 | 불필요 |
| 브리지 동작 | ○ | **○** | ○ | **✗** |
| 제3자 의존 | 없음 | 없음 | **있음** | 없음 |
| 구현 미지수 | **큼** | 없음 | 작음 | 없음 |
| SW 오프라인 | ✗ | **○** | ○ | ✗ |

**B 를 채택한다.** A 는 §2 실측 후 재검토, C 는 임베드 전용, D 는 뷰어 전용.

---

## 4. 오프라인 동작

### 4.1 지금 성립하는 오프라인의 정확한 범위

- SW 는 `autoUpdate` 로 등록된다(`vite.config.ts:100`).
- `js/css/html/png/svg/ico/woff/woff2/ttf/otf` 는 **precache** 된다(134행).
- `.wasm` 은 precache 되지 **않고**, 런타임 CacheFirst 로만 잡힌다(136-145행).

따라서:

> **첫 방문 이후에만 오프라인이다.** 그리고 첫 방문에서 `.wasm` 이 실제로 요청됐어야
> 한다 — 문서를 한 번도 열지 않고 나가면 캐시에 안 들어갈 수 있다. **확인되지 않음** —
> 스튜디오가 초기화 시점에 항상 WASM 을 받는지 대조하지 않았다
> (`wasm-bridge.ts:259` 의 `await init()` 호출 시점 확인 필요).

### 4.2 검증은 빌드본에서만 가능

`devOptions.enabled: false`(`vite.config.ts:147-149`) 때문에 `npm run dev` 에서는
SW 가 돌지 않는다. 오프라인 검증 절차:

```bash
cd rhwp-studio && npx vite build && npx vite preview
# 브라우저 DevTools → Application → Service Workers → Offline 체크 → 새로고침
```

### 4.3 외부 폰트 CDN — 데모에서 반드시 꺼야 한다

`rhwp-studio/src/core/font-loader.ts:41-43` 에 **jsDelivr 절대 URL 셋**이 있다.

```
https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_2104@1.0/HANBatang.woff
https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_2104@1.0/HANBatangB.woff
https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_four@1.0/HCRDotum.woff
```

끄는 스위치가 **이미 있다** — `vite.config.ts:17-21`:

```ts
__RHWP_DISABLE_EXTERNAL_WEBFONTS__: JSON.stringify(
  process.env.RHWP_DISABLE_EXTERNAL_WEBFONTS === '1',
),
```

주석: "셀프 호스팅 빌드에서 외부(CDN) 웹폰트 로드를 빌드 시점에 끈다."

**오프라인 데모 빌드는 `RHWP_DISABLE_EXTERNAL_WEBFONTS=1` 로 만든다.** 안 그러면
오프라인에서 폰트 요청이 실패하고, 더 중요하게는 **제3자에게 요청이 나간다** —
문서를 보는 행위가 외부에 관측된다. 이건 성능 문제가 아니라 **경계 문제**다
([browser_bridge.md §5.2](browser_bridge.md)).

### 4.4 CSP

확장 페이지는 `script-src 'self' 'wasm-unsafe-eval'` 을 명시한다
(`rhwp-chrome/manifest.json:43`, `rhwp-firefox/manifest.json:52`).
데모 페이지도 같은 제약을 전제로 쓴다 — **인라인 스크립트 금지**.
studio `index.html:8-10` 이 이미 그 규칙을 따르며 이유를 주석에 남긴다.

이 제약은 §3-A(단일 HTML)에 또 하나의 벽이다. 인라인 `data:` 로더는 CSP 와 싸운다.

---

## 5. 데모 페이지 설계

### 5.1 무엇을 보여주나 — 렌더가 아니라 동사

기존 studio 는 **문서를 예쁘게 보여주는 것**을 시연한다. 데모 페이지는 다르다.
#3869 의 주장 — "이해도 실행도 아니다. 시작이다" — 을 화면으로 옮긴다.

보여줄 것(우선순위 순):

1. **자기서술** — `capabilities()` 결과를 첫 화면에 그대로. "이 모듈이 할 수 있는 일"
   ([self_description.md §4.1](self_description.md)).
2. **`digest`** — #3869 §0 이 인용하는 실측(393쪽 문서 1,375 B vs `export-text`
   645,108 B)을 **브라우저에서 재현**. 이 대비가 이 축의 핵심 주장이다.
3. **`fields` → `fill` → `verify`** — 양식 왕복. `exportHwpVerify` 는 **이미 WASM 에
   있다**(`wasm_api.rs:5733`).
4. **`search` / `extract-data`** — 주소(구역·문단·페이지·오프셋)가 붙은 결과.
5. **`inspect`** — 출처 표지가 왜 필요한지 보여주는 자리
   ([../agent_security/threat_model.md](../agent_security/threat_model.md)).

**1·3·4 의 일부는 지금 WASM 에 이미 있다.** 2·5 는 자기서술 조각 S5 이후에 가능하다.

### 5.2 번들 샘플 — 실측 크기

`rhwp-studio/public/samples/` 에 이미 문서가 실려 나간다(실측):

```
  8704  shift-return.hwp
 16384  para-head-num-2.hwp
 16896  oullim-01.hwp
 24576  BlogForm_BookReview.hwp
 33792  biz_plan.hwp
131571  form-002.hwpx
514560  field-01.hwp
```

데모 기본 문서는 **`biz_plan.hwp`(33,792 B)** 또는 양식 시연용 **`form-002.hwpx`
(131,571 B)** 가 적당하다. `field-01.hwp`(514,560 B)는 누름틀이 많지만 무겁다.

**파일 업로드 없이도 1클릭으로 동작해야 한다.** 사용자가 자기 HWP 를 찾아 올리는
순간 "설치 0"의 마찰 감소가 절반 사라진다.

### 5.3 3단계 시나리오

| 단계 | 사용자 행동 | 보여주는 것 | 필요 조각 |
| --- | --- | --- | --- |
| 0초 | URL 열기 | `capabilities()` 목록 | 자기서술 S3 |
| 5초 | "예제 열기" 1클릭 | `digest` 봉투 (수 KB) | S5 |
| 15초 | 필드 채우기 | `fields`→`fill`→`verify` 왕복 | 이미 대부분 존재 |
| — | "내 에이전트에 붙이기" | 브리지 스니펫 | [browser_bridge.md](browser_bridge.md) B3 |

마지막 줄이 온보딩의 목적이다. 데모는 장난감이 아니라 **진입점**이다.

### 5.4 봉투를 화면에 그대로 보여준다

가공된 UI 카드가 아니라 **JSON 봉투 원문**을 보여준다. 이유:

- 소비자는 에이전트다. 사람용 요약은 그들이 볼 것이 아니다.
- `untrustedContent`/`untrustedFields` 표지가 **눈에 보여야** 소비자가 그 존재를 안다
  ([self_description.md §4.4](self_description.md)).
- CLI 출력과 나란히 두면 **동등성이 시각적으로 검증**된다(#3869 W2).

---

## 6. 이 문서가 다루지 않는 것

#3869 는 W3(Python 휠)·W4(npm 패키지)로 **브라우저 밖 설치 0**도 제안했다.

```
pip install rhwp    # 바이너리 없음. WASM 이 휠 안에 들어 있다
npm install rhwp    # 동일
```

**이 문서는 그 축을 다루지 않는다.** 해당 Python·Node 패키지 제안은 v0.8.4에서
공식 채택이 철회됐다(#4655). 브라우저 WASM 표면은 기존 공식 웹 배포 범위 안에서
별도로 검증한다.

---

## 7. 조각 분해

| # | 조각 | 선행 | 검증 |
| --- | --- | --- | --- |
| Z1 | **`.wasm` 크기 실측** (원본·gzip·brotli) | 없음 | CI 로그에 숫자가 남는다 |
| Z2 | wasm-pack 0.15.0 의 wasm-opt 기본 동작 확정 | 없음 | 문서·로그 |
| Z3 | 오프라인 실검증 (`vite build` + SW Offline) | 없음 | 재현 절차 기록 |
| Z4 | 데모 라우트 추가 (`/rhwp/demo/`) | 자기서술 S3 | 첫 화면에 `capabilities()` |
| Z5 | `RHWP_DISABLE_EXTERNAL_WEBFONTS=1` 데모 빌드 | Z3 | 네트워크 탭에 외부 요청 0 |
| Z6 | 1클릭 예제 문서 배선 | Z4 | 업로드 없이 동작 |
| Z7 | `digest` 대비 시연 | 자기서술 S5 | 봉투 크기 실측 표기 |
| Z8 | "에이전트에 붙이기" 스니펫 | 브리지 B3 | 복사→동작 |

**Z1 이 첫 조각이다.** 크기를 모르면 §3 의 판정을 재검토할 수도 없고,
20 MB SW 상한(`vite.config.ts:135`)에 걸리는지도 모른다.

---

## 8. 확인되지 않음

1. **`rhwp_bg.wasm` 크기 (원본·gzip·brotli)** — 이 축 전체에서 가장 큰 공백.
2. **wasm-pack 0.15.0 release 빌드의 wasm-opt 기본 동작** (§2.5).
3. **vite 빌드가 실제로 번들하는 폰트 집합** — 22 MB 전량인지 부분인지 미확인.
4. **초기화 시 `.wasm` 이 항상 요청되는가** — SW 런타임 캐시 진입 조건(§4.1).
5. **`data:` URI 인라인 로더의 작업량** — `wasm-pack --target web` 글루 수정 범위(§3-A).
6. **`file:` 에서 뷰어만이라도 도는가** — SW·모듈 스크립트 제약 미검증(§3-D).
7. **첫 로드 시간·WASM 컴파일 시간** — 측정하지 않았다. **이 문서에 성능 수치가
   하나도 없는 이유다.**

---

## 9. 관련 문서

- [README.md](README.md) — 이 축의 지도
- [self_description.md](self_description.md) — 데모 첫 화면이 보여줄 것
- [browser_bridge.md](browser_bridge.md) — "에이전트에 붙이기"의 실체
- [../wasm_pack_version_policy.md](../wasm_pack_version_policy.md) — 툴체인 고정
- [CLI 명령 레퍼런스](../../manual/cli_commands.md) — 현재 진입로 계약
- [../agent_security/threat_model.md](../agent_security/threat_model.md) — 데모에서
  `inspect` 를 보여줘야 하는 이유
- [../agent_security/consumer_guide.md](../agent_security/consumer_guide.md) — 붙인 다음의 책임
- 이슈 [#3608](https://github.com/edwardkim/rhwp/issues/3608) M24 ·
  [#3869](https://github.com/edwardkim/rhwp/issues/3869) §1·W3~W6
