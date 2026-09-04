---
kind: working
status: done
canonical: mydocs/working/task_m100_5447_stage2.md
last_verified: 2026-08-20
---

# #5447 Stage 2 — 판정 증적을 저장소에 고정한다

- **계기**: PR [#5647](https://github.com/edwardkim/rhwp/pull/5647) 검토 보류
  (`mydocs/pr/archives/pr_5647_review.md`) — 코드 결함은 없고, **한컴 38건 판정을 재계산할
  자산이 저장소에 없다**는 것이 유일한 차단 사유였다.
- **보고서**: [`../report/task_m100_5447_report.md`](../report/archives/task_m100_5447_report.md) §1
- **원장**: [`samples/issue5447/MANIFEST.json`](../../samples/issue5447/MANIFEST.json)
- **브랜치**: `task5447` (`622d4ac1b` 위에 쌓음 — rebase 하지 않았다, §5)

## 1. 무엇을 했나

메인테이너 요구 3항에 1:1 로 대응한다.

| 요구 | 한 것 |
|---|---|
| ① 재현 가능한 bundle + 파일별 SHA-256 + 38건 판정 매니페스트 | `samples/issue5447/`(원본 38 + `PANJEONG.md`), `pdf/issue5447/`(한컴 PDF 38), `MANIFEST.json`(38행 원장) |
| ② 정상 1 + 경계 2 대표 PDF/PNG 를 안정 경로에, 보고서에서 역할·경로 구분 | `mydocs/pr/assets/pr_5647_issue5447_*.png` 3장, 보고서 §1-1 자산 3역할 표 |
| ③ 용량 제한 시 LFS 정책 위치 + 재생성 절차 | 합계 5.3 MB · 최대 111 KB 로 LFS 임계(50 MB) 미달 근거를 보고서 §1-1 과 `pdf/issue5447/README.md` 에 명시, 재생성은 보고서 §1-3 |

요구 밖에서 둘을 더 닫았다.

- **CI 트립와이어** `b2_judgment_assets_match_the_manifest` — 자산과 원장이 한쪽만 늙는 것을 막는다
- **재계산기** `tools/hancom_chart_judgment_verify.py` — 원장을 통째로 다시 계산한다

## 2. 자산의 세 역할과 경로

| 역할 | 경로 | 수 · 용량(디렉터리 합계) | 규약 근거 |
|---|---|---|---|
| 원본(판정 입력) | `samples/issue5447/` | 41 파일, 1.47 MB | `pr_review/visual_fixture_evidence.md` 「원본 fixture와 기준 PDF 보존」의 `samples/issueN` |
| 변환본(한컴의 답) | `pdf/issue5447/` | 39 파일, 3.54 MB | `pdf/README.md` §2 `{stem}-2022.pdf` + `pdf/issue_4055_b1_spike/` 의 포맷 구분자 |
| 대표 asset | `mydocs/pr/assets/pr_5647_issue5447_*.png` | 3 PNG, 0.17 MB | 같은 문서 「대표 asset과 안정 URL」, 옵션 1(현재 PR head 포함) |

합계 **5.18 MB**, 최대 단일 파일 113,064 B(110 KB). 원본 38 + `PANJEONG.md` + `MANIFEST.json` +
`README.md` = 41, PDF 38 + `README.md` = 39 이다.

PDF 명명은 같은 stem 이 두 포맷으로 존재하므로 `{stem}-hwp-2022.pdf` / `{stem}-hwpx-2022.pdf`
로 갈랐다(`00-control-hwp-2020.pdf` 선례). 38 원본 ↔ 38 PDF 가 1:1 이다.

**파일별 전체 SHA-256 은 원장에만 둔다.** README 나 보고서에 복사하지 않는다 — 두 곳에 적으면
한 곳이 조용히 늙고, 그 조용함이 이 이슈가 고치려는 실패 모드다.

### 코퍼스 스윕 안전성

`samples/` 를 훑는 게이트는 전부 비재귀다(`convert_verify_corpus_ratchet.rs` 는 `read_dir` +
확장자 필터라 하위 디렉터리가 탈락하고, `samples/chart` 만 명시적으로 재귀 합류한다).
`samples/issue5447/` 는 어디에도 잡히지 않으며, 해당 게이트를 실제로 돌려 확인했다(§4-1).
판정 자산 중 둘은 **의도적으로 의미가 깨진 경계 변종**이라 회귀 코퍼스로 승격되면 잘못된
신호를 낸다 — `samples/issue5447/README.md` 첫 절이 그 경고다.

## 3. 판정 원장 `MANIFEST.json`

38행 각각에 원본 경로·SHA-256, 한컴 PDF 경로·SHA-256, 래스터 해시 2축, 판형, 판정,
편집기 관측을 적는다. 머리에는 한컴 버전·변환일·생성기 명령·생성 시점 devel SHA·rasterizer
버전·집계가, 꼬리에는 **불변식 19건**이 붙는다.

**절대 래스터 해시는 도구에 딸린 값**이므로 두 축을 따로 적고 섞지 않는다.

| 축 | 도구 | 상태 |
|---|---|---|
| `pdftoppm_144dpi_ppm_sha256` | poppler `pdftoppm -r 144 -f 1 -l 1` | 보고서 §2 표의 축. poppler 25.12.0 으로 재계산 |
| `pymupdf_144dpi_rgb_sha256` | PyMuPDF 1.28.0 (MuPDF 1.29.0) 144dpi RGB | poppler 없이도 도는 기본 축 |

도구를 넘어 성립해야 하는 계약은 `invariants` 19건이다 — 포맷 쌍 동일 14, 변환본 == 원본 hwp
행추가, 원형 계열추가 == 대조군, 주식형 계열삭제 != 대조군, 전건 1쪽 `1190x1682`,
판정 단위 15와 분포.

### 숫자 정정

보고서 초판 §2 는 "18 변종", PR 본문 S1 은 "17/18 반영" 이었다. **둘 다 근거가 없다.**
생성기 자신이 `대조군 9 + 변종 14 × 2포맷 + 변환본 1 = 38` 로 단언하므로 변종은 14,
판정 단위는 14 + 1 = **15** 다. 원장을 만들며 래스터로 다시 세니:

```text
판정 단위 15 — {'반영': 13, '반영_의미깨짐': 1, '미반영': 1}
```

원장 머리의 `counts` 와 트립와이어가 이제 이 숫자를 강제한다.

## 4. 검증 실측 (2026-08-20, dev profile, worktree `../rhwp-5647`)

| 게이트 | 결과 |
|---|---|
| `cargo test --test issue_4100_chart_data_edit` | **38 passed / 0 failed / 2 ignored**, 4.2s (Stage 1 의 37 + 트립와이어 1) |
| `b2_judgment_assets_match_the_manifest` 단독 | ok, 0.16s |
| `python tools/hancom_chart_judgment_verify.py` (PyMuPDF) | **통과 — 검사 262건 전건 일치 (38 파일)** |
| `python …verify.py --rasterizer none` | 통과 — 검사 186건 |
| 원장 위조 음성 테스트(파일 해시·판정·불변식 각 1건 조작) | **실패 4건 정확히 검출**, exit 1 |
| poppler 축 독립 재계산 (`minidocks/poppler` 25.12.0) | 38/38 원장과 일치 |
| **보고서 §2 해시 접두사 15건 대조** | **15/15 일치** — `af8236e8…`·`e16a2a67…` 포함 |
| `rustfmt --check tests/issue_4100_chart_data_edit.rs` | 통과 |

`--rasterizer pdftoppm` 은 이 PC 에 poppler 가 없어 안내와 함께 exit 1 한다 — 스크립트가
컨테이너 대안 명령을 출력하고, 그 명령으로 낸 값이 위 표의 "poppler 축 독립 재계산" 이다.

### 4-1. 코퍼스 스윕 무간섭 — `samples/` 를 훑는 게이트를 직접 돌렸다

`samples/` 나 `samples/chart` 를 열거하는 타깃만 골라 실행했다(전체 `cargo test` 는 dev cold
빌드 비용 때문에 CI 몫이다 — Stage 1 과 같은 운용).

| 타깃 | 포함된 스윕 | 결과 |
|---|---|---|
| `issue_4055_b1_chart_edit_probe` | `checked == 56` 차트 코퍼스 고정 | ok 9 passed |
| `issue_4100_chart_data_edit` | `CORPUS_FILES = 56`, 코퍼스 3종 | ok 38 passed / 2 ignored |
| `overflow_cell_baseline` | `read_dir(samples/)` | ok 1 passed (436s) |
| `regression_suite_008` | `hwpx_roundtrip_baseline` 등 | ok 125 passed |
| `regression_suite_012` | `convert_verify_corpus_ratchet`(samples/ + samples/chart 재귀) | **ok 110 passed** (227s) |
| `regression_suite_021` | `issue_3546_chart_preserved_on_save` 등 | 105 passed / **2 failed** ↓ |
| `regression_suite_023` | `issue_4097_mini_cfb_root_clsid` 등 | 113 passed / **1 failed** ↓ |

**코퍼스 열거 테스트는 전건 통과했다.** `samples/issue5447/` 는 하위 디렉터리라 비재귀
`read_dir` 의 확장자 필터에서 탈락하고, 재귀 합류 대상은 `samples/chart` 뿐이라 잡히지 않는다.

실패 3건은 **이 변경과 무관한 Windows 체크아웃·부하 산물**이고 원인을 각각 확인했다.

| 실패 | 원인 |
|---|---|
| `agent_bug_hunter_skill_contract::skill_frontmatter_names_bug_hunter`<br>`agent_fde_skill_contract::skill_frontmatter_names_fde` | 테스트가 `starts_with("---\n")` 를 단언하는데 `core.autocrlf=true` 체크아웃이라 파일이 `---\r\n` 으로 시작한다(`b'---\r\nn'` 실측). 두 `SKILL.md` 는 이 PR 이 손대지 않았고 `git status .agents/` 는 청결. Linux CI 에는 없는 실패다 |
| `issue_2833_hml_adapter_row_sizes::inflated_row_count_does_not_slow_down_parsing` | 500ms 상한 성능 테스트가 **최적화 없는 dev 빌드 + 7타깃 병렬 부하**에서 794.7ms. `--test-threads=1` 단독 재실행은 **0.26s 로 통과**했다 |

### 4-2. 재생성 결정성 — 페이로드는 결정적, ZIP 호스트 바이트만 갈린다

생성기를 다시 돌려 커밋본과 바이트 대조했다.

```text
재생성 39  커밋본 39   →  바이트 동일 25, 불일치 14 (전부 .hwpx 변종)
hwp(CFB) 15건 — 바이트 동일 15
대조군 hwpx 9건 — 바이트 동일 9
```

불일치 14건을 열어 보면 **차이가 ZIP 중앙 디렉터리의 `create_system` 1바이트씩뿐**이다:

| 항목 | 재생성(Windows) | 커밋본(판정 당시) |
|---|---|---|
| 엔트리 이름·순서 | 동일 (15개) | 동일 |
| 엔트리 내용 SHA-256 | 전건 동일 | 전건 동일 |
| CRC · 압축크기 · 원본크기 · 압축방식 | 전건 동일 | 전건 동일 |
| `date_time` | `1980-01-01` (`fixed_mtime()`) | 동일 |
| **`create_system`** | **0 (FAT)** | **3 (Unix)** |
| 파일 크기 | 31097 | 31097 — 다른 바이트 15개 = 엔트리 수 |

`zip` 크레이트가 "version made by" 의 호스트 OS 를 빌드 플랫폼에서 가져오기 때문이고,
**문서 내용에는 아무 영향이 없다.** 즉 HWPX 내보내기는 **플랫폼 안에서 결정적**이고,
같은 플랫폼(Unix)에서 재생성하면 커밋본과 바이트까지 같아질 것으로 본다.

그래도 **커밋본이 정본**이다 — 한컴이 실제로 연 것은 여기 있는 바이트이고, 판정은 그 바이트에
대한 관측이기 때문이다. 생성기 doc-comment 에 그 문장을 박아 뒀다.

## 5. 브랜치 운용 — rebase 하지 않았다

메인테이너가 리뷰 문서에서 trailing commit `622d4ac1` 을 SHA 로 인용했고 PR 이 `MERGEABLE`
이다. rebase 는 그 SHA 를 다시 써서 인용을 깨뜨린다. 그래서 **커밋을 위에 쌓았다.**
최신 `upstream/devel` 과의 정합은 별도로 확인한다(§6).

## 6. 남은 것

- PR 본문 갱신(숫자 정정 + 증적 경로 + 재검증 명령)과 리뷰 회신
- 대표 PNG 의 `devel` raw URL 승격은 merge 이후 메인테이너 경로
  (`visual_fixture_evidence.md` 「asset 반영 경로」 옵션 M/2)
