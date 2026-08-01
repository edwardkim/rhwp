# 이주 가이드 — 기존 파이썬 HWP 도구에서 옮겨오기

기존 생태계의 도구들과 `rhwp` 는 **접근 방식이 다르다.** API 이름을 하나씩 대응
시키는 대신, 무엇이 왜 다른지부터 정리한다. 그래야 옮긴 코드가 제대로 돈다.

---

## 1. 근본 차이 셋

### ① COM 자동화가 아니다

한글 프로그램을 띄워 조작하는 방식(win32com)은 **한컴 오피스가 설치된 윈도우**에서만
돈다. `rhwp` 는 문서 포맷을 직접 파싱·직렬화하므로 리눅스·macOS·컨테이너에서 돌고,
한글 설치가 필요 없다.

| | COM 자동화 | rhwp |
|---|---|---|
| 플랫폼 | 윈도우 + 한글 설치 | 어디서나 |
| 서버 배포 | 라이선스·GUI 세션 필요 | 바이너리 하나 |
| 실패 양상 | GUI 대화상자로 멈춤 | 종료 코드 + 봉투 |
| 병렬 처리 | 인스턴스 충돌 | 프로세스만큼 |

### ② 읽기 전용이 아니다

평문 추출만 하는 라이브러리와 달리 **편집·저장·렌더**까지 한다. 다만 그만큼
"저장본이 진짜 맞나"가 중요해지므로 `verify` 규약이 있다.

### ③ 판정이 데이터다

