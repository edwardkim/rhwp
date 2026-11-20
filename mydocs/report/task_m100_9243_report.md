# 검증 원본과 기준 PDF 수집 범위 조사

Issue: #9243 · 부모 이슈 #9242

기준 커밋은 `411a9f944cd6ff202e1f8c5b835ecccbc837486f`이다. 아래 수치는 Git에 추적된 경로 수다. 네 디렉터리의 HWP/HWPX 3,800개 경로는 Git blob 3,651개를 가리킨다. 같은 내용을 가리키는 추가 경로 149개가 있다. 이 수로 실제 문서 수나 유효한 한컴 Print PDF 수를 추정할 수 없다.

| 루트 | HWP | HWPX | PDF |
|---|---:|---:|---:|
| samples | 918 | 949 | 93 |
| tests/fixtures | 263 | 368 | 95 |
| mydocs/pr/assets | 380 | 578 | 541 |
| pdf | 127 | 217 | 4,036 |

수집 범위는 HWP/HWPX 원본과 PDF다. HML, 이미지, 생성 스크립트, 글꼴 등은 표에서 제외했다. 실제 문서·합성·손상 여부는 확장자나 경로로 확정하지 않았다. 저장소 루트에서 아래 명령으로 집계를 재현할 수 있다.

```python
from collections import Counter
from pathlib import PurePosixPath
from subprocess import check_output

roots = ("samples", "tests/fixtures", "mydocs/pr/assets", "pdf")
sha = "411a9f944cd6ff202e1f8c5b835ecccbc837486f"
counts, sources = Counter(), []
for entry in check_output(["git", "ls-tree", "-rz", sha, "--", *roots]).split(b"\0"):
    if not entry:
        continue
    meta, name = entry.split(b"\t", 1)
    path = PurePosixPath(name.decode())
    ext = path.suffix.lower()
    if ext in {".hwp", ".hwpx", ".pdf"}:
        root = next(r for r in roots if path.is_relative_to(r))
        counts[root, ext] += 1
        if ext != ".pdf":
            sources.append(meta.split()[-1])
print(counts, len(sources), len(set(sources)))
```

## 실제 선택과 참조

- `tools/oracle_public/oracle_resolver.py`의 `walk_samples`는 samples/ 아래 원본에서 이관 목록의 상대 경로와 보류 목록의 이름을 제외한다. 해당 SHA에서 1,867개 중 62개가 이 규칙에 해당한다. 이들은 여전히 원본 인벤토리에 속한다.
- `samples/pr9071/issue9046/hancom/01-none.hwp`와 이웃 `hancom-hwpx-to-hwp/01-none.hwp`는 파일명이 같지만 Git blob이 각각 `7b8cdd6a`와 `c1dc1b66`이다. 이름만으로 Print PDF를 공유할 수 없다.
- `tests/cases/issue_7737_ascii_quote_line_owner.rs`는 PR 증적 디렉터리의 서로 다른 두 HWPX를 직접 포함한다. 이 파일들을 이동하려면 양쪽 입력과 해당 include 경로를 함께 검증해야 한다.
- `scripts/data/contributor_visual_gate.json`의 네 쌍에 대해 고정 커밋에서 원본과 PDF의 실제 SHA-256 여덟 개가 기록과 일치했다. 이 manifest는 확인된 검증 자료의 식별 근거다. 새로 발견한 PDF가 실제로 어느 원본을 어느 조건에서 인쇄한 것인지는 생성 기록에서 별도로 확인해야 한다.
- `tools/refresh_oracle_pdfs.py`는 유일한 canonical PDF가 이미 있으면 원본의 metadata 확인을 생략할 수 있다. 원본 바이트가 교체되어도 파일명은 그대로일 수 있으므로 다음 단계에서는 기존 증거와 현재 해시가 일치하는지 검증해야 한다.

우선 기존 resolver의 후보와 증거를 가진 대응을 구분하고, 다른 경로·같은 이름의 원본과 동일 경로의 내용 교체를 반례로 삼는다. 경로 이동은 실제 소비자가 있는 문서군별로 판단한다.
