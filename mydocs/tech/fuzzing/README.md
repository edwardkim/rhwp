---
kind: guide
status: active
canonical: mydocs/tech/fuzzing/agent_surface_robustness.md
last_verified: 2026-08-03
---

# 퍼징·견고성 문서 지도

`mydocs/tech/fuzzing/` 은 rhwp 의 **퍼징 인프라를 어떻게 운영하는가**를 다룬다.
로드맵 [#3608](https://github.com/edwardkim/rhwp/issues/3608) **M21 — 퍼징·견고성**의
운영 문서이고, 근거가 되는 RFC 는 [#3141](https://github.com/edwardkim/rhwp/issues/3141) 이다.

> **이 묶음은 새 인프라를 제안하지 않는다.** 퍼징 인프라는 **이미 저장소에 있다**
> (`fuzz/`, 커밋 `e78883617`·`2b87440ea`). 이 문서들은 그것을 **돌리고, 나온 것을
> 처리하고, 에이전트 표면 계약과 연결하는 방법**을 적는다.

이 묶음의 모든 기술 주장에는 **코드 경로(`파일:줄`)·커밋 해시·실제 명령 출력**이 붙는다.
근거를 대지 못한 항목은 **"확인되지 않음"** 으로 적었다.

---

## 1. 지금 있는 것 — 실측 (2026-08-03, `rhwp v0.8.2`, HEAD `9095cd52d`)

### 1-1. 하네스 6개

`fuzz/Cargo.toml` 의 `[[bin]]` 6개, `fuzz/fuzz_targets/*.rs` 6개 (`ls fuzz/fuzz_targets/` 실측).

| 타깃 | 겨냥하는 진입점 | 층 |
| --- | --- | --- |
| `parse_hwp` | `rhwp::parser::parse_hwp(&[u8])` — HWP 5.x, CFB 컨테이너 | 포맷 최상위 |
| `parse_hwp3` | `rhwp::parser::hwp3::parse_hwp3(&[u8])` — HWP 3.x 바이너리 | 포맷 최상위 |
| `parse_hwpx` | `rhwp::parser::hwpx::parse_hwpx(&[u8])` — HWPX, ZIP 컨테이너 | 포맷 최상위 |
| `parse_hml` | `rhwp::parser::hml::parse_hml(&[u8])` — HML, XML | 포맷 최상위 |
| `parse_wmf` | `WMFConverter::new(data, SVGPlayer::new()).run()` | 임베드 이미지 |
| `parse_ooxml_chart` | `rhwp::ooxml_chart::parser::parse_chart_xml(&[u8])` | 임베드 차트 |

6개 전부 `let _ = parse_xxx(data);` 형태다 — `Err` 반환은 **정상**이고, 검출 대상은
패닉·abort·OOM·타임아웃뿐이다(각 하네스 파일 상단 주석).

### 1-2. 시드 코퍼스 12개 (269KB)

`fuzz/corpus/<타깃>/` 실측: `parse_hwp` 3 · `parse_hwp3` 2 · `parse_hwpx` 3 ·
`parse_hml` 2 · `parse_wmf` 1 · `parse_ooxml_chart` 1. 출처와 확장 방법은
[operations.md §3](operations.md) 참조.

### 1-3. 도입 이력

| 단계 | 커밋 | 무엇 |
| --- | --- | --- |
| 1단계 (#3158) | `e78883617` (2026-07-24) | `fuzz/` 독립 크레이트 + 하네스 4개 + 시드 10개 + `fuzz/README.md` |
| 2단계 (#3273→#3275) | `2b87440ea` (2026-07-25) | `parse_wmf`·`parse_ooxml_chart` 하네스 2개 + 합성 시드 2개 |

`fuzz/Cargo.toml` 의 `[workspace] members = ["."]` 로 루트 워크스페이스에서 분리돼 있어
본 크레이트의 빌드·`cargo test` 에 영향이 없다(`fuzz/Cargo.toml` 15~17행).

---

## 2. RFC #3141 의 계획 대비 어디까지 왔나

RFC 6장이 정한 6단계와 7장(OSS-Fuzz)의 현재 상태다.

| RFC 항목 | 상태 | 근거 |
| --- | --- | --- |
| ① `cargo fuzz init` 스캐폴드 | **완료** | `fuzz/` 존재, 커밋 `e78883617` |
| ② 1순위 하네스 4개 | **완료** | `fuzz/fuzz_targets/parse_{hwp,hwp3,hwpx,hml}.rs` |
| ③ 시드 코퍼스 | **완료** | `fuzz/corpus/` 12개 |
| ④ `-rss_limit_mb=2048 -timeout=30` 기본 | **문서화 완료** | `fuzz/README.md` §권장 플래그 |
| ⑤ CI 스모크 퍼징 | **미착수** | `.github/workflows/` 12개 파일 전수 grep 결과 `fuzz` 0건 |
| ⑥ 2순위 내부 파서 하네스 | **부분** — WMF·OOXML 차트만. `parse_body_text_section`·`parse_doc_info`·`parse_control`·EMF 없음 | `fuzz/Cargo.toml` `[[bin]]` 6개 |
| ⑦ OSS-Fuzz 등재 | **확인되지 않음** | 저장소에 `project.yaml`·`build.sh` 없음. 신청 여부는 이 저장소에서 확인 불가 |
| ⑧ `fuzz/regressions/<타깃>/` | **미생성** | `ls fuzz/regressions` → `No such file or directory` |

메인테이너 확인 (#3141 코멘트 2건):
- "1단계 구현 #3158 이 devel `e78883617e` 로 merge … 이 이슈는 2단계 이후 진행을 위해 열어 둡니다"
- "2단계 … `2b87440ea9` 로 merge … 로드맵 잔여: EMF 등 내부 파서 직접 하네스, CI 통합, OSS-Fuzz 등재"

### 현황판 드리프트 — 고쳐야 할 것

로드맵 #3608 §8 의 M21 체크리스트는 **네 항목 모두 미체크**다.

```
### M21 — 퍼징·견고성 (#3141 RFC 실행)
- [ ] cargo-fuzz 타깃 4종(HWP5/HWP3/HWPX/HML 파서)
- [ ] 크래시 코퍼스 회귀 스위트 편입
- [ ] OSS-Fuzz 등재 신청
- [ ] 발견 결함의 이슈→수정 파이프라인 실적 공개
```

그런데 첫 항목은 `e78883617`(4종) + `2b87440ea`(총 6종)로 **이미 머지됐다**.
#3608 본문이 "체크 = 머지 · 진행률의 유일 기준"이라고 선언하므로, 이 어긋남은
진행률을 실제보다 낮게 보이게 한다. **현황판 갱신이 M21 의 첫 작업이다.**

---

## 3. 이 묶음의 문서 4개

| 문서 | 무엇을 답하나 |
| --- | --- |
| **README.md** (이 문서) | 지금 뭐가 있고 RFC 대비 어디까지 왔나 |
| [operations.md](operations.md) | **어떻게 돌리나** — 명령·플래그·코퍼스·시간 예산·CI·이 PC 실측 |
| [crash_triage.md](crash_triage.md) | **나왔을 때 어떻게 하나** — 최소화·재현·층 판별·회귀 테스트 승격 |
| [agent_surface_robustness.md](agent_surface_robustness.md) | **안 죽는 것만으로 충분한가** — 봉투·exit·부분 결과 (이 묶음의 canonical) |

읽는 순서: 처음이면 이 문서 → `operations.md`. 크래시를 손에 들고 있으면
곧장 `crash_triage.md`. 왜 이 축이 에이전트 도구에서 특별한지는
`agent_surface_robustness.md`.

---

## 4. 인접 축과의 경계 — 이 묶음이 다루지 않는 것

[에이전트 보안 위협 모델](../agent_security/threat_model.md) §1.1 의 도식이 경계를 이미 그어 놓았다.

```
 ┌──────────────┐        ┌────────────────┐        ┌──────────────────┐
 │  .hwp/.hwpx  │  ①    │  rhwp 프로세스  │  ②    │ 에이전트 컨텍스트 │
 └──────────────┘ ─────▶ └────────────────┘ ─────▶ └──────────────────┘
   신뢰 없음               메모리 안전                 지시로 읽힘
                          (Rust, fuzz 대상)
```

- **① 파일 → rhwp** — **이 묶음의 주제.** threat_model.md 는 이 경계를 "메모리 안전은
  Rust 와 `fuzz/` 하네스가 담당하며, **이 문서의 주제가 아니다**"라고 명시적으로 넘긴다.
  넘겨받은 곳이 여기다.
- **② rhwp → 에이전트 컨텍스트** — [agent_security/](../agent_security/README.md) 의 주제.
  단, ①의 실패가 ②의 봉투로 어떻게 새어 나오는지는 경계가 겹친다 —
  그 겹침이 [agent_surface_robustness.md](agent_surface_robustness.md) 다.

| 축 | 다루는 실패 | 문서 |
| --- | --- | --- |
| 퍼징(이 묶음) | 패닉 · abort · OOM · 무한루프 | 여기 |
| 에이전트 보안 | 문서가 에이전트를 조종함 | [agent_security/](../agent_security/README.md) |
| 경계 무결성 | 경로·교정단서·자원한계·핸들 | [agent_boundary_contract.md](../agent_boundary_contract.md) |
| 왕복 정합성 | 정상 입력의 1속성 소실 | #2740 (닫힘). 퍼징 대상 아님 — RFC #3141 §3 |

---

## 5. 확인되지 않음 (추측으로 채우지 않은 칸)

| 항목 | 왜 확인 못 했나 | 확인하려면 |
| --- | --- | --- |
| `cargo +nightly fuzz build` 가 이 저장소에서 통과하는가 | nightly·cargo-fuzz 미설치, MSVC 링크 불가([operations.md §6](operations.md)) | Linux/macOS 또는 CI 에서 1회 실행 |
| 누적 퍼징 실행 시간·발견 건수 | 저장소에 실행 기록 없음. `fuzz/artifacts/` 는 `.gitignore` 대상 | 실행 로그를 남기는 규약부터 정해야 함 |
| OSS-Fuzz 등재 신청 여부 | 저장소 밖 정보 | `google/oss-fuzz` 저장소 검색, 메인테이너 확인 |
| `fuzz/README.md` 의 Windows `rust-lld` 우회가 실제로 통하는가 | 이 PC 는 그 앞 단계(nightly 부재)에서 막힘 | Windows + nightly 환경에서 실측 |
| 코퍼스 12개의 실제 커버리지 비율 | `cargo fuzz coverage` 미실행 | 퍼징 가능한 환경에서 측정 |

---

## 6. 이 문서를 다시 검증하는 법

6개월 뒤 아래를 그대로 돌려 §1 표가 아직 참인지 확인한다.

```sh
ls fuzz/fuzz_targets/                       # 하네스 개수
grep -c '^\[\[bin\]\]' fuzz/Cargo.toml      # 등록된 타깃 수 (일치해야 함)
du -sh fuzz/corpus                          # 코퍼스 크기
ls fuzz/regressions 2>/dev/null || echo "미생성"
grep -ril fuzz .github/                     # CI 편입 여부 (0건이면 수동 전용)
```

`ls fuzz/fuzz_targets/` 개수와 `[[bin]]` 개수가 어긋나면 **어느 한쪽이 죽은 것**이다 —
`[[bin]]` 이 없는 하네스는 빌드되지 않고, 파일 없는 `[[bin]]` 은 빌드를 깬다.

## 관련

- 로드맵: [#3608](https://github.com/edwardkim/rhwp/issues/3608) M21 · RFC: [#3141](https://github.com/edwardkim/rhwp/issues/3141)
- 저장소 안 운영 문서: [`fuzz/README.md`](../../../fuzz/README.md) (실행 명령의 1차 출처)
- 제보 경로: [`SECURITY.md`](../../../SECURITY.md) · [agent_security/disclosure.md](../agent_security/disclosure.md)