예외를 던지는 대신 봉투에 판정을 담는다. 자세한 것은 [§4](#4-예외-처리-바꾸기).

---

## 2. 문서 읽기

### 평문 추출

```python
# 기존: 문서 전체를 한 문자열로
text = some_lib.extract_text("문서.hwp")

# rhwp: 쪽별로 나뉘어 온다 — 주소가 보존된다
result = rhwp.export_text("문서.hwp")
text = "\n".join(p.text for p in result.pages)          # 통합하고 싶으면
first_page = result.pages[0].text                        # 쪽 단위가 필요하면
```

**왜 쪽별인가**: RAG·인용 검증에서 "몇 쪽"에 답하려면 주소가 필요하다. 통합은
호출자가 언제든 할 수 있지만, 잃어버린 주소는 복구할 수 없다.

### 문서 정보

```python
meta = rhwp.info("문서.hwp")
meta.page_count      # 쪽수
meta.format          # "hwp5" | "hwpx" | "hwp3" | "hml"
meta.sections        # 구역 수
meta.fonts           # 글꼴 목록
```

### 표 읽기

```python
for table in rhwp.export_tables("문서.hwp").tables:
    for cell in table.cells:
        print(table.index, cell.row, cell.col, cell.text)
```

**병합 셀 주의**: 병합된 셀은 **좌상단 좌표 하나로만** 나온다. 덮인 좌표는 목록에
없다. 격자를 만들 때 빈 칸을 기본값으로 채워야 한다
([요리책 §4](COOKBOOK.md#4-표를-데이터셋으로)).

---

## 3. 문서 편집

### 누름틀 채우기

```python
# COM 방식은 대략 이런 흐름이었다
# hwp.PutFieldText("성명", "홍길동")
# hwp.Save()

# rhwp — 한 번에, 검증까지
result = rhwp.fill_fields("서식.hwp", {"성명": "홍길동"}, out="제출본.hwp", verify=True)
assert result.verify.identical
```

동명 누름틀은 `#순번` 으로 지정한다: `{"성명#0": "갑", "성명#1": "을"}`.

### 여러 편집을 한 번에

COM 은 열어 두고 하나씩 조작한 뒤 저장했다. `rhwp` 에는 두 가지 대응이 있다.

**세션(2층)** — COM 의 사용 감각에 가깝다.

```python
with rhwp.open("서식.hwp") as doc:
    doc.fill_fields({"성명": "홍길동"})
    doc.replace_text("2025년", "2026년")
    doc.save("제출본.hwp", verify=True)
```

**계획(3층)** — 더 안전하다. 하나라도 불가능하면 **아무것도 저장하지 않는다.**

```python
plan = (rhwp.Plan("서식.hwp", "제출본.hwp")
        .fill_fields({"성명": "홍길동"})
        .replace_text("2025년", "2026년")
        .verify())
if plan.check().ok:      # 디스크 무변경 검사
    plan.run()
```

COM 에서 "중간에 실패해 반쯤 편집된 문서가 남는" 문제를 겪었다면 3층이 답이다.

---

## 4. 예외 처리 바꾸기

가장 흔한 이주 실수다.

```python
# 기존 감각 — 모든 실패가 예외
try:
    convert("a.hwp", "b.hwpx")
except Exception:
    handle_failure()
```

`rhwp` 는 **고장과 판정을 가른다.**

```python
try:
    result = rhwp.export_hwpx("a.hwp", out="b.hwpx", verify=True)
except rhwp.RhwpRuntimeError:
    ...   # 못 읽었다·못 썼다 — 진짜 고장
except rhwp.UsageError:
    ...   # 인자가 틀렸다 — 우리 코드 버그

# 변환은 됐는데 내용이 달라졌다 → 예외가 아니라 판정
if not result.verify.identical:
    print(f"차이 {result.verify.diff_count}건")
```

**왜**: 검증 실패를 예외로 만들면 호출자가 "고장"으로 다루고, 정작 봉투에 담긴
판정 근거를 읽지 않는다. 예외가 편하면 `raise_on_verdict=True` 로 명시한다.

| 기존 | rhwp |
|---|---|
| `except Exception` | `except rhwp.RhwpError` (모든 rhwp 예외의 기반) |
| 파일 없음 | `RhwpRuntimeError` |
| 잘못된 인자 | `UsageError` (`.suggestion` 에 교정 힌트) |
| 변환 손실 | **예외 아님** — `result.verify.identical` |

---

## 5. 대량 처리

```python
# 기존: 파이썬 루프
for path in paths:
    text = some_lib.extract_text(path)      # 실패 하나가 루프를 멈춘다

# rhwp: 배치 — 부분 실패를 잃지 않는다
for record in rhwp.batch("export-text", paths):
    if "error" in record:
        log_failure(record["source"], record["error"])
        continue
    handle(record["text"])
```

**부분 실패도 실패지만 성공분은 남는다.** 수백 건 중 하나가 손상됐다고 나머지를
버릴 이유가 없다.

---

## 6. 없는 기능·다른 기능

### `rhwp` 에 없는 것

| 기존 기능 | 대안 |
|---|---|
| 한글 GUI 조작(인쇄 대화상자 등) | 없음 — 포맷 수준 작업만 |
| 매크로 실행 | 없음 |
| 실시간 편집 UI | `rhwp-studio`(별도) |

### `rhwp` 에만 있는 것

| 기능 | 설명 |
|---|---|
| `verify` | 저장본이 의도한 문서인지 자기검증 |
| `changed_pages` | 편집이 바꾼 쪽 지정 → 그 쪽만 렌더해 확인 |
| `Plan` | 원자적 다단 편집 (전부 아니면 전무) |
| `ir_diff` | 두 문서의 구조 차이를 범주별로 |
| `digest` | 주소를 보존한 RAG 청킹 |
| `export-ir-schema` | 문서 모델의 기계 판독 스키마 |
| MCP 서버 | 에이전트가 바로 붙는 도구 표면 |

---

## 7. 이주 점검표

- [ ] 평문 추출이 **쪽별**로 온다는 것을 반영했는가
- [ ] 표의 **병합 셀**이 좌상단 좌표로만 나온다는 것을 처리했는가
- [ ] `except Exception` 을 **고장/판정**으로 갈랐는가
- [ ] 저장에 `verify=True` 를 붙였는가
- [ ] `verify` 의 `None`(검증 안 함)과 실패를 구분했는가
- [ ] `changed_pages` 의 `None`(모름)과 `[]`(없음)을 구분했는가
- [ ] 세션을 `with` 로 감쌌는가 (안 그러면 프로세스가 남는다)
- [ ] 대량 처리에서 **부분 실패**를 버리지 않는가
- [ ] 표·쪽 좌표를 **추측하지 않고** 조회로 확인하는가

---

## 8. 성능 감각

| 작업 | 대략 |
|---|---|
| 1층 호출 하나 | 프로세스 기동(수십 ms) + 문서 파싱 |
| 세션 열기 | 파싱 한 번, 이후 호출은 재파싱 없음 |
| 배치 | 프로세스 하나가 목록 전체를 처리 |

**같은 문서에 3회 이상 접근하면 세션이 유리하다.** 서로 다른 문서 수백 개는
배치가 유리하다.

```python
# 느림 — 같은 문서를 세 번 파싱
rhwp.info(p); rhwp.fields(p); rhwp.export_tables(p)

# 빠름 — 한 번 파싱
with rhwp.open(p) as doc:
    doc.info(); doc.fields(); doc.tables()
```

---

## 9. 라이선스

`rhwp` 와 이 바인딩은 **MIT** 다. 상용 제품에 넣을 수 있고, 소스 공개 의무가 없다.

---

## 막히면

[문제 해결](TROUBLESHOOTING.md)에 증상별 처방이 있다. 그래도 안 되면 이슈를 열 때
아래를 붙여 달라.

```python
import rhwp
print(rhwp.__version__, rhwp.find_binary(), rhwp.capabilities().version)
```
