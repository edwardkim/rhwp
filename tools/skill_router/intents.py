#!/usr/bin/env python3
"""Classify a Korean/English user request into one rhwp skill intent.

More specific intents beat overlapping generic ones (form-fill over cli).
A request never maps to two skills that share a responsibility.
"""

from __future__ import annotations

import re
from typing import Any

# Higher specificity wins when two intents both match.
INTENT_SPECS: tuple[dict[str, Any], ...] = (
    {
        "id": "contribute",
        "label": "기여·PR",
        "skill": "rhwp-contributor",
        "capability": "rhwp-contributor",
        "specificity": 100,
        "patterns": (
            r"pr\s*올려",
            r"피알\s*올려",
            r"pull\s*request",
            r"\bgh\s+pr\b",
            r"open\s+(a\s+)?pr\b",
            r"create\s+(a\s+)?pr\b",
            r"submit\s+(a\s+)?pr\b",
            r"make\s+(a\s+)?pr\b",
            r"기여",
            r"contribut",
            r"이슈\s*만들",
            r"이슈\s*생성",
            r"이슈\s*올리고",
            r"create\s+(an\s+)?issue",
            r"open\s+(an\s+)?issue",
            r"make\s+(an\s+)?issue",
            r"버그\s*고쳐서\s*제출",
            r"기여\s*절차",
            r"upstream/devel",
            r"closes\s*#",
        ),
    },
    {
        "id": "fill-form",
        "label": "서식 채움",
        "skill": "rhwp-form-fill",
        "capability": "rhwp-form-fill",
        "specificity": 95,
        "patterns": (
            r"서식\s*채",
            r"양식\s*채",
            r"신청서\s*채",
            r"이\s*서식",
            r"누름틀",
            r"fill-fields",
            r"fill\s+(this\s+|the\s+|out\s+(this\s+|the\s+)?)form",
            r"form\s*fill",
            r"메일\s*머지",
            r"mail\s*merge",
            r"명단으로",
            r"n명분",
            r"몇\s*명분",
            r"필드에\s*값",
            r"값\s*채워",
            r"채워\s*줘",
            r"제출용으로\s*만들",
            r"fields\s+.+\s*--json",
            r"batch\s+fill",
        ),
    },
    {
        "id": "exam-ingest",
        "label": "시험지 수집",
        "skill": "rhwp-exam-ingest",
        "capability": "rhwp-exam-ingest",
        "specificity": 92,
        "patterns": (
            r"시험지",
            r"시험문제",
            r"exam\s*ingest",
            r"build-from-ingest",
            r"한글\s*시험지",
            r"ingest\.json",
            r"pdf.*hwpx.*시험",
            r"시험.*hwpx",
        ),
    },
    {
        "id": "table-csv",
        "label": "표↔CSV",
        "skill": "rhwp-table-exchange",
        "capability": "rhwp-table-exchange",
        "specificity": 90,
        "patterns": (
            r"표를?\s*csv",
            r"csv\s*로\s*뽑",
            r"table-to-csv",
            r"csv-to-table",
            r"엑셀로\s*뽑",
            r"표\s*왕복",
            r"표↔",
            r"표\s*<->",
            r"export-tables",
            r"표\s*셀",
            r"셀\s*(값|하나|만)",
            r"set-cell",
            r"스프레드시트",
            r"csv\s*를\s*(문서\s*)?표",
            r"tables?\s*to\s*csv",
            r"csv\s*round\s*trip",
        ),
    },
    {
        "id": "provenance",
        "label": "출처 표지",
        "skill": "rhwp-provenance",
        "capability": "rhwp-provenance",
        "specificity": 88,
        "patterns": (
            r"untrustedcontent",
            r"untrustedfields",
            r"export-provenance-map",
            r"출처\s*표지",
            r"문서에서\s*온",
            r"프롬프트\s*주입\s*방어",
            r"\barmor\b",
            r"신뢰할\s*수\s*없는",
            r"출처\s*모르는\s*문서\s*처리",
            r"문서\s*텍스트를\s*llm",
        ),
    },
    {
        "id": "receipt",
        "label": "작업 영수증",
        "skill": "rhwp-work-receipt",
        "capability": "rhwp-work-receipt",
        "specificity": 86,
        "patterns": (
            r"영수증",
            r"\breplay\b",
            r"작업\s*캡슐",
            r"capsule",
            r"\baudit\b",
            r"\blineage\b",
            r"재현율",
            r"work\s*receipt",
            r"작업\s*증명",
            r"계보\s*검증",
            r"--capsule",
            r"--parent",
        ),
    },
    {
        "id": "visual",
        "label": "시각 회귀",
        "skill": "rhwp-visual-regression",
        "capability": "rhwp-visual-regression",
        "specificity": 84,
        "patterns": (
            r"render-diff",
            r"레이아웃\s*회귀",
            r"시각\s*(회귀|검증)",
            r"편집\s*전후\s*(화면|비교)",
            r"visual\s*regression",
            r"struct_mismatch",
            r"화면\s*비교",
            r"라운드트립\s*시각",
            r"바뀐\s*게\s*의도",
            r"geom_inventory",
        ),
    },
    {
        "id": "security",
        "label": "보안 스윕",
        "skill": "rhwp-security-sweep",
        "capability": "rhwp-security-sweep",
        "specificity": 82,
        "patterns": (
            r"보내도\s*돼",
            r"배포\s*전",
            r"숨긴\s*텍스트",
            r"hidden-text",
            r"inspect\s+injection",
            r"inspect\s+unicode",
            r"inspect\s+hidden",
            r"개인정보\s*마스킹",
            r"\bredact\b",
            r"받은\s*첨부",
            r"메타데이터\s*지워",
            r"sanitize",
            r"주입\s*검사",
            r"유니코드\s*검사",
            r"안전한지\s*확인",
            r"security\s*sweep",
        ),
    },
    {
        "id": "mcp",
        "label": "MCP 세션",
        "skill": "rhwp-mcp-session",
        "capability": "rhwp-mcp-session",
        "specificity": 80,
        "patterns": (
            r"mcp-serve",
            r"mcp\s*로\s*붙",
            r"mcp로\s*등록",
            r"hwp_open",
            r"hwp_doc_",
            r"hwp_close",
            r"세션으로\s*문서",
            r"tools/list",
            r"capabilities\s+--mcp",
            r"재파싱\s*없이",
            r"mcp\s*세션",
            r"mcp\s*server",
        ),
    },
    {
        "id": "bulk",
        "label": "대량 처리",
        "skill": "rhwp-bulk-pipeline",
        "capability": "rhwp-bulk-pipeline",
        "specificity": 78,
        "patterns": (
            r"폴더\s*전체",
            r"대량\s*처리",
            r"rhwp\s+batch",
            r"\bbatch\s+(info|export-|search|convert|fields|extract)",
            r"한꺼번에\s*(변환|처리)",
            r"코퍼스",
            r"아카이브\s*전역",
            r"수백\s*건",
            r"여러\s*hwp",
            r"bulk\s*pipeline",
            r"stdin.*ndjson",
        ),
    },
    {
        "id": "safe-edit",
        "label": "안전 편집",
        "skill": "rhwp-safe-edit",
        "capability": "rhwp-safe-edit",
        "specificity": 70,
        "patterns": (
            r"안전하게\s*편집",
            r"safe\s*edit",
            r"dry-run으로\s*먼저",
            r"run\s*계획서",
            r"replace-text",
            r"문구\s*일괄\s*치환",
            r"체크박스",
            r"여러\s*편집을\s*한\s*번에",
            r"원자적",
            r"원본\s*(훼손\s*없이|불변)",
            r"export-plan-schema",
            r"\brhwp\s+run\b",
        ),
    },
    {
        "id": "onboard",
        "label": "온보딩",
        "skill": "rhwp-onboarding",
        "capability": "rhwp-onboarding",
        "specificity": 65,
        "patterns": (
            r"온보딩",
            r"onboard",
            r"rhwp\s*처음",
            r"처음\s*만나",
            r"설치",
            r"셋업",
            r"setup",
            r"부트스트랩",
            r"bootstrap",
            r"rhwp_doctor",
            r"돌아가는지\s*확인",
            r"뭐부터",
            r"어떻게\s*(시작|붙여)",
            r"\.mcp\.json\s*만들",
            r"에이전트\s*온보딩",
        ),
    },
    {
        "id": "triage",
        "label": "문서 트리아지",
        "skill": "rhwp-doc-triage",
        "capability": "rhwp-doc-triage",
        "specificity": 60,
        "patterns": (
            r"뭔\s*문서",
            r"무슨\s*문서",
            r"이\s*hwp",
            r"이\s*파일\s*뭐",
            r"내용\s*요약",
            r"목차\s*뽑",
            r"어디에.{0,12}나와",
            r"날짜.{0,6}금액",
            r"긴\s*문서",
            r"다\s*읽지\s*말고",
            r"파악해",
            r"\bdigest\b",
            r"\bexplain\b",
            r"export-structure",
            r"extract-data",
            r"what\s+is\s+this\s+(hwp|hwpx|document)",
            r"summarize\s+(this\s+)?(hwp|document)",
            r"doc\s*triage",
        ),
    },
    {
        "id": "inspect-cli",
        "label": "CLI 분석·내보내기",
        "skill": "rhwp-cli",
        "capability": "rhwp-cli",
        "specificity": 40,
        "patterns": (
            r"export-svg",
            r"export-png",
            r"export-pdf",
            r"export-text",
            r"export-markdown",
            r"dump-pages",
            r"dump-records",
            r"\bdump\b",
            r"\bdiag\b",
            r"export-render-tree",
            r"페이지네이션",
            r"조판부호",
            r"render\s*tree",
            r"hwp5-inventory",
            r"레이아웃.{0,8}버그",
            r"겹침",
            r"svg/png/pdf",
            r"텍스트로\s*내보내",
            r"png로\s*내보내",
            r"pdf로\s*내보내",
            r"svg로\s*내보내",
            r"\brhwp\s+(info|convert|thumbnail)\b",
        ),
    },
    {
        "id": "codex",
        "label": "명령 교본",
        "skill": "rhwp-codex",
        "capability": "rhwp-codex",
        "specificity": 30,
        "patterns": (
            r"코덱스",
            r"\bcodex\b",
            r"명령\s*교본",
            r"에이전트\s*대전",
            r"뭘\s*쓸지\s*모르겠",
            r"전체\s*명령",
            r"사용법",
            r"capabilities\s+--search",
            r"agent_codex",
            r"대전\s*재생성",
            r"문서\s*신선도",
        ),
    },
)

