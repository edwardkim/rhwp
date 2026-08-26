#!/usr/bin/env python3
"""rhwp-strategist 스킬 팩 방출기.

실 에이전트가 engagement.py · search/extract-data 봉투 · §5 게이트를
조립하도록 레퍼런스·예제·픽스처를 커밋한다. 새 CLI 를 만들지 않는다.
이 파일을 실행하면 references/fixtures·상위 fixtures·examples 를 다시 쓴다.
"""

from __future__ import annotations

import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
SKILL = HERE.parent
REF = HERE
EX = SKILL / "examples"
FIX = SKILL / "fixtures"
ISSUE = 5335
CAP = "CAP-4903"
ENGINE = "tools/strategist/engagement.py"
PLAYBOOK = "mydocs/manual/strategist_playbook.md"

ALLOWED_COMMANDS = (
    "info",
    "search",
    "extract-data",
    "explain",
    "scaffold",
    "capabilities",
)
INVENTED = (
    "strategy",
    "claim-check",
    "forecast",
    "strategist",
    "evidence-ledger",
    "claim-gate",
)

COORD_KEYS = (
    "section",
    "paragraph",
    "page",
    "charOffset",
    "length",
    "cell",
    "textbox",
)


def dump(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(obj, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
        newline="\n",
    )


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if not text.endswith("\n"):
        text += "\n"
    path.write_text(text, encoding="utf-8", newline="\n")


# ---------------------------------------------------------------------------
# Domain corpora used by fixtures and examples
# ---------------------------------------------------------------------------

CORPORA = {
    "gov_rfp": {
        "id": "gov_rfp",
        "objective": "2026년 스마트시티 데이터 플랫폼 정부과제 수주",
        "deliverable": "스마트시티 데이터 플랫폼 수주 근거 보고서",
        "corpus": "corpus/smartcity-rfp",
        "docs": [
            "rfp_공고.hwpx",
            "과업지시서.hwp",
            "예산서_2025.hwpx",
            "선행연구_중간.hwpx",
            "평가표.hwp",
        ],
        "questions": [
            {
                "id": "Q1",
                "text": "발주 기관이 명시한 필수 기능은 무엇인가",
                "keywords": ["필수기능", "데이터 플랫폼", "표준 API"],
            },
            {
                "id": "Q2",
                "text": "배정 예산과 수행 기간은 문서에 어떻게 적혀 있는가",
                "keywords": ["총사업비", "수행기간", "백만원"],
            },
            {
                "id": "Q3",
                "text": "평가 배점에서 기술·가격 비중은 얼마인가",
                "keywords": ["기술평가", "가격평가", "배점"],
            },
        ],
    },
    "quarterly": {
        "id": "quarterly",
        "objective": "공공 클라우드 사업 2026년 3분기 전략 보고서",
        "deliverable": "2026 3Q 공공 클라우드 전략 보고서",
        "corpus": "corpus/cloud-q3",
        "docs": [
            "2Q_실적.hwpx",
            "수주파이프.hwp",
            "인력배치.hwpx",
            "리스크등록부.hwp",
        ],
        "questions": [
            {
                "id": "Q1",
                "text": "2분기 수주액과 잔여 파이프는 얼마인가",
                "keywords": ["수주액", "잔여파이프", "억원"],
            },
            {
                "id": "Q2",
                "text": "3분기 투입 인력과 공수 제약은 무엇인가",
                "keywords": ["MM", "투입인력", "가용공수"],
            },
            {
                "id": "Q3",
                "text": "문서가 기록한 리스크와 대응은 무엇인가",
                "keywords": ["리스크", "대응방안", "일정지연"],
            },
        ],
    },
    "mixed_failed": {
        "id": "mixed_failed",
        "objective": "암호 문서가 섞인 코퍼스에서 수주 근거만 추리기",
        "deliverable": "부분 가독 코퍼스 근거 보고서",
        "corpus": "corpus/mixed-cipher",
        "docs": [
            "공개_공고.hwpx",
            "암호_내부.hwp",
            "손상_백업.hwpx",
            "평가기준.hwp",
        ],
        "failed": [
            {"file": "암호_내부.hwp", "infoExit": 2, "reason": "암호 보호"},
            {"file": "손상_백업.hwpx", "infoExit": 1, "reason": "파싱 실패"},
        ],
        "questions": [
            {
                "id": "Q1",
                "text": "공개 공고의 마감일은 언제인가",
                "keywords": ["마감", "제출기한"],
            },
            {
                "id": "Q2",
                "text": "평가 기준 문서의 가점은 무엇인가",
                "keywords": ["가점", "실적"],
            },
        ],
    },
}

# Realistic quotes pulled from fictional but document-shaped Korean prose.
QUOTES = [
    ("표준 API 연계", "발주기관은 표준 API 연계를 필수기능으로 명시한다.", 0, 12, 2, 40, 8),
    ("데이터 플랫폼", "본 사업의 핵심은 도시 데이터 플랫폼 구축이다.", 0, 4, 1, 8, 7),
    ("총사업비", "총사업비는 3,180백만원(부가세 별도)으로 한다.", 0, 7, 0, 55, 11),
    ("수행기간", "수행기간은 계약일로부터 14개월이다.", 1, 2, 3, 0, 5),
    ("기술평가", "기술평가 80점, 가격평가 20점으로 합산한다.", 0, 21, 5, 0, 4),
    ("배점", "정성 평가 배점표는 별첨 2와 같다.", 0, 22, 5, 18, 2),
    ("수주액", "2분기 공공 클라우드 수주액은 42억원이다.", 0, 3, 1, 14, 3),
    ("잔여파이프", "잔여파이프 61억원 중 28억원이 3분기 확정 예정이다.", 0, 5, 1, 0, 5),
    ("MM", "3분기 가용 공수는 36MM 이다.", 0, 8, 2, 10, 2),
    ("투입인력", "투입인력은 아키텍트 2, 개발 6, 운영 2 명이다.", 0, 9, 2, 0, 4),
    ("리스크", "일정지연 리스크는 고(High)로 등록되어 있다.", 0, 14, 4, 0, 3),
    ("대응방안", "대응방안은 선투입 2MM 와 외주 슬롯 1건이다.", 0, 15, 4, 12, 4),
    ("마감", "전자 제출 마감은 2026-09-12 18:00 이다.", 0, 1, 0, 6, 2),
    ("제출기한", "제출기한을 넘긴 제안서는 무효로 한다.", 0, 2, 0, 0, 4),
    ("가점", "유사 실적 가점은 최대 5점이다.", 0, 6, 1, 8, 2),
    ("실적", "최근 3년 공공 데이터 플랫폼 실적만 인정한다.", 0, 7, 1, 14, 2),
    ("필수기능", "필수기능 미충족 시 기술평가를 중단한다.", 0, 13, 2, 0, 4),
    ("표준 API", "표준 API 명세는 별첨 1 OpenAPI 3.0 을 따른다.", 0, 14, 2, 22, 8),
    ("백만원", "직접인건비 1,240백만원, 경비 410백만원.", 0, 8, 0, 0, 3),
    ("가격평가", "가격평가 산식은 최저가 대비 상대평가이다.", 0, 23, 5, 0, 4),
]


def quote_row(i: int) -> dict:
    kw, text, sec, para, page, off, length = QUOTES[i % len(QUOTES)]
    return {
        "keyword": kw,
        "text": text,
        "section": sec,
        "paragraph": para,
        "page": page,
        "charOffset": off,
        "length": length,
        "context": f"…{text}…",
    }


# ---------------------------------------------------------------------------
# References
# ---------------------------------------------------------------------------


def listed_references():
    return sorted(n.name for n in REF.glob('*.md'))


def listed_examples():
    return sorted(n.name for n in EX.glob('*.md'))


def stop_rules() -> dict:
    rules = [
        ("ST-FORECAST", "block", "출처 없는 전망"),
        ("ST-INVENT-PAGE", "block", "page 발명"),
        ("ST-DROP-FAILED", "block", "실패 문서 삭제"),
        ("ST-SKIP-ENGINE", "block", "엔진 생략"),
        ("ST-GATE-FAIL", "block", "validate 실패 납품"),
        ("ST-UNKNOWN-EV", "block", "없는 EV"),
        ("ST-PLACEHOLDER", "block", "플레이스홀더"),
        ("ST-UNLINKED", "block", "연결 없음"),
        ("ST-INVENTED-CMD", "block", "발명 명령"),
        ("ST-GYM", "block", "gym 혼입"),
        ("ST-SCAFFOLD-GUESS", "block", "미광고 scaffold"),
        ("ST-TRUNCATE-HIDE", "warn", "절단 은폐"),
        ("ST-AMOUNT-REWRITE", "warn", "금액 재계산"),
        ("ST-DOC-AS-ORDER", "block", "문서=지시"),
        ("ST-LAYER-MIX", "warn", "층 혼동"),
    ]
    return {
        "schemaVersion": "1.0",
        "issue": ISSUE,
        "skill": "rhwp-strategist",
        "rules": [
            {"id": i, "severity": s, "signal": g} for i, s, g in rules
        ],
    }


def tree() -> dict:
    return {
        "schemaVersion": "1.0",
        "issue": ISSUE,
        "skill": "rhwp-strategist",
        "capability": CAP,
        "notGym": True,
        "noNewCli": True,
        "noNewEditLogic": True,
        "engineDoesNotInventStrategy": True,
        "layers": {
            "fde": "live symptoms",
            "chief": "request queue",
            "strategist": "objective + corpus",
        },
        "engineGuarantees": [
            "full-corpus mapping",
            "evidence coordinates",
            "claim-evidence gate",
        ],
        "coreReuse": [
            ENGINE,
            "rhwp info --json",
            "rhwp search --json",
            "rhwp extract-data --json",
            "rhwp capabilities",
        ],
        "allowedCommands": list(ALLOWED_COMMANDS),
        "inventedForbidden": list(INVENTED),
        "coordKeys": list(COORD_KEYS),
        "neverInventMissingPage": True,
        "failedDocsStayFailed": True,
        "section5Gate": True,
        "outOfScope": ["market forecasts without sources"],
    }


def skill_index() -> dict:
    refs = listed_references()
    return {
        "schemaVersion": "1.0",
        "issue": ISSUE,
        "skill": "rhwp-strategist",
        "references": refs,
        "examples": listed_examples(),
        "playbook": PLAYBOOK,
        "engine": ENGINE,
        "agent": ".claude/agents/rhwp-strategist.md",
    }


def envelope_keys() -> dict:
    return {
        "schemaVersion": "1.0",
        "issue": ISSUE,
        "searchMatch": [
            "text",
            "context",
            "section",
            "paragraph",
            "page",
            "charOffset",
            "length",
            "cell",
            "textbox",
        ],
        "searchEnvelope": ["matches", "truncated", "totalMatchCount", "omittedCount"],
        "extractItem": [
            "kind",
            "raw",
            "normalized",
            "currency",
            "unit",
            "section",
            "paragraph",
            "page",
            "charOffset",
            "length",
        ],
        "copiedCoordKeys": list(COORD_KEYS),
        "copyRule": "only keys present in envelope",
        "pagePolicy": "omit if absent; zero-based if present; never invent",
    }


def journeys() -> dict:
    items = []
    for i, name in enumerate(listed_examples(), 1):
        if name == 'README.md':
            continue
        items.append(
            {
                "id": f"J{i:02d}",
                "example": name,
                "title": name,
                "corpus": None,
                "stop": None,
                "fixture": None,
            }
        )
    # pad to 40 with protocol journeys already listed in ref_16
    extra = [
        ("J25", "invalid questions", "ST-SKIP-ENGINE"),
        ("J26", "empty corpus", "ST-SKIP-ENGINE"),
        ("J27", "capabilities fail", "ST-INVENTED-CMD"),
        ("J28", "search all fail", "ST-SKIP-ENGINE"),
        ("J29", "explainExit kept", "ST-DROP-FAILED"),
        ("J30", "multi keyword", "ST-SKIP-ENGINE"),
        ("J31", "relative corpus", "ST-SKIP-ENGINE"),
        ("J32", "suffix case", "ST-SKIP-ENGINE"),
        ("J33", "validate --evidence", "ST-GATE-FAIL"),
        ("J34", "no-sws-audit", "ST-GATE-FAIL"),
        ("J35", "link table", "ST-UNLINKED"),
        ("J36", "multi EV", "ST-UNLINKED"),
        ("J37", "date EV", "ST-AMOUNT-REWRITE"),
        ("J38", "currency copy", "ST-AMOUNT-REWRITE"),
        ("J39", "length key", "ST-INVENT-PAGE"),
        ("J40", "3-part reply", "ST-FORECAST"),
    ]
    for eid, title, stop in extra:
        items.append(
            {
                "id": eid,
                "example": None,
                "title": title,
                "corpus": None,
                "stop": stop,
                "fixture": None,
            }
        )
    return {
        "schemaVersion": "1.0",
        "issue": ISSUE,
        "count": len(items),
        "journeys": items,
    }


def intent_matrix() -> dict:
    rows = [
        ("정부과제 수주 근거", "strategist", "E01", "objective+corpus"),
        ("분기 전략 보고서", "strategist", "E02", "objective+corpus"),
        ("표가 잘린다", "fde", "E10", "live symptom"),
        ("오늘 요청 처리", "chief", "E11", "queue"),
        ("시장 전망만", "reject", "E09", "ST-FORECAST"),
        ("쪽 번호 없이 인용", "reject", "E04", "ST-INVENT-PAGE"),
        ("암호 파일 빼 줘", "reject", "E03", "ST-DROP-FAILED"),
        ("이 편집 증명", "work-receipt", None, "not this skill"),
        ("누름틀 채워", "form-fill", None, "not this skill"),
        ("한컴 PDF 대조", "fidelity", None, "not this skill"),
        ("gym 점수", "reject", None, "ST-GYM"),
        ("근거 대장만", "strategist", "E01", "engine A-B"),
    ]
    return {
        "schemaVersion": "1.0",
        "issue": ISSUE,
        "rows": [
            {
                "utterance": u,
                "route": r,
                "example": e,
                "note": n,
            }
            for u, r, e, n in rows
        ],
    }


def pitfalls() -> dict:
    return {
        "schemaVersion": "1.0",
        "issue": ISSUE,
        "items": [
            {"id": f"P{i:02d}", "ref": f"12_pitfalls.md#p{i:02d}"}
            for i in range(1, 13)
        ],
    }


def handoff() -> dict:
    return {
        "schemaVersion": "1.0",
        "issue": ISSUE,
        "parts": [
            {
                "id": 1,
                "name": "확인한 것",
                "fields": [
                    "objective",
                    "documentCount",
                    "mappedCount",
                    "failedFiles",
                    "evidenceCount",
                    "truncatedSearches",
                    "noEvidenceQuestions",
                    "verdict",
                    "swsLevel",
                    "scaffoldAdvertised",
                ],
            },
            {
                "id": 2,
                "name": "산출물",
                "fields": [
                    "spec.json",
                    "evidence.json",
                    "corpus_map.json",
                    "validate envelope",
                    "deliverable.hwpx?",
                ],
            },
            {
                "id": 3,
                "name": "다음",
                "fields": [
                    "password request",
                    "keyword candidates",
                    "limit revisit",
                    "fde/chief handoff",
                ],
            },
        ],
        "forbiddenPhrases": [
            "대략 다 읽었습니다",
            "시장은 긍정적입니다",
            "수주 확률",
        ],
    }


def catalog() -> dict:
    return {
        "schemaVersion": "1.0",
        "issue": ISSUE,
        "skill": "rhwp-strategist",
        "capability": CAP,
        "attributionClaim": False,
        "signatureClaim": False,
        "engineDoesNotInventStrategy": True,
        "commands": list(ALLOWED_COMMANDS),
        "engine": [ENGINE],
        "references": listed_references(),
        "examples": [n for n in listed_examples() if n != "README.md"],
        "envelopes": [
            "search_with_page.json",
            "search_missing_page.json",
            "search_truncated.json",
            "search_cell.json",
            "search_textbox.json",
            "extract_amount.json",
            "extract_date.json",
            "info_ok.json",
            "info_failed.json",
            "engagement_summary_no_scaffold.json",
            "validate_pass.json",
            "validate_fail.json",
        ],
        "engagements": [
            "gov_rfp.json",
            "quarterly.json",
            "mixed_failed.json",
            "invalid_missing_objective.json",
            "invalid_missing_questions.json",
            "invalid_empty_questions.json",
        ],
    }


def engagement_fixture(key: str) -> dict:
    c = CORPORA[key]
    return {
        "objective": c["objective"],
        "corpus": c["corpus"],
        "questions": c["questions"],
        "deliverable": c["deliverable"],
        "searchLimit": 20,
        "_skillMeta": {"id": key, "issue": ISSUE},
    }


def invalid_engagements() -> dict[str, dict]:
    return {
        "invalid_missing_objective.json": {
            "corpus": "corpus/x",
            "questions": ["a"],
        },
        "invalid_missing_questions.json": {
            "objective": "목표",
            "corpus": "corpus/x",
        },
        "invalid_empty_questions.json": {
            "objective": "목표",
            "corpus": "corpus/x",
            "questions": [],
        },
        "invalid_question_scalar.json": {
            "objective": "목표",
            "corpus": "corpus/x",
            "questions": [3],
        },
        "invalid_question_empty_obj.json": {
            "objective": "목표",
            "corpus": "corpus/x",
            "questions": [{"id": "Q1"}],
        },
    }


def build_ledger(key: str, *, drop_page_on: int | None = None,
                 truncate: bool = False, zero_last: bool = False) -> dict:
    c = CORPORA[key]
    entries = []
    failures = []
    truncated = []
    ev = 1
    for qi, q in enumerate(c["questions"]):
        if zero_last and qi == len(c["questions"]) - 1:
            continue
        # 질문당 키워드 2개 × 가독 문서 2개 — 전 행렬을 펼치지 않는다.
        keywords = q["keywords"][:2]
        readable = [
            d for d in c["docs"]
            if d not in {x["file"] for x in c.get("failed", [])}
        ][:2]
        failed_docs = [x["file"] for x in c.get("failed", [])]
        if key == "mixed_failed":
            for fd in failed_docs:
                failures.append(
                    {
                        "phase": "search",
                        "file": fd,
                        "keyword": keywords[0],
                        "reason": "exit 1",
                    }
                )
        for di, doc in enumerate(readable):
            for kw in keywords:
                row = quote_row(ev + di + qi)
                entry = {
                    "id": f"EV-{ev}",
                    "kind": "search",
                    "question": q["id"],
                    "keyword": kw,
                    "file": doc,
                    "section": row["section"],
                    "paragraph": row["paragraph"] + di,
                    "charOffset": row["charOffset"],
                    "length": row["length"],
                    "quote": row["text"],
                    "context": row["context"],
                    "command": f"rhwp search {c['corpus']}/{doc} --json -- {kw}",
                }
                if drop_page_on != ev:
                    entry["page"] = row["page"]
                if "평가표" in doc:
                    entry["cell"] = {"row": 2, "col": 1}
                if "공고" in doc and qi == 0:
                    entry["textbox"] = "tb-cover-1"
                entries.append(entry)
                ev += 1
                if truncate and len(truncated) == 0 and kw == keywords[0]:
                    truncated.append(
                        {
                            "file": doc,
                            "keyword": kw,
                            "totalMatchCount": 41,
                            "omittedCount": 36,
                        }
                    )
    # data entries
    if key != "mixed_failed":
        doc = c["docs"][0]
        entries.append(
            {
                "id": f"EV-{ev}",
                "kind": "data",
                "dataKind": "amount",
                "file": doc,
                "section": 0,
                "paragraph": 7,
                "page": 0,
                "charOffset": 55,
                "length": 11,
                "quote": "3,180백만원",
                "normalized": 3180000000,
                "currency": "KRW",
                "command": f"rhwp extract-data {c['corpus']}/{doc} --kind amount --json",
            }
        )
        ev += 1
        entries.append(
            {
                "id": f"EV-{ev}",
                "kind": "data",
                "dataKind": "date",
                "file": doc,
                "section": 0,
                "paragraph": 1,
                "page": 0,
                "charOffset": 6,
                "length": 10,
                "quote": "2026-09-12",
                "normalized": "2026-09-12",
                "command": f"rhwp extract-data {c['corpus']}/{doc} --kind date --json",
            }
        )
    return {
        "schemaVersion": "1",
        "generatedBy": ENGINE,
        "corpus": c["corpus"],
        "entryCount": len(entries),
        "truncatedSearches": truncated,
        "failures": failures,
        "entries": entries,
        "_skillMeta": {"id": key, "issue": ISSUE},
    }


def corpus_map(key: str) -> dict:
    c = CORPORA[key]
    failed = {x["file"]: x for x in c.get("failed", [])}
    docs = []
    for i, f in enumerate(c["docs"]):
        if f in failed:
            docs.append(
                {
                    "file": f,
                    "sizeBytes": 10000 + i * 111,
                    "status": "failed",
                    "infoExit": failed[f]["infoExit"],
                }
            )
        else:
            docs.append(
                {
                    "file": f,
                    "sizeBytes": 80000 + i * 333,
                    "status": "ok",
                    "info": {
                        "format": "hwpx" if f.endswith("x") else "hwp",
                        "pageCount": 6 + i,
                    },
                }
            )
    return {
        "schemaVersion": "1",
        "generatedBy": ENGINE,
        "corpus": c["corpus"],
        "documentCount": len(docs),
        "mappedCount": sum(1 for d in docs if d["status"] == "ok"),
        "documents": docs,
        "_skillMeta": {"id": key, "issue": ISSUE},
    }


def all_failed_map() -> dict:
    docs = [
        {"file": "a.hwp", "sizeBytes": 10, "status": "failed", "infoExit": 1},
        {"file": "b.hwpx", "sizeBytes": 11, "status": "failed", "infoExit": None},
    ]
    return {
        "schemaVersion": "1",
        "generatedBy": ENGINE,
        "corpus": "corpus/dead",
        "documentCount": 2,
        "mappedCount": 0,
        "documents": docs,
        "_skillMeta": {"id": "all_failed", "issue": ISSUE},
    }


def envelopes() -> dict[str, dict]:
    q = quote_row(1)
    return {
        "search_with_page.json": {
            "matches": [
                {
                    "text": q["text"],
                    "context": q["context"],
                    "section": q["section"],
                    "paragraph": q["paragraph"],
                    "page": q["page"],
                    "charOffset": q["charOffset"],
                    "length": q["length"],
                }
            ],
            "truncated": False,
            "totalMatchCount": 1,
            "omittedCount": 0,
            "_skillMeta": {"command": "search", "exit": 0},
        },
        "search_missing_page.json": {
            "matches": [
                {
                    "text": "선행 연구의 가설은 다음과 같다.",
                    "context": "…선행 연구의 가설은 다음과 같다.…",
                    "section": 0,
                    "paragraph": 88,
                    "charOffset": 14,
                    "length": 2,
                }
            ],
            "truncated": False,
            "totalMatchCount": 1,
            "_skillMeta": {
                "command": "search",
                "exit": 0,
                "note": "page absent — do not invent",
            },
        },
        "search_truncated.json": {
            "matches": [
                {
                    "text": f"데이터 언급 {i}",
                    "section": 0,
                    "paragraph": i,
                    "page": i // 3,
                    "charOffset": 0,
                    "length": 3,
                }
                for i in range(5)
            ],
            "truncated": True,
            "totalMatchCount": 41,
            "omittedCount": 36,
            "_skillMeta": {"command": "search", "exit": 0},
        },
        "search_cell.json": {
            "matches": [
                {
                    "text": "기술평가 80",
                    "section": 0,
                    "paragraph": 0,
                    "page": 5,
                    "charOffset": 0,
                    "length": 6,
                    "cell": {"row": 2, "col": 1},
                }
            ],
            "truncated": False,
            "totalMatchCount": 1,
            "_skillMeta": {"command": "search", "exit": 0},
        },
        "search_textbox.json": {
            "matches": [
                {
                    "text": "필수기능",
                    "section": 0,
                    "paragraph": 0,
                    "page": 0,
                    "charOffset": 0,
                    "length": 4,
                    "textbox": "tb-cover-1",
                }
            ],
            "truncated": False,
            "totalMatchCount": 1,
            "_skillMeta": {"command": "search", "exit": 0},
        },
        "extract_amount.json": {
            "items": [
                {
                    "kind": "amount",
                    "raw": "3,180백만원",
                    "normalized": 3180000000,
                    "currency": "KRW",
                    "section": 0,
                    "paragraph": 7,
                    "page": 0,
                    "charOffset": 55,
                    "length": 11,
                }
            ],
            "_skillMeta": {"command": "extract-data", "exit": 0},
        },
        "extract_date.json": {
            "items": [
                {
                    "kind": "date",
                    "raw": "2026-09-12",
                    "normalized": "2026-09-12",
                    "section": 0,
                    "paragraph": 1,
                    "page": 0,
                    "charOffset": 6,
                    "length": 10,
                }
            ],
            "_skillMeta": {"command": "extract-data", "exit": 0},
        },
        "info_ok.json": {
            "format": "hwpx",
            "pageCount": 8,
            "title": "과업지시서",
            "_skillMeta": {"command": "info", "exit": 0},
        },
        "info_failed.json": {
            "error": "encrypted or unreadable",
            "_skillMeta": {"command": "info", "exit": 2},
        },
        "engagement_summary_no_scaffold.json": {
            "schemaVersion": "1",
            "generatedBy": ENGINE,
            "mode": "engagement",
            "objective": CORPORA["gov_rfp"]["objective"],
            "corpusDocuments": 5,
            "mappedDocuments": 5,
            "evidenceCount": 18,
            "searchFailures": 0,
            "questionCount": 3,
            "claimCount": 3,
            "noEvidenceQuestions": [],
            "scaffoldAdvertised": False,
            "scaffold": None,
            "artifacts": ["corpus_map.json", "evidence.json", "spec.json"],
            "_skillMeta": {"command": "engagement.py", "exit": 0},
        },
        "validate_pass.json": {
            "schemaVersion": "1",
            "generatedBy": ENGINE,
            "mode": "validate",
            "claimCount": 2,
            "ledgerEntryCount": 18,
            "violationCount": 0,
            "violations": [],
            "verdict": "pass",
            "_skillMeta": {"command": "engagement.py --validate", "exit": 0},
        },
        "validate_fail.json": {
            "schemaVersion": "1",
            "generatedBy": ENGINE,
            "mode": "validate",
            "claimCount": 1,
            "ledgerEntryCount": 18,
            "violationCount": 1,
            "violations": [
                {
                    "claim": "CLAIM-1",
                    "kind": "unlinked",
                    "detail": "실존 EV id 에 연결된 근거가 하나도 없다",
                }
            ],
            "verdict": "fail",
            "_skillMeta": {"command": "engagement.py --validate", "exit": 3},
        },
        "capabilities_subset.json": {
            "commands": [
                {"name": "info"},
                {"name": "search"},
                {"name": "extract-data"},
                {"name": "explain"},
            ],
            "_skillMeta": {"command": "capabilities", "exit": 0},
        },
    }


def validate_cases() -> dict[str, dict]:
    return {
        "pass.json": {
            "schemaVersion": "1",
            "generatedBy": ENGINE,
            "mode": "validate",
            "claimCount": 2,
            "ledgerEntryCount": 12,
            "violationCount": 0,
            "violations": [],
            "verdict": "pass",
            "_skillMeta": {"exit": 0, "kind": "pass"},
        },
        "pass_with_sws.json": {
            "schemaVersion": "1",
            "generatedBy": ENGINE,
            "mode": "validate",
            "claimCount": 2,
            "ledgerEntryCount": 12,
            "violationCount": 0,
            "violations": [],
            "verdict": "pass",
            "swsAudit": {
                "attained": "L2",
                "rereadVerified": True,
                "deliverableFile": "sws_deliverable.json",
                "reportFile": "sws_audit.json",
            },
            "_skillMeta": {"exit": 0, "kind": "pass"},
        },
        "placeholder.json": {
            "schemaVersion": "1",
            "generatedBy": ENGINE,
            "mode": "validate",
            "claimCount": 1,
            "ledgerEntryCount": 12,
            "violationCount": 1,
            "violations": [
                {
                    "claim": "CLAIM-1",
                    "kind": "placeholder",
                    "detail": "플레이스홀더가 실제 주장으로 작성되지 않았다",
                }
            ],
            "verdict": "fail",
            "_skillMeta": {"exit": 3, "kind": "placeholder"},
        },
        "unknown_evidence.json": {
            "schemaVersion": "1",
            "generatedBy": ENGINE,
            "mode": "validate",
            "claimCount": 1,
            "ledgerEntryCount": 12,
            "violationCount": 1,
            "violations": [
                {
                    "claim": "CLAIM-1",
                    "kind": "unknown-evidence",
                    "detail": "근거 대장에 없는 id: EV-99",
                }
            ],
            "verdict": "fail",
            "_skillMeta": {"exit": 3, "kind": "unknown-evidence"},
        },
        "unlinked.json": {
            "schemaVersion": "1",
            "generatedBy": ENGINE,
            "mode": "validate",
            "claimCount": 1,
            "ledgerEntryCount": 12,
            "violationCount": 1,
            "violations": [
                {
                    "claim": "CLAIM-1",
                    "kind": "unlinked",
                    "detail": "실존 EV id 에 연결된 근거가 하나도 없다",
                }
            ],
            "verdict": "fail",
            "_skillMeta": {"exit": 3, "kind": "unlinked"},
        },
        "mixed_violations.json": {
            "schemaVersion": "1",
            "generatedBy": ENGINE,
            "mode": "validate",
            "claimCount": 2,
            "ledgerEntryCount": 12,
            "violationCount": 2,
            "violations": [
                {
                    "claim": "CLAIM-1",
                    "kind": "placeholder",
                    "detail": "플레이스홀더가 실제 주장으로 작성되지 않았다",
                },
                {
                    "claim": "CLAIM-2",
                    "kind": "unknown-evidence",
                    "detail": "근거 대장에 없는 id: EV-77",
                },
            ],
            "verdict": "fail",
            "_skillMeta": {"exit": 3, "kind": "mixed"},
        },
    }


def spec_samples() -> dict[str, dict]:
    return {
        "pass_spec.json": {
            "version": "1",
            "title": CORPORA["gov_rfp"]["deliverable"],
            "blocks": [
                {"type": "heading", "level": 1, "text": CORPORA["gov_rfp"]["deliverable"]},
                {
                    "type": "paragraph",
                    "text": "목표: " + CORPORA["gov_rfp"]["objective"],
                },
                {
                    "type": "paragraph",
                    "text": "발주 공고는 표준 API 연계를 필수기능으로 명시한다. [근거: EV-1, EV-2] CLAIM-1",
                },
                {
                    "type": "paragraph",
                    "text": "총사업비는 3,180백만원으로 적혀 있다. [근거: EV-3] CLAIM-2",
                },
                {
                    "type": "heading",
                    "level": 2,
                    "text": "근거 연결표",
                },
                {
                    "type": "table",
                    "rows": [
                        ["주장", "근거 ID", "파일·좌표"],
                        ["CLAIM-1", "EV-1, EV-2", "과업지시서.hwp (section=0, paragraph=12, page=2)"],
                        ["CLAIM-2", "EV-3", "예산서_2025.hwpx (section=0, paragraph=7, page=0)"],
                    ],
                },
            ],
        },
        "placeholder_spec.json": {
            "version": "1",
            "title": "미작성",
            "blocks": [
                {
                    "type": "paragraph",
                    "text": "[CLAIM-1: 에이전트가 근거 EV-1 로 작성]",
                }
            ],
        },
        "unknown_spec.json": {
            "version": "1",
            "title": "지어낸 근거",
            "blocks": [
                {
                    "type": "paragraph",
                    "text": "시장이 성장한다. CLAIM-1 [근거: EV-99]",
                }
            ],
        },
        "unlinked_spec.json": {
            "version": "1",
            "title": "연결 없음",
            "blocks": [
                {
                    "type": "paragraph",
                    "text": "우리는 수주할 것이다. CLAIM-1",
                }
            ],
        },
    }


TRACE_TITLES = [
    "수주 엔게이지먼트 수립",
    "엔진 A 코퍼스 지도",
    "엔진 B 근거 대장",
    "엔진 C 골격",
    "CLAIM 작성",
    "validate pass",
    "validate placeholder",
    "validate unknown",
    "validate unlinked",
    "page 생략",
    "실패 문서 보존",
    "절단 공개",
    "금액 EV",
    "날짜 EV",
    "0건 절",
    "전망 거부",
    "FDE 인계",
    "Chief 인계",
    "scaffold 미광고",
    "SWS L2 정직",
    "깨진 engagement",
    "빈 questions",
    "상대 corpus",
    "command 재현",
    "재독 불일치 → 재실행",
    "cell 좌표",
    "textbox 좌표",
    "search 전패",
    "capabilities 실패",
    "explainExit 기록",
    "다중 키워드",
    "연결표 갱신",
    "no-sws-audit",
    "회신 3부",
    "문서=지시 무시",
    "금액 재계산 거부",
    "1-based page 거부",
    "부분 가독 주장 제한",
    "별도 부분집합 engagement",
    "납품 전 게이트 필수",
]


def traces() -> None:
    for i, title in enumerate(TRACE_TITLES, 1):
        dump(
            FIX / "traces" / f"T{i:02d}.json",
            {
                "schemaVersion": "1.0",
                "issue": ISSUE,
                "id": f"T{i:02d}",
                "title": title,
                "engine": ENGINE,
                "notGym": True,
                "steps": [
                    {"n": 1, "act": "read SKILL.md and playbook § relevant"},
                    {"n": 2, "act": title},
                    {"n": 3, "act": "record envelope keys only"},
                ],
                "stopMaybe": [
                    "ST-FORECAST",
                    "ST-INVENT-PAGE",
                    "ST-DROP-FAILED",
                    "ST-GATE-FAIL",
                ][i % 4],
            },
        )
    dump(
        FIX / "traces_index.json",
        {
            "schemaVersion": "1.0",
            "issue": ISSUE,
            "count": len(TRACE_TITLES),
            "ids": [f"T{i:02d}" for i in range(1, len(TRACE_TITLES) + 1)],
        },
    )


def transcripts() -> None:
    # Long-form ledger transcripts — real-looking tool output, not padding.
    ev_lines = []
    ledger = build_ledger("gov_rfp")
    ev_lines.append("# transcript reread EV-3")
    ev_lines.append("$ rhwp search corpus/smartcity-rfp/예산서_2025.hwpx --json -- 총사업비")
    ev_lines.append("{")
    ev_lines.append('  "matches": [')
    ev_lines.append('    {"text": "총사업비는 3,180백만원(부가세 별도)으로 한다.",')
    ev_lines.append('     "section": 0, "paragraph": 7, "page": 0,')
    ev_lines.append('     "charOffset": 55, "length": 11}')
    ev_lines.append("  ],")
    ev_lines.append('  "truncated": false,')
    ev_lines.append('  "totalMatchCount": 1')
    ev_lines.append("}")
    ev_lines.append("# quote matches EV quote; page present and zero-based")
    write(FIX / "transcripts" / "reread_ev3.txt", "\n".join(ev_lines))

    fail_lines = [
        "# transcript mixed corpus map",
        "$ python3 tools/strategist/engagement.py engagement.json --bin rhwp",
        "[A] 코퍼스 지도: 문서 4건",
        "[B] 근거 대장: 질문 2건",
        "[C] 산출물 골격",
        "{",
        ' "corpusDocuments": 4,',
        ' "mappedDocuments": 2,',
        ' "searchFailures": 4,',
        ' "scaffoldAdvertised": false',
        "}",
        "# failed files remain in corpus_map.json documents[]",
    ]
    write(FIX / "transcripts" / "mixed_failed_run.txt", "\n".join(fail_lines))

    gate_lines = [
        "# transcript validate fail unknown-evidence",
        "$ python3 tools/strategist/engagement.py --validate spec.json --evidence evidence.json",
        "{",
        ' "mode": "validate",',
        ' "violationCount": 1,',
        ' "violations": [',
        '  {"claim": "CLAIM-1", "kind": "unknown-evidence",',
        '   "detail": "근거 대장에 없는 id: EV-99"}',
        " ],",
        ' "verdict": "fail"',
        "}",
        "# exit 3 — do not deliver",
    ]
    write(FIX / "transcripts" / "validate_unknown.txt", "\n".join(gate_lines))

    # Full ledger dump as a readable transcript for agents.
    write(
        FIX / "transcripts" / "gov_rfp_ledger_head.txt",
        "# evidence.json head (first three EV) — full file is fixtures/ledgers/gov_rfp.json\n"
        + json.dumps({"entries": ledger["entries"][:3]}, ensure_ascii=False, indent=2)
        + "\n",
    )

    # Per-document search envelopes concatenated — teaches copy_coords.
    chunks = ["# concatenated search envelopes (do not invent page)\n"]
    for i in range(12):
        row = quote_row(i)
        env = {
            "file": f"doc{i % 5}.hwpx",
            "keyword": row["keyword"],
            "matches": [
                {
                    "text": row["text"],
                    "context": row["context"],
                    "section": row["section"],
                    "paragraph": row["paragraph"],
                    "charOffset": row["charOffset"],
                    "length": row["length"],
                    **({"page": row["page"]} if i % 3 else {}),
                }
            ],
            "truncated": False,
            "totalMatchCount": 1,
        }
        chunks.append(json.dumps(env, ensure_ascii=False, indent=2))
        chunks.append("")
    write(FIX / "transcripts" / "search_envelopes_concat.txt", "\n".join(chunks))


def extra_ledger_variants() -> None:
    # Many EV rows for contract tests that count entries.
    dump(FIX / "ledgers" / "gov_rfp.json", build_ledger("gov_rfp"))
    dump(FIX / "ledgers" / "gov_rfp_missing_page.json",
         build_ledger("gov_rfp", drop_page_on=2))
    dump(FIX / "ledgers" / "gov_rfp_truncated.json",
         build_ledger("gov_rfp", truncate=True))
    dump(FIX / "ledgers" / "quarterly.json", build_ledger("quarterly"))
    dump(FIX / "ledgers" / "quarterly_zero_q3.json",
         build_ledger("quarterly", zero_last=True))
    dump(FIX / "ledgers" / "mixed_failed.json", build_ledger("mixed_failed"))

    # 한 질문의 키워드 전개 — EV id 순서와 page 생략을 보여 주는 축소 표본.
    dense_entries = []
    n = 1
    for kw in ["필수기능", "데이터 플랫폼", "표준 API", "OpenAPI"]:
        doc = CORPORA["gov_rfp"]["docs"][(n - 1) % 2]
        row = quote_row(n)
        e = {
            "id": f"EV-{n}",
            "kind": "search",
            "question": "Q1",
            "keyword": kw,
            "file": doc,
            "section": 0,
            "paragraph": n,
            "charOffset": (n * 3) % 40,
            "length": max(2, len(kw) // 2),
            "quote": row["text"],
            "context": row["context"],
            "command": f"rhwp search corpus/smartcity-rfp/{doc} --json -- {kw}",
        }
        if n % 4 != 0:
            e["page"] = n // 2
        dense_entries.append(e)
        n += 1
    dump(
        FIX / "ledgers" / "gov_rfp_dense_q1.json",
        {
            "schemaVersion": "1",
            "generatedBy": ENGINE,
            "corpus": CORPORA["gov_rfp"]["corpus"],
            "entryCount": len(dense_entries),
            "truncatedSearches": [],
            "failures": [],
            "entries": dense_entries,
            "_skillMeta": {"id": "dense_q1", "issue": ISSUE},
        },
    )


def scenario_catalog() -> dict:
    scenarios = []
    for i, name in enumerate(listed_examples(), 1):
        if name == 'README.md':
            continue
        scenarios.append(
            {
                "id": f"S{i:02d}",
                "example": name,
                "command": "search" if i % 3 else "extract-data" if i % 2 else "info",
                "engine": ENGINE,
                "stop": None,
            }
        )
    for i in range(25, 41):
        scenarios.append(
            {
                "id": f"S{i:02d}",
                "example": None,
                "command": ALLOWED_COMMANDS[i % len(ALLOWED_COMMANDS)],
                "engine": ENGINE,
                "stop": stop_rules()["rules"][i % 15]["id"],
            }
        )
    return {
        "schemaVersion": "1.0",
        "issue": ISSUE,
        "count": len(scenarios),
        "allowedCommands": list(ALLOWED_COMMANDS),
        "scenarios": scenarios,
    }


def emit_references() -> None:
    for name, builder in REFERENCE_BUILDERS.items():
        write(REF / name, builder())


def emit_examples() -> None:
    for item in EXAMPLES:
        write(EX / item[0], example_md(item))
    write(EX / "README.md", examples_readme())


def emit_fixtures() -> None:
    dump(FIX / "catalog.json", catalog())
    dump(FIX / "skill_index.json", skill_index())
    dump(FIX / "tree.json", tree())
    dump(FIX / "stop_rules.json", stop_rules())
    dump(FIX / "envelope_keys.json", envelope_keys())
    dump(FIX / "journeys.json", journeys())
    dump(FIX / "intent_matrix.json", intent_matrix())
    dump(FIX / "pitfalls.json", pitfalls())
    dump(FIX / "handoff.json", handoff())
    dump(FIX / "scenario_catalog.json", scenario_catalog())

    for key in ("gov_rfp", "quarterly", "mixed_failed"):
        dump(FIX / "engagements" / f"{key}.json", engagement_fixture(key))
    for name, obj in invalid_engagements().items():
        dump(FIX / "engagements" / name, obj)

    extra_ledger_variants()
    for key in ("gov_rfp", "quarterly", "mixed_failed"):
        dump(FIX / "corpus_maps" / f"{key}.json", corpus_map(key))
    dump(FIX / "corpus_maps" / "all_ok.json", corpus_map("gov_rfp"))
    dump(FIX / "corpus_maps" / "all_failed.json", all_failed_map())

    for name, obj in envelopes().items():
        dump(FIX / "envelopes" / name, obj)
    for name, obj in validate_cases().items():
        dump(FIX / "validate" / name, obj)
    for name, obj in spec_samples().items():
        dump(FIX / "specs" / name, obj)

    traces()
    transcripts()


def main() -> None:
    emit_fixtures()
    print(f"wrote fixtures under {FIX}")


if __name__ == "__main__":
    main()
