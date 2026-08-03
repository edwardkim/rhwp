---
kind: guide
status: active
canonical: mydocs/tech/wasm_agent_surface/README.md
last_verified: 2026-08-03
---

# WASM/브라우저 에이전트 표면 문서 지도

`mydocs/tech/wasm_agent_surface/`는 **로드맵 [#3608](https://github.com/edwardkim/rhwp/issues/3608)
M24 "WASM/브라우저 에이전트 표면"** 의 설계 문서를 모은다. 같은 축을 다루는 이슈
[#3869](https://github.com/edwardkim/rhwp/issues/3869)("설치 없는 실행")와 정렬해 쓴다.

이 축의 모든 기술 주장에는 **코드 경로(`파일:줄`) 또는 실제 명령 출력**이 붙는다.
근거를 대지 못하는 항목은 **"확인되지 않음"** 으로 적었다. 특히 **성능·크기는 지어내지
않는다** — 이 저장소는 `*.wasm` 을 커밋하지 않으므로(`.gitignore:12`) 번들 크기 대부분이
아직 실측되지 않은 상태다.

## 왜 이 축이 생겼는가

M24 는 세 줄짜리 체크리스트다(#3608 본문 §8 "장기 지평(M18~M30)" 안의 M24 절).

```
### M24 — WASM/브라우저 에이전트 표면
- [ ] wasm_api 에 capabilities 자기서술 대응물
- [ ] 브라우저 내 MCP-유사 브리지 설계(studio 연동)
- [ ] 오프라인 데모 페이지(설치 0 온보딩)
```

세 줄이 각각 다른 공백을 가리킨다.

- **자기서술** — CLI 는 `rhwp capabilities` 로, MCP 는 `tools/list` 로 자기를 서술한다.
  브라우저 안의 소비자에게는 **대응물이 없다.** `src/wasm_api.rs` 에는 `capabilities`
  라는 문자열도, 봉투 `schemaVersion` 도 등장하지 않는다(실측: §"재현" 참조).
- **브리지** — 브라우저에는 stdio 가 없다. MCP JSON-RPC 를 무엇에 얹을지가 미정이다.
  다만 **바닥부터 시작하지 않는다** — studio 는 이미 `MessageChannel` 기반 RPC 를 돌린다
  (`rhwp-studio/src/embed/runtime.ts`, 162줄).
- **온보딩** — #3869 가 말하는 "모든 진입로가 공유하는 첫 관문"(바이너리 확보)을 없애는
  경로다. rhwp 는 이미 GitHub Pages 로 배포된다(`.github/workflows/deploy-pages.yml`).

## 문서 지도

| 문서 | kind | 다루는 것 | 언제 읽나 |
| --- | --- | --- | --- |
| [WASM capabilities 자기서술 설계](self_description.md) | canonical | 브라우저 소비자가 "이 모듈이 뭘 할 수 있나"를 아는 방법, CLI 와의 동등성 유지 | M24 첫 줄을 구현할 때. **이 축의 전제** |
| [브라우저 내 MCP-유사 브리지](browser_bridge.md) | canonical | JSON-RPC 를 postMessage·MessageChannel·Worker 중 무엇에 얹을지, 그리고 origin 경계 | M24 둘째 줄. studio 연동을 설계할 때 |
| [설치 0 온보딩 데모](zero_install_onboarding.md) | guide | 아무것도 설치하지 않고 rhwp 를 써보는 경로, 크기 예산, 오프라인 동작 | M24 셋째 줄. #3869 의 진입점 |

셋은 **순서대로 의존한다.** 자기서술이 없으면 브리지가 무엇을 노출할지 정할 수 없고,
브리지가 없으면 데모 페이지는 "보기 좋은 뷰어"에서 멈춘다.

## 0. 지금 확정된 실측 (2026-08-03)

이 축을 논할 때 반복해서 인용되는 숫자다. 재현 명령은 §"재현"에 있다.

| 항목 | 값 | 근거 |
| --- | --- | --- |
| `src/wasm_api.rs` 줄 수 | 7,621 | `wc -l src/wasm_api.rs` |
| `wasm_bindgen` 출현 | 372곳 | `grep -c wasm_bindgen src/wasm_api.rs` |
| 고유 `js_name` export | 360개 | `grep -oE "js_name\s*=\s*[A-Za-z0-9_]+" \| sort -u \| wc -l` |
| 고유 `pub fn` | 367개 | 같은 방식 |
| CLI 명령 수 / `--json` 계약 | 61 / 31 | `rhwp capabilities \| jq` |
| MCP 도구 수 | 39 | `rhwp capabilities --mcp \| jq '.tools \| length'` |
| `wasm_api.rs` 안의 `capabilities`·`schemaVersion` | **0곳** | `grep -c` (§자기서술 1.2) |
| `rhwp.js` 글루 크기 | 284,769 B | `ls -la rhwp-studio/public/rhwp.js` |
| `rhwp_bg.wasm` 크기 | **미실측** | `.gitignore:12` 가 `*.wasm` 제외 — 이 작업 트리에 산출물 없음 |
| 번들 폰트 총량 | 22 MB | `du -sh assets/fonts` |

**`rhwp_bg.wasm` 크기가 미실측이라는 점을 가볍게 넘기지 마라.** `rhwp-studio/vite.config.ts:132`
주석이 `WASM (~12 MB)` 라고 적지만 이건 **소스 주석이지 측정치가 아니다.** 온보딩 설계의
모든 크기 논의는 이 숫자에 걸려 있고, 이 숫자는 아직 없다
([zero_install_onboarding.md §2](zero_install_onboarding.md)).

## 1. 이 축이 하지 않는 것

#3869 §4 를 그대로 승계한다. 문서가 약속을 넓히면 구현이 따라가지 못한다.

- **기존 CLI·MCP·바인딩을 대체하지 않는다.** WASM 표면은 **네 번째 진입로**이지 교체가 아니다.
- **새 판정·편집 로직을 만들지 않는다.** `src/document_core/` 의 코어를 노출할 뿐이다.
  다행히 그 코어는 이미 lib 안에 있다(§자기서술 2.3).
- **렌더링 API 를 에이전트 표면에 섞지 않는다.** `renderPageToCanvas` 계열 12종은
  브라우저 UI 의 것이다. 에이전트 동사와 같은 이름공간에 두지 않는다.
- **성능이 네이티브와 같다고 주장하지 않는다.** 이 축의 문서 어디에도 WASM 성능
  수치는 없다 — **측정하지 않았기 때문이다.**

## 2. 세 문서가 공유하는 결론 하나

조사에서 나온 가장 중요한 사실은 세 문서에 모두 걸린다.

> **에이전트 동사의 *로직* 은 이미 lib 안에 있고 wasm32 로 컴파일된다.
> 없는 것은 *봉투* 와 *노출* 이다.**

- `extract_data` 는 `src/document_core/queries/extract_data.rs:850` 에 있다(1,251줄).
- 인젝션 판정은 `injection_scan.rs:1034` 에 있다(1,467줄).
- 표 격자 추출은 `table_extract.rs:292` 에 있다(304줄).
- 그런데 **봉투를 조립하는 코드는 전부 `src/main.rs`(17,058줄) 안**, 즉 **bin 크레이트**다.
  `capabilities_command_entries()` 는 `src/main.rs:1463` 에 있고, lib(`src/lib.rs`, 54줄)은
  그것을 볼 수 없다.

따라서 M24 는 "WASM 에 기능을 추가하는 일"이 아니라 **"봉투 층을 bin 에서 lib 으로
끌어내리는 일"** 에 가깝다. 이 판단의 근거와 반례는
[self_description.md §2](self_description.md) 에 있다.

## 3. 재현

이 지도의 숫자를 다시 만드는 명령이다. 저장소 루트에서 실행한다.

```bash
wc -l src/wasm_api.rs src/main.rs src/lib.rs
grep -c wasm_bindgen src/wasm_api.rs
grep -oE "js_name\s*=\s*[A-Za-z0-9_]+" src/wasm_api.rs | sed 's/.*= *//' | sort -u | wc -l

# 자기서술 부재 확인 — 0 이 나와야 한다
grep -cE "text_security|inspect|extract_data|digest|export_tables|schemaVersion" src/wasm_api.rs

# CLI 자기서술
rhwp capabilities | jq '{commands: (.commands|length), json: ([.commands[]|select(.json)]|length)}'
rhwp capabilities --mcp | jq '.tools | length'

# 크기
ls -la rhwp-studio/public/rhwp.js
du -sh assets/fonts
```

`rhwp` 는 릴리스 바이너리를 가리킨다. 이 PC 에서는
`target/release/rhwp.exe` 로 실행해 위 수치를 얻었다.

## 4. 확인되지 않음 (이 축 전체)

정직하게 열어 둔다. 채워지기 전에는 설계 결정의 근거로 쓰지 않는다.

1. **`rhwp_bg.wasm` 실제 크기** — release·wasm-opt 적용 후 값. 이 PC 는 rhwp 를
   빌드하지 못한다(별도 기록). CI 산출물에서 재야 한다.
2. **WASM vs 네이티브 성능** — #3869 수용 기준이 요구하지만 아직 아무 측정이 없다.
3. **`document_core/queries/` 전 모듈의 wasm32 컴파일 가능성** — `grep.rs`·
   `search_query.rs`·`form_query.rs` 가 `std::fs`/`std::io` 를 참조한다. 실제로 wasm32
   빌드가 깨지는지는 빌드해 보지 않았다.
4. **wasm-pack 의 wasm-opt 기본 동작** — 워크플로우에 `wasm-opt` 단계가 없고
   `Cargo.toml` 에 `[package.metadata.wasm-pack]` 도 없다. 기본값이 무엇인지 미확인.

## 5. 관련 문서

- **보안** — [에이전트 보안 문서 지도](../agent_security/README.md),
  [위협 모델](../agent_security/threat_model.md),
  [공격 표면](../agent_security/attack_surface.md),
  [소비 에이전트 가이드](../agent_security/consumer_guide.md).
  브라우저는 **새 경계**를 추가한다(origin). 위협 모델의 `#3787` 축과 어긋나지 않게 쓴다 —
  [browser_bridge.md §5](browser_bridge.md).
- **외부 바인딩** — [IR 스키마 버저닝·표면 판단](../bindings_foundation.md)(M18~M20).
  WASM 표면은 그 판단표에 **네 번째 열**로 들어간다.
- **툴체인** — [wasm-pack 버전 고정 정책](../wasm_pack_version_policy.md).
- **경계 계약** — [에이전트 경계 무결성 계약](../agent_boundary_contract.md).

## 6. 이슈

- [#3608](https://github.com/edwardkim/rhwp/issues/3608) — 마일스톤 현황판. **M24 의 권위**
- [#3869](https://github.com/edwardkim/rhwp/issues/3869) — 설치 없는 실행. W1~W6 조각 정의
- [#3787](https://github.com/edwardkim/rhwp/issues/3787) — 에이전트 보안 구현
- [#3719](https://github.com/edwardkim/rhwp/issues/3719) — 상위 6층 아키텍처 지도