_COMPILED: list[tuple[dict[str, Any], tuple[re.Pattern[str], ...]]] = [
    (spec, tuple(re.compile(pat, re.IGNORECASE) for pat in spec["patterns"]))
    for spec in INTENT_SPECS
]

INTENT_IDS: tuple[str, ...] = tuple(spec["id"] for spec in INTENT_SPECS)

_FALLBACK = {
    "id": "codex",
    "label": "명령 교본",
    "skill": "rhwp-codex",
    "capability": "rhwp-codex",
    "specificity": 0,
    "reason": "구체 스킬에 못 붙여 명령 교본으로 항해한다",
}


def _score(text: str, compiled: tuple[re.Pattern[str], ...]) -> int:
    return sum(1 for pat in compiled if pat.search(text))


def classify(text: str) -> dict[str, Any]:
    """Return the winning intent. Keys: id, label, confidence, skill, capability, reason."""
    raw = (text or "").strip()
    if not raw:
        raise ValueError("empty request")

    scored: list[tuple[int, dict[str, Any]]] = []
    for spec, compiled in _COMPILED:
        hits = _score(raw, compiled)
        if hits:
            scored.append((hits, spec))

    if not scored:
        result = dict(_FALLBACK)
        result["confidence"] = 0.35
        return result

    # Any match enters the pool; the more specific skill wins (form-fill over cli).
    scored.sort(key=lambda item: (item[1]["specificity"], item[0]), reverse=True)
    hits, spec = scored[0]
    confidence = min(0.99, 0.62 + 0.08 * hits + 0.002 * spec["specificity"])
    return {
        "id": spec["id"],
        "label": spec["label"],
        "confidence": round(confidence, 3),
        "skill": spec["skill"],
        "capability": spec["capability"],
        "specificity": spec["specificity"],
        "reason": (
            f"{spec['label']} 요청이므로 {spec['skill']} 을(를) 선택한다"
            " (겹치면 더 구체적인 스킬)"
        ),
    }
