---
kind: reference
status: active
canonical: mydocs/tech/agent_runtime/surface_spec.md
last_verified: 2026-08-03
---

# 진입로 비용 모델 — 실측값과 재현 방법

> **v0.8.4 현행성 주의:** Python·Node 바인딩 측정은 철회 전 실험 기록이다.
> #4655 이후 현재 지원 표면의 성능 기준으로 사용하지 않는다.

[entrypoint_decision.md](entrypoint_decision.md) 의 모든 성능 주장이 여기서 나온다.
숫자마다 **어느 문서·몇 회·어느 머신**인지를 붙인다. 조건 없는 숫자는 거짓말이다.

측정하지 못한 축은 §10 에 "미측정"으로 명시했다. **추정치를 쓰지 않았다.**

- 진입로 선택: [entrypoint_decision.md](entrypoint_decision.md)
- 실패 증상: [failure_dictionary.md](failure_dictionary.md)
- 봉투 동등성 계약: [envelope_parity.md](envelope_parity.md)
- WASM 표면 설계: [surface_spec.md](surface_spec.md) — 성능은 그쪽도 미측정
- 로드맵 이슈: [#3869](https://github.com/edwardkim/rhwp/issues/3869)

---

## 1. 측정 환경

| 항목 | 값 |
|---|---|
| OS | Windows 11 Home 10.0.26200 |
| 논리 코어 | 32 (`nproc`) |
| 바이너리 | `target/release/rhwp.exe` **v0.8.2**, 18,108,416 B (2026-08-03 빌드) |
| 셸 | Git Bash (`date +%s%N` 로 벽시계 측정) |
| Python | 3.11.9 (MCP·바인딩 측정용) |
| 측정일 | 2026-08-03 |
| 코퍼스 | 저장소 `samples/` |

> **이 머신의 특이점**: 실시간 보호(AV) 가 동작 중인 개인 PC 다. 프로세스 기동
> 바닥값(§2)이 서버 환경보다 높을 수 있다. 절대값이 아니라 **경로 간 상대비**로
> 읽어라. 상대비는 같은 머신·같은 세션에서 잰 것이라 유효하다.

### 측정에 쓴 문서

| 티어 | 경로 | 크기 | 쪽 | 문단 |
|---|---|---:|---:|---:|
| 소형 | `samples/복학원서.hwp` | 114,688 B | 1 | 17 |
| 중형 | `samples/k-water-rfp-2024.hwp` | 2,716,160 B | 27 | — |
| 대형 | `samples/2025 행정업무운영 편람(최종).hwp` | 10,687,488 B | 393 | 2,618 |

### 배치 코퍼스 (N=20 / N=10)

```bash
ls -S samples/*.hwp | grep -v password | sed -n '11,30p' > corpus20.txt   # 20건
head -10 corpus20.txt > corpus10.txt                                      # 10건
```

총 **84,021,553 B**(평균 4.20 MB/건). 크기 상위권에서 뽑았으므로 "무거운 아카이브"
쪽 조건이다. 작은 문서로 채운 코퍼스라면 배치 이득이 더 커진다(기동 비용의 비중이
커지므로).

### 반복 측정 방법

```bash
R=target/release/rhwp.exe
rep() { for k in 1 2 3; do
  s=$(date +%s%N); eval "$@" >/dev/null 2>&1; e=$(date +%s%N)
  echo $(( (e-s)/1000000 ))
done; }
rep 'for i in $(seq 20); do "$R" info --json "$DOC"; done'
```

표에 run1/run2/run3 을 그대로 적었다. **평균만 적고 분산을 감추지 않는다.**
워밍업으로 각 대상을 3회 미리 실행해 페이지 캐시를 데운 뒤 측정했다.

---

## 2. 프로세스 생성 비용

`rhwp` 를 20회 부르고 총시간을 잰다. `--version` 은 파일을 열지 않으므로
**순수 기동 비용의 바닥값**이다.

> **조건**: 이 절은 **같은 문서 하나를 20번** 처리한다(개별 루프도, batch stdin 20줄도).
> 문서 다양성·디스크 I/O 변수를 없애고 **기동 비용만** 분리하기 위해서다.
> 서로 다른 파일 20개를 쓰는 실제 아카이브 측정은 §4·§5 다.

| 워크로드 | run1 | run2 | run3 | 1회당 |
|---|---:|---:|---:|---:|
| `rhwp --version` ×20 | 1,449 | 1,477 | 1,507 | **73.9 ms** |
| `info --json` 소형 ×20 | 1,538 | 1,524 | 1,537 | **76.6 ms** |
| `info --json` 대형 ×20 | 4,129 | 4,077 | 4,086 | **204.9 ms** |

같은 20건을 `batch` 로 (stdin 에 같은 경로 20줄):

| 워크로드 | run1 | run2 | run3 | 내부 처리(stderr 보고) |
|---|---:|---:|---:|---:|
| `batch info --threads 1` 소형 ×20 | 187 | 183 | 189 | 101 ms |
| `batch info --threads 1` 대형 ×20 | 2,698 | 2,673 | 2,681 | 2,681 ms |
| `batch info` (t=32) 대형 ×20 | 884 | 890 | 895 | 805 ms |

### 비용 분해 — 이 표의 핵심

대형 문서에서 두 경로의 차이가 그대로 기동 비용이다.

```
개별 프로세스 1회 = 204.9 ms
batch 내부 1건    = 2,681 / 20 = 134.1 ms   (파싱+봉투 직렬화)
차이              =  70.8 ms   ← 기동 비용 (--version 바닥값 73.9 ms 와 일치)
```

**같은 20건인데 개별 4,086 ms vs batch(t=1) 2,681 ms.** 병렬을 전혀 쓰지 않아도
프로세스를 다시 띄우지 않는 것만으로 **1.52배**다. 같은 표의 t=32 행(890 ms)과
비교하면 **4.59배**다 — 다만 이 20건은 같은 파일 20줄이므로 파일 다양성이 있는
코퍼스 측정은 §4·§5 를 본다.

소형 문서에서는 파싱이 5 ms 남짓이라 **비용의 94%가 기동**이다
(76.6 ms 중 73.9 ms). 작은 문서를 많이 처리할수록 batch 이득이 커지는 이유다.

---

## 3. 봉투 크기 — 컨텍스트 예산

`--json` stdout 의 바이트 수(개행 포함). `wc -c` 로 측정.

```bash
"$R" digest --json "$DOC" 2>/dev/null | wc -c
```

| 명령 | 소형 1쪽 | 중형 27쪽 | 대형 393쪽 |
|---|---:|---:|---:|
| `info --json` | 350 | 576 | **636** |
| `digest --json` | 2,003 | 2,884 | **1,375** |
| `digest --json --sections` | — | 13,286 | 2,692 |
| `export-structure --json` | 968 | 26,982 | **284,892** |
| `fields --json` | 167 | 968 | 47,738 |
| `export-text --json` | 1,913 | 46,714 | **645,108** |
| `export-tables --json` | 3,862 | 69,622 | **759,030** |
| `extract-data --json` | — | 7,352 | 117,869 |

### 읽어야 할 세 가지

**① 대형 문서에서 `digest` 는 `export-text` 의 1/469 다.**
1,375 B vs 645,108 B. 작업이 "이 문서가 뭐냐"라면 469배를 낼 이유가 없다.
`export-tables` 는 552배다.

**② `digest` 는 문서가 커도 거의 커지지 않는다.** 1쪽 2,003 B → 393쪽 1,375 B.
발췌 상한(`--max-chars` 기본 2000)이 있고 메타·개요 상위 노드만 담기 때문이다.
**대형 문서에서 오히려 작다** — 1쪽 문서는 본문 전체가 발췌에 들어가서다.

**③ 그래서 소형 문서에서는 `digest` 가 손해다.** 1쪽 문서: `digest` 2,003 B >
`export-text` 1,913 B. 1~2쪽이면 본문을 바로 부르는 편이 짧다.

### 좁혀 부르기의 효과 (대형 393쪽)

| 호출 | 봉투 | 전체 대비 |
|---|---:|---:|
| `export-text --json` | 645,108 B | 1× |
| `export-text --json -p 41` (한 쪽) | **324 B** | 1/1,991 |
| `digest --json --pages 41..41` | **398 B** | 1/1,621 |
| `search --json "문서"` (상한 없음) | 611,818 B | 0.95× |
| `search --json --max-matches 20 "문서"` | **8,177 B** | 1/75 |

`search` 는 상한을 안 걸면 본문만큼 크다. **`--max-matches` 는 선택이 아니라 기본
습관이어야 한다.** 절단되면 봉투에 `truncated:true`·`omittedCount` 가 남으므로
정보가 조용히 사라지지 않는다.

### `--max-chars` 는 컨텍스트 절약 수단이 아니다

`export-text --json --max-chars N` (대형 393쪽):

| N | 봉투 |
|---:|---:|
| 1 | **22,904 B** |
| 500 | 24,044 B |
| 2,000 | 27,042 B |
| 20,000 | 69,547 B |
| 200,000 | 481,563 B |
| 무제한 | 645,108 B |

**바닥값 22,904 B 는 지워지지 않는다.** 상한을 넘은 쪽도 `{"page":N,"text":"",
"truncated":true,"omittedCount":…}` 객체로 남기 때문이다(쪽 주소를 잃지 않으려는
설계). 393쪽 문서면 그 골격만 22.9 KB 다.

컨텍스트를 실제로 줄이려면 **쪽을 좁혀야 한다** — `-p N`(324 B) 또는
`digest --pages a..b`(398 B).

### 문서와 무관한 봉투 (온보딩 비용)

| 명령 | 크기 |
|---|---:|
| `capabilities` | 16,723 B (61개 명령, 그중 `json:true` 31개) |
| `capabilities --mcp` | 47,022 B (무상태 도구 39개) |
| `export-capabilities-schema` | 24,411 B |
| `export-ir-schema` | 44,119 B |
| `export-provenance-map --json` | 11,885 B |

`capabilities` 한 번(16.7 KB)이면 명령 표면·플래그·종료 코드·JSON 계약을 전부
얻는다. help 파싱이나 플래그 추측보다 싸고 정확하다.

---

## 4. 배치 이득 — N개 문서에 같은 명령

코퍼스 20건(84 MB)·10건. 개별은 `while read f; do rhwp <cmd> "$f"; done`,
배치는 `rhwp batch <cmd> --json < corpus.txt`. 배치는 `--threads` 기본값(=32).

```bash
s=$(date +%s%N); while read f; do "$R" info --json "$f" >/dev/null 2>&1; done < corpus20.txt; e=$(date +%s%N)
s=$(date +%s%N); "$R" batch info --json < corpus20.txt >/dev/null 2>&1;              e=$(date +%s%N)
```

| 명령 | N | 개별 (ms) | batch (ms) | 이득 |
|---|---:|---:|---:|---:|
| `info` | 10 | 1,047 | 184 | **5.7×** |
| `info` | 20 | 2,347 / 2,298 / 2,309 | 296 / 306 / 321 | **7.5×** |
| `export-text` | 10 | 2,988 | 548 | **5.5×** |
| `export-text` | 20 | 5,443 / 5,309 | 715 / 757 | **7.3×** |

N 이 늘수록 이득이 커진다(10→20 에서 5.6배→7.4배). 프로세스 기동은 N 에 비례해
쌓이지만 batch 의 고정비는 하나뿐이고, 파일 간 병렬도 N 이 커야 채워지기 때문이다.

### batch 레코드 vs 단건 봉투 — 크기는 같고 모양이 다르다

같은 중형 문서(27쪽):

| | 크기 | 모양 |
|---|---:|---|
| `export-text --json` 단건 | 46,714 B | `pages[]` = 쪽별 객체 27개 |
| `batch export-text` 레코드 1줄 | 46,137 B | `text` = 문서 전체 한 덩어리 |
| `info --json` 단건 | 576 B | — |
| `batch info` 레코드 1줄 | 576 B | 동일 |

`batch export-text` 레코드 필드는
`pageCount·schemaVersion·source·text·untrustedContent·untrustedFields` 다 —
**`pages[]` 가 없다.** 쪽 주소로 인용해야 하는 RAG·감사 작업이면 batch 로 선별한 뒤
대상만 단건으로 다시 받아야 한다. 크기 이득이 있어서가 아니라(거의 같다)
**주소가 필요해서**다.

---

## 5. `--threads` 효과

코퍼스 20건(84 MB), 32코어 머신. 벽시계와 batch 자체가 stderr 로 보고하는
내부 처리 시간을 함께 적는다.

### `batch export-text`

| threads | 내부 (ms) | 벽시계 (ms) | t=1 대비 |
|---:|---:|---:|---:|
| 1 | 3,563 | 3,656 | 1.00× |
| 2 | 2,003 | 2,101 | 1.78× |
| 4 | 1,213 | 1,303 | 2.94× |
| 8 | 816 | 908 | 4.37× |
| 16 | 618 | 709 | 5.76× |
| 32 | 556 | 647 | 6.41× |

### `batch info`

| threads | 내부 (ms) | 벽시계 (ms) | t=1 대비 |
|---:|---:|---:|---:|
| 1 | 833 | 924 | 1.00× |
| 2 | 475 | 568 | 1.75× |
| 4 | 313 | 407 | 2.66× |
| 8 | 259 | 355 | 3.22× |
| 16 | 230 | 345 | 3.62× |
| 32 | 190 | 285 | 4.38× |

### 읽는 법

- **8스레드까지가 값싼 구간이다.** export-text 는 4.37배, info 는 3.22배가 8에서
  나온다. 8→32 에서 export-text 는 4.37→6.41(+47%), info 는 3.22→4.38(+36%).
- **`info` 가 먼저 포화한다.** 건당 작업이 작아 조율·I/O 비중이 커서다.
- **N=20 은 32스레드를 채우지 못한다.** 파일이 20개인데 스레드가 32개면 절반이
  논다. 이 표의 t=32 값은 "32코어의 상한"이 아니라 "N=20 에서의 값"이다.
- **기본값(코어 수)을 바꿀 이유는 거의 없다.** 올려서 손해는 없고, 내리는 것은
  다른 작업과 CPU 를 나눠야 할 때뿐이다.

---

## 6. MCP — 세션 vs 무상태

Python 3.11 로 `rhwp mcp-serve` 를 stdio 로 몰아 측정
(스크립트: `subprocess.Popen` + 줄 단위 JSON-RPC, `time.perf_counter`).

### 서버 기동·핸드셰이크

| 항목 | run1 | run2 | run3 |
|---|---:|---:|---:|
| `Popen` → `initialize` 응답 | 51 ms | 49 ms | 49 ms |
| `tools/list` 왕복 | 0.9 ms | 0.8 ms | 0.9 ms |
| `tools/list` 응답 크기 | 40,503 B | 40,503 B | 40,503 B |

도구 51개(무상태 39 + 세션 12). **서버 기동이 CLI 한 번보다 싸다**(49 ms vs 73.9 ms) —
셸 fork 가 없어서다. 프로세스 하나를 계속 살려두는 비용은 실질적으로 0 이다.

`resources/list` 는 4건 1,161 B(`rhwp://capabilities/mcp`, `rhwp://docs/llms.txt`,
`agent_knowledge_map.md`, `agent_troubleshooting_guide.md`). 파일 접근 없이 문서를
받을 수 있는 경로다.

### 세션 (대형 393쪽 문서)

| 단계 | 시간 |
|---|---:|
| `hwp_open` | **125.0 ms** |
| `hwp_doc_text` ×20쪽 | **13.5 ms** (쪽당 **0.68 ms**) |
| `hwp_close` | 9.6 ms |
| 세션 전체 (initialize→close) | 346.3 ms |
| (대조) 같은 서버에서 무상태 `hwp_info` 1회 | **189.3 ms** |

### 같은 일을 CLI 로 하면

대형 문서의 0~19쪽을 읽는 작업:

| 경로 | 시간 | 배수 |
|---|---:|---:|
| `export-text --json -p N` ×20 (개별 프로세스) | **4,419 ms** (220 ms/회) | 1× |
| MCP 세션 (open 125 + 20쪽 13.5 + close 9.6) | **148.1 ms** | **1/29.8** |

**재파싱을 20번 하느냐 한 번 하느냐의 차이다.** 대형 문서를 여러 번 볼 것이
확실하면 세션 외의 선택지는 없다.

### MCP 봉투 팽창 — 같은 내용을 두 번 담는다

`tools/call` 응답은 봉투를 `content[0].text`(이스케이프된 JSON 문자열)와
`structuredContent`(객체) **양쪽에** 싣는다. JSON-RPC 한 줄의 UTF-8 바이트 수:

| 도구 | 문서 | `content[0].text` | 전체 응답 | 배율 |
|---|---|---:|---:|---:|
| `hwp_info` | 소형 | 349 | 857 | 2.46× |
| `hwp_digest` | 소형 | 2,002 | 4,219 | 2.11× |
| `hwp_export_text` | 소형 | 1,912 | 4,033 | 2.11× |
| `hwp_export_tables` | 소형 | 3,861 | 8,409 | 2.18× |
| `hwp_info` | 대형 | 635 | 1,453 | 2.29× |
| `hwp_digest` | 대형 | 1,374 | 2,922 | 2.13× |
| `hwp_export_structure` | 대형 | 284,891 | 583,901 | 2.05× |
| `hwp_export_text` | 대형 | 645,107 | **1,309,290** | 2.03× |
| `hwp_export_tables` | 대형 | 759,029 | **1,604,384** | 2.11× |

`content[0].text` 는 CLI stdout 에서 개행 1바이트를 뺀 값과 정확히 일치한다
(645,107 = 645,108 − 1). **봉투 자체는 같고, 전송 프레임이 2배다.**

주의해서 읽어라: 모델 컨텍스트에 무엇이 들어가는지는 **호스트가 정한다.**
`content[0].text` 만 넣는 호스트라면 모델 비용은 CLI 와 같고, 둘 다 넣는 호스트면
2배다. 확실한 것은 **파이프·호스트 메모리에 오가는 바이트가 2배**라는 것뿐이다.

### 프로필로 도구 표면 줄이기

`capabilities --mcp --profile <직무>` 의 크기(= 그 프로필로 띄운 서버의 `tools/list`
범위):

| 프로필 | 무상태 도구 수 | 선언 크기 |
|---|---:|---:|
| (필터 없음) | 39 | 47,022 B |
| 경영보고 | 6 | 6,378 B |
| 행정서식 | 8 | 10,237 B |
| 데이터분석 | 6 | 6,931 B |
| 콘텐츠제작 | 6 | 5,939 B |
| 아카이브검색 | 7 | 8,257 B |
| 품질검증 | 6 | 5,970 B |
| 개발통합 | 39 | 47,490 B |

직무 프로필은 도구 목록 컨텍스트를 **약 1/6** 로 줄인다. 세션 축도 이름 단위로
걸리므로(`아카이브검색` 은 조회 8종만) 표면 축소가 곧 권한 축소이기도 하다.

---

## 7. 계획 실행기 `run` vs 편집 체이닝

같은 결과(표 0 의 (2,1)·(2,3) 두 칸 채우기)를 두 방식으로. 소형 문서, 5회 반복.

| 방식 | 총시간 | 1회당 | 프로세스 | 중간 파일 |
|---|---:|---:|---:|---:|
| `run` 계획 1개 (2 step) | 448 ms | **89.6 ms** | 1 | 0 |
| `edit set-cell` ×2 (`-o` 체이닝) | 827 ms | **165.4 ms** | 2 | 1 |

**1.85배.** 기동 1회분(73.9 ms)이 그대로 차이다 — 파싱도 1회 아낀다.

봉투 크기:

| | 크기 |
|---|---:|
| `run --json` | 501 B |
| `run --json --dry-run` | 480 B (`preview[]` 에 `currentText`→`newText`) |
| `edit set-cell --json` | 375 B |

step 이 늘수록 격차가 벌어진다(체이닝은 step 마다 기동+파싱+저장, `run` 은 전체에
한 번). 다만 **주된 이유는 속도가 아니라 원자성과 선검증**이다
([entrypoint_decision.md](entrypoint_decision.md) §3.5).

---

## 8. 철회된 언어 바인딩 실측 이력

이 절은 #4655 이전 Python 바인딩의 측정값을 보존한 historical record다. 현재
지원 진입로의 성능 근거나 재현 절차로 사용하지 않는다.

Python 바인딩을 소스에서(`PYTHONPATH=bindings/python/src`,
`RHWP_BIN=target/release/rhwp.exe`) 직접 실행.

| 워크로드 | 총시간 | 1회당 |
|---|---:|---:|
| `rhwp.info()` 소형 ×20 | 1,117 ms | **55.8 ms** |
| `rhwp.info()` 대형 ×20 | 3,811 ms | **190.6 ms** |
| `rhwp.open()` 대형 (세션 열기) | — | **178 ms** |
| `doc.text(page=i)` ×20쪽 | 14.8 ms | **0.74 ms/쪽** |

### 읽는 법

- **바인딩은 CLI 보다 싸지 않다.** 대형 문서 190.6 ms vs CLI 루프 204.9 ms — 차이는
  Git Bash 의 fork 오버헤드이지 바인딩의 최적화가 아니다. 바인딩도 호출마다
  프로세스를 띄운다.
- **세션 층의 값은 그대로 온다.** `doc.text()` 0.74 ms/쪽 vs 원시 MCP 세션
  0.68 ms/쪽. 차이(0.06 ms)는 Python 왕복이다. 바인딩이 계약을 얇게 재포장한다는
  주장이 성능에서도 확인된다.
- 절약은 **층 선택**에서 나오지 언어 선택에서 나오지 않는다.

Node 바인딩은 **미측정**이다(§10).

---

## 9. 한 장 요약 — 어떤 절약이 큰가

| 절약 수단 | 배수 | 조건 |
|---|---:|---|
| `export-text` 대신 `digest` (대형) | **469×** 컨텍스트 | 393쪽, 봉투 바이트 |
| `export-text` 전체 대신 `-p N` | **1,991×** 컨텍스트 | 393쪽 → 한 쪽 |
| `search` 에 `--max-matches 20` | **75×** 컨텍스트 | 393쪽, 흔한 검색어 |
| CLI 20쪽 읽기 → MCP 세션 | **29.8×** 시간 | 393쪽 문서 20쪽 |
| 개별 20건 → `batch` | **7.5×** 시간 | 84 MB 코퍼스 |
| `--threads` 1 → 8 | **4.4×** 시간 | batch export-text, N=20 |
| `--threads` 8 → 32 | 1.47× 시간 | 같은 조건 (수확 체감) |
| 편집 체이닝 → `run` 계획 | 1.85× 시간 | 2 step |
| MCP 프로필 지정 | **6~8×** 도구 목록 컨텍스트 | 39개 → 6~8개 |

**컨텍스트 절약이 시간 절약보다 자릿수가 크다.** 에이전트 파이프라인을 최적화할
때 첫 손질은 언제나 "무엇을 부르는가"이고, "어떻게 부르는가"는 그다음이다.

---

## 10. 미측정 — 그러므로 주장하지 않는다

| 축 | 왜 못 쟀나 |
|---|---|
| **WASM 표면 전부** | 구현체가 없다. 모듈 크기·인스턴스화·파싱·경계 복사 전부 미측정. [surface_spec.md](surface_spec.md) §8 과 같은 상태다 |
| **Node 바인딩 런타임** | `dist/` 빌드(tsup)와 `node_modules` 가 없고, 이 조사에서 빌드를 하지 않았다. 구조는 Python 과 같은 서브프로세스 설계이므로 1층 비용은 비슷할 것으로 **예상되나 재지 않았다** |
| **`export-png`** | 이 바이너리는 `native-skia` feature 없이 빌드됐다 — exit 2 로 끝난다 |
| **메모리 상주 크기** | 393쪽 문서의 IR 이 몇 바이트인지 재지 않았다. 세션을 몇 개까지 열 수 있는지 판단할 근거가 없다 |
| **세션 다중 문서** | 동시에 여러 `docId` 를 열었을 때의 비용·상한을 재지 않았다 |
| **exit 4 (`--verify-pages` 쪽수 불일치)** | 시도한 10개 문서에서 재현되지 않았다. 계약은 코드·`capabilities` 에 선언돼 있다 |
| **컨테이너·서버 환경** | 이 PC 한 대에서만 쟀다. AV 가 없는 리눅스 서버라면 기동 바닥값이 낮을 것이다 |
| **대규모 N (수백~수천 건)** | N=10·20 만 쟀다. 더 큰 N 의 지속 처리율은 [cli_json_pipeline_guide.md](../../manual/cli_json_pipeline_guide.md) 의 별도 실측(271·1,350건)을 참조하되, 그것은 다른 조건의 값이다 |

**측정하지 않은 항목으로 설계 결정을 내리지 않는다.** 이 표의 항목이 판단에
필요해지면 먼저 재고 이 문서를 갱신한다.

---

## 인접 문서

- [entrypoint_decision.md](entrypoint_decision.md) — 이 숫자로 내리는 선택
- [failure_dictionary.md](failure_dictionary.md) — 경로별 실패 증상
- [surface_spec.md](surface_spec.md) — WASM 표면 설계(canonical)
- [envelope_parity.md](envelope_parity.md) — 봉투 동등성 계약
- [README.md](README.md) — 축 지도
- [cli_commands.md](../../manual/cli_commands.md) — 명령·플래그 권위
- [cli_json_pipeline_guide.md](../../manual/cli_json_pipeline_guide.md) — 다른 조건의 배치 실측
- [mcp_integration_guide.md](../../manual/mcp_integration_guide.md) — MCP 두 경로
- 공식 Python·Node 바인딩 철회: [#4655](https://github.com/edwardkim/rhwp/issues/4655)
- 이슈 [#3869](https://github.com/edwardkim/rhwp/issues/3869)
