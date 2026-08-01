# rhwp — 파이썬 바인딩

HWP·HWPX·HWP3·HML 문서를 읽고 편집·렌더링하는 [rhwp](https://github.com/edwardkim/rhwp) 엔진의 파이썬 바인딩입니다.

> **바인딩은 새 표면이 아니라 기존 계약의 재포장입니다.**
> CLI `--json` 봉투와 `mcp-serve` 세션 도구가 이미 증명한 계약 위에만 서고,
> 파이썬 쪽에서 판정 로직을 새로 만들지 않습니다. rhwp 본체에 명령이 늘면
> 바인딩은 자동으로 따라옵니다 — 계약 패리티 가드가 뒤처짐을 CI 에서 잡습니다.

## 설치

```bash
pip install rhwp
```

`rhwp` 실행 파일이 필요합니다. 탐색 순서는 다음과 같습니다.

1. 환경변수 `RHWP_BIN` — 로컬 빌드를 가리킬 때
2. 패키지 동봉 (`rhwp/_bin/`)
3. `PATH`

```bash
export RHWP_BIN=/path/to/rhwp        # 선택
python -c "import rhwp; print(rhwp.find_binary())"
```

환경변수를 **줬는데 쓸 수 없으면** 조용히 다음 경로로 넘어가지 않고 즉시 실패합니다.
사용자가 그 바이너리를 쓰고 있다고 믿는데 다른 게 실행되면 디버깅이 불가능하기 때문입니다.

## 3층 구조

rhwp 의 에이전트 표면이 그대로 파이썬 API 가 됩니다.

| 층 | 무엇 | 언제 |
|---|---|---|
| 1층 무상태 | `rhwp.info(path)` 등 | 호출 하나 = 작업 하나 |
| 2층 세션 | `with rhwp.open(path) as doc:` | 같은 문서를 반복해서 만질 때 |
| 3층 계획 | `rhwp.Plan(...).run()` | 여러 편집을 원자적으로, 검증까지 |

### 1층 — 무상태

```python
import rhwp

meta = rhwp.info("보고서.hwp")
print(meta.page_count, meta.format)      # 속성 (snake_case)
print(meta["pageCount"])                 # 원문 키도 그대로

# 주소가 붙은 검색 — 매치마다 (구역·문단·쪽·문자 오프셋)
for m in rhwp.search("보고서.hwp", "예산").matches:
    print(f"{m.page}쪽: {m.snippet}")

# 표를 셀 좌표와 함께
for table in rhwp.export_tables("양식.hwpx").tables:
    print(table.index, len(table.cells))
```

### 2층 — 세션

문서를 한 번 열어 두고 여러 번 만집니다. 호출마다 재파싱하지 않으므로 대형 문서에서 빠릅니다.

```python
with rhwp.open("서식.hwp") as doc:
    doc.fill_fields({"성명": "홍길동", "부서": "기획팀"})
    doc.replace_text("2025년", "2026년")

    result = doc.save("제출본.hwp", verify=True)
    assert result.verify.identical          # 저장본이 의도한 문서인가

    # 바뀐 쪽만 눈으로 확인 — 상수 비용
    for page in result.changed_pages or []:
        doc.render_page(page, f"확인_{page}.svg")
```

`with` 블록을 벗어나면 핸들이 닫히고 자식 프로세스가 정리됩니다. 예외로 빠져나가도 마찬가지입니다.

### 3층 — 계획

여러 편집을 **의도**로 선언하면 rhwp 가 안전을 보장합니다.
정적 선검증(실행 0) → 원자 실행(인메모리) → 단언 통과 시에만 단 한 번 저장.

```python
plan = (
    rhwp.Plan("서식.hwp", "제출본.hwp")
    .fill_fields({"성명": "홍길동"})
    .replace_text("2025년", "2026년")
    .set_checkbox(1)
    .verify()
)

preview = plan.check()          # 디스크 무변경 — 실행 전 검사
if not preview.ok:
    print(preview.describe_violations())
else:
    journal = plan.run()
    assert journal.verify.identical
```

중간 step 이 실패해도 **반쪽 편집 문서가 남지 않습니다** — 전 step 이 메모리에서 통과해야 저장합니다.

## 판정 vs 고장

이 바인딩의 핵심 규약입니다.

```python
# 판정 실패는 예외가 아니다 — 도구는 정상 동작했고, 문서에 대한 단언이 실패한 것
result = rhwp.export_hwpx("원본.hwp", out="변환본.hwpx", verify=True)
if not result.verify.identical:
    print(f"차이 {result.verify.diff_count}건")   # 봉투를 읽어 판단

# 예외를 원하면 명시
rhwp.export_hwpx("원본.hwp", verify=True, raise_on_verdict=True)   # VerdictFailed
```

| 상황 | 종료 코드 | 파이썬 |
|---|---|---|
| 성공 | 0 | 정상 반환 |
| 읽기·파싱·렌더·쓰기 실패 | 1 | `RhwpRuntimeError` |
| 인자가 틀림 (**우리 쪽 버그**) | 2 | `UsageError` |
| 검증 단언 실패 | 3 | **반환값의 판정 필드** |
| 페이지 수 불일치 | 4 | **반환값의 판정 필드** |

exit 3/4 를 기본으로 예외로 만들면 호출자가 `try/except` 로 "고장"처럼 다루게 되고,
정작 봉투에 담긴 판정 근거(`diff_count`·`status`)를 읽지 않게 됩니다.

## "모름"과 "없음"의 구분

```python
result = rhwp.fill_fields("서식.hwp", {"성명": "값"}, out="산출.hwp")

result.changed_pages    # None = 확정 불가 / [] = 바뀐 쪽 없음 / [0,2] = 그 쪽들
result.verify           # None = 검증 안 함 (실패가 아님)
```

부분 목록은 침묵보다 나쁩니다 — 빠뜨린 항목이 있는 목록은 거짓 통과를 만듭니다.
그래서 rhwp 는 확정할 수 없으면 `null` 을 내고, 바인딩은 그걸 `None` 으로 전합니다.

## 오타는 조용히 넘어가지 않습니다

```python
meta = rhwp.info("보고서.hwp")
meta.page_conut          # AttributeError: 있는 필드를 함께 알려준다
```

없는 필드가 `None` 이 되면, 필드 이름을 잘못 쓴 코드가 "값이 없네"로 흘러가
가장 찾기 어려운 버그가 됩니다.

## 대량 처리

```python
records = rhwp.batch("export-text", ["a.hwp", "b.hwp", "c.hwp"])
for r in records:
    if "error" in r:
        print(f"실패: {r['source']} — {r['error']}")
    else:
        print(f"{r['source']}: {r['pageCount']}쪽")
```

부분 실패도 실패지만 **성공분은 스트림에 남습니다.** 실패 하나로 전체를 버리지 마세요.

## 개발

```bash
cd bindings/python
pip install -e ".[dev]"

pytest tests/ -q                          # 전체
pytest tests/ -q -m "not integration"     # 바이너리 없이 단위만
```

단위 테스트는 rhwp 빌드 없이 돕니다 — 탐색·변환·예외 매핑·계획 직렬화는 순수 로직입니다.
실제 문서를 만지는 통합 테스트만 `@pytest.mark.integration` 으로 격리돼 있습니다.

### 계약 패리티 가드

`test_binding_covers_every_agent_value_command` 가 `rhwp capabilities` 선언과
파이썬 API 를 대조합니다. rhwp 에 명령이 늘었는데 바인딩이 뒤처지면 **CI 에서 실패**합니다.
수기 목록을 두지 않는 것이 이 바인딩이 뒤처지지 않는 이유입니다.

## 라이선스

MIT — rhwp 본체와 동일합니다.
