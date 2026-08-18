#!/usr/bin/env python3
"""Generate distinct Korean criteria-decomp fixture rows (V-decomp).

Each NDJSON row is (task, criterionId, envelopeField, atomPass,
holisticWouldHide) bound to an existing rhwp envelope field. Rows are
unique verification tasks, not comment padding. This axis does not
rank Best-of-N and does not emit per-step rewards.
"""

from __future__ import annotations

import json
from pathlib import Path

SCHEMA = "v-decomp.1.0"
ROWS = 120_000
SHARD_SIZE = 3_000

AGENCIES = [
    "과학기술정보통신부",
    "행정안전부",
    "기획재정부",
    "법무부",
    "교육부",
    "국방부",
    "보건복지부",
    "고용노동부",
    "국토교통부",
    "환경부",
    "산업통상자원부",
    "중소벤처기업부",
    "문화체육관광부",
    "농림축산식품부",
    "해양수산부",
    "여성가족부",
    "통일부",
    "외교부",
    "국가보훈부",
    "인사혁신처",
    "국무조정실",
    "감사원",
    "공정거래위원회",
    "금융위원회",
    "개인정보보호위원회",
    "방송통신위원회",
    "원자력안전위원회",
    "국민권익위원회",
    "서울특별시",
    "부산광역시",
    "대구광역시",
    "인천광역시",
    "광주광역시",
    "대전광역시",
    "울산광역시",
    "세종특별자치시",
    "경기도",
    "강원특별자치도",
    "충청북도",
    "충청남도",
    "전북특별자치도",
    "전라남도",
    "경상북도",
    "경상남도",
    "제주특별자치도",
    "서울중앙지방법원",
    "서울고등법원",
    "특허법원",
    "헌법재판소",
    "대검찰청",
    "경찰청",
    "국세청",
    "관세청",
    "조달청",
    "통계청",
    "기상청",
    "병무청",
    "산림청",
    "특허청",
    "소방청",
    "해양경찰청",
    "질병관리청",
    "한국토지주택공사",
    "한국도로공사",
    "한국수자원공사",
    "한국전력공사",
    "국민건강보험공단",
    "근로복지공단",
    "한국연구재단",
    "한국인터넷진흥원",
]

DOC_TYPES = [
    "과업지시서",
    "제안요청서",
    "입찰공고",
    "계약서",
    "일반기안문",
    "간이기안문",
    "시행문",
    "공문",
    "훈령",
    "예규",
    "고시",
    "공고",
    "지침",
    "규정",
    "예산요구서",
    "사업계획서",
    "결과보고서",
    "감사보고서",
    "회의록",
    "출장복명서",
    "인사발령",
    "민원회신",
    "판결문",
    "결정문",
    "견적서",
    "준공검사조서",
    "보안서약서",
    "개인정보처리방침",
    "하도급계약서",
    "제안서평가기준",
]

SUBJECTS = [
    "표준 API 연계",
    "전자문서 유통",
    "본인확인 연계",
    "전자서명 검증",
    "공공마이데이터 제공",
    "온나라 문서 이관",
    "클라우드 이전",
    "정보시스템 감리",
    "개인정보 영향평가",
    "보안적합성 검증",
    "암호모듈 검증",
    "망분리 예외",
    "업무연속성 계획",
    "재해복구 센터",
    "로그 보존",
    "접근권한 통제",
    "오픈소스 라이선스 점검",
    "시험운영",
    "사용자 교육",
    "운영 이관",
    "하자보수",
    "유지관리",
    "성능시험",
    "취약점 진단",
    "가명정보 결합",
    "국고보조금 정산",
    "계약금액 조정",
    "지체상금",
    "선금 지급",
    "준공금 지급",
    "하도급 대금",
    "낙찰자 결정",
    "중대재해 예방",
    "고유식별정보 처리",
]

# (command, [(field, expect_kind)]) — fields exist on that command's envelope.
FAMILIES: list[tuple[str, list[tuple[str, str]]]] = [
    (
        "edit fill-fields",
        [
            ("filledCount", "u64"),
            ("notFound", "empty_seq"),
            ("ambiguous", "empty_seq"),
            ("verify.identical", "bool"),
            ("verify.diffCount", "u64"),
            ("dryRun", "bool"),
            ("untrustedContent", "bool"),
            ("outputFormat", "str"),
        ],
    ),
    (
        "edit replace-text",
        [
            ("replacedCount", "u64"),
            ("dryRun", "bool"),
            ("verify.identical", "bool"),
            ("verify.diffCount", "u64"),
            ("output", "present"),
            ("untrustedContent", "bool"),
        ],
    ),
    (
        "edit redact",
        [
            ("findingCount", "u64"),
            ("redactedCount", "u64"),
            ("dryRun", "bool"),
            ("inPlace", "bool"),
            ("untrustedContent", "bool"),
        ],
    ),
    (
        "edit sanitize",
        [
            ("removedCount", "u64"),
            ("keepPreview", "bool"),
            ("dryRun", "bool"),
        ],
    ),
    (
        "inspect hidden-text",
        [
            ("clean", "bool"),
            ("hiddenCharCount", "u64"),
            ("untrustedContent", "bool"),
        ],
    ),
    (
        "inspect injection",
        [
            ("clean", "bool"),
            ("signalCount", "u64"),
            ("highestConfidence", "str"),
            ("untrustedContent", "bool"),
        ],
    ),
    (
        "inspect unicode",
        [
            ("clean", "bool"),
            ("findingCount", "u64"),
            ("scannedChars", "u64"),
            ("untrustedContent", "bool"),
        ],
    ),
    (
        "threat-scan",
        [
            ("clean", "bool"),
            ("findingCount", "u64"),
            ("highestSeverity", "str"),
        ],
    ),
    (
        "ir-diff",
        [
            ("identical", "bool"),
            ("diffCount", "u64"),
            ("untrustedContent", "bool"),
            ("schemaVersion", "present"),
        ],
    ),
    (
        "render-diff",
        [
            ("status", "str"),
            ("regression", "bool"),
            ("overPages", "u64"),
            ("pageCountMismatch", "bool"),
            ("maxDisp", "u64"),
            ("worstPage", "u64"),
        ],
    ),
    (
        "layout-anomaly",
        [
            ("hasSignal", "bool"),
            ("overflowCount", "u64"),
            ("overlapCount", "u64"),
            ("textOverlapCount", "u64"),
            ("emptyPageCount", "u64"),
            ("offCanvasCount", "u64"),
            ("strict", "bool"),
        ],
    ),
    (
        "search",
        [
            ("truncated", "bool"),
            ("matches", "present"),
            ("untrustedContent", "bool"),
        ],
    ),
    (
        "extract-data",
        [
            ("truncated", "bool"),
            ("items", "present"),
            ("untrustedContent", "bool"),
        ],
    ),
    (
        "info",
        [
            ("pageCount", "u64"),
            ("paraCount", "u64"),
            ("format", "str"),
            ("encrypted", "bool"),
            ("warnings", "empty_seq"),
        ],
    ),
    (
        "word-count",
        [
            ("charCount", "u64"),
            ("wordCount", "u64"),
            ("sectionCount", "u64"),
            ("paragraphCount", "u64"),
        ],
    ),
    (
        "export-hwpx",
        [
            ("verify.identical", "bool"),
            ("verify.diffCount", "u64"),
            ("wasDistribution", "bool"),
        ],
    ),
    (
        "convert",
        [
            ("wasDistribution", "bool"),
            ("verify.identical", "bool"),
            ("verify.diffCount", "u64"),
        ],
    ),
    (
        "verify",
        [
            ("passCount", "u64"),
            ("failCount", "u64"),
            ("verdict", "str"),
        ],
    ),
    (
        "fields",
        [
            ("fieldCount", "u64"),
            ("untrustedContent", "bool"),
        ],
    ),
    (
        "form-value",
        [
            ("ok", "bool"),
        ],
    ),
    (
        "replay",
        [
            ("valid", "bool"),
            ("reproduced", "bool"),
        ],
    ),
    (
        "verify-signature",
        [
            ("signatureOk", "bool"),
            ("capsuleShaMatches", "bool"),
            ("keyKnown", "bool"),
            ("verdict", "str"),
        ],
    ),
    (
        "csv-to-table",
        [
            ("dryRun", "bool"),
            ("verify.identical", "bool"),
            ("verify.diffCount", "u64"),
        ],
    ),
]

INVENTED = [
    "holisticScore",
    "overall",
    "quality",
    "stars",
    "grade",
    "confidence",
    "vibe",
    "humanPage",
    "pdfPage",
    "bestOfN",
    "processReward",
    "stepReward",
    "rank",
    "winner",
    "llmScore",
]

FIELD_KO = {
    "filledCount": "채운 누름틀 수",
    "notFound": "없는 필드 이름 목록",
    "ambiguous": "모호한 필드 이름 목록",
    "verify.identical": "자기검증 identical",
    "verify.diffCount": "자기검증 차이 수",
    "dryRun": "미리보기 여부",
    "untrustedContent": "문서 파생 표지",
    "outputFormat": "산출 형식",
    "replacedCount": "치환 건수",
    "output": "저장 경로",
    "findingCount": "탐지 개수",
    "redactedCount": "마스킹 개수",
    "inPlace": "원본 덮어쓰기",
    "removedCount": "제거한 메타데이터 수",
    "keepPreview": "미리보기 그림 유지",
    "clean": "탐지 0건 요약",
    "hiddenCharCount": "은닉 문자 수",
    "signalCount": "주입 신호 수",
    "highestConfidence": "최고 신뢰도",
    "scannedChars": "검사한 문자 수",
    "highestSeverity": "최고 심각도",
    "identical": "IR 동일 여부",
    "diffCount": "차이 개수",
    "schemaVersion": "봉투 스키마 버전",
    "status": "시각 회귀 상태",
    "regression": "시각 회귀 여부",
    "overPages": "임계 초과 쪽 수",
    "pageCountMismatch": "쪽 수 불일치",
    "maxDisp": "최대 변위",
    "worstPage": "최악 쪽",
    "hasSignal": "조판 이상 신호",
    "overflowCount": "넘침 신호 수",
    "overlapCount": "겹침 신호 수",
    "textOverlapCount": "글자 겹침 수",
    "emptyPageCount": "빈 쪽 신호 수",
    "offCanvasCount": "캔버스 밖 수",
    "strict": "엄격 모드",
    "truncated": "잘림 여부",
    "matches": "검색 매치 배열",
    "items": "추출 항목 배열",
    "pageCount": "쪽 수",
    "paraCount": "문단 수",
    "format": "문서 형식",
    "encrypted": "암호화 여부",
    "warnings": "파싱 경고",
    "charCount": "글자 수",
    "wordCount": "어절 수",
    "sectionCount": "구역 수",
    "paragraphCount": "문단 수",
    "wasDistribution": "배포용 입력이었는지",
    "passCount": "만족한 기대 수",
    "failCount": "불만족 기대 수",
    "verdict": "요약 판정 문자열",
    "fieldCount": "누름틀 총수",
    "ok": "양식 값 읽기 성공",
    "valid": "재현 유효",
    "reproduced": "재현 여부",
    "signatureOk": "서명 암호 검증",
    "capsuleShaMatches": "캡슐 해시 일치",
    "keyKnown": "키 등록 여부",
}

INVENTED_KO = {
    "holisticScore": "한 덩어리 점수",
    "overall": "종합 인상",
    "quality": "품질 감점",
    "stars": "별점",
    "grade": "등급",
    "confidence": "모델 자신감",
    "vibe": "분위기",
    "humanPage": "사람이 센 쪽",
    "pdfPage": "PDF 쪽",
    "bestOfN": "Best-of-N 순위",
    "processReward": "과정 보상",
    "stepReward": "단계 보상",
    "rank": "순위",
    "winner": "승자 후보",
    "llmScore": "모델 총점",
}


def hangul_year(i: int) -> int:
    return 2018 + (i % 9)


def doc_no(i: int, agency: str, doc: str) -> str:
    return f"{len(agency):02d}{len(doc):02d}-{hangul_year(i)}-{i + 1:06d}"


def field_slug(field: str) -> str:
    return field.replace(".", "_")


def expected_of(kind: str, i: int, pass_row: bool) -> tuple[dict, object | None]:
    """Return (expected, observed) for a passing or mismatching atom."""
    if kind == "bool":
        want = i % 2 == 0
        exp = {"kind": "bool", "value": want}
        if pass_row:
            return exp, want
        return exp, (not want)
    if kind == "u64":
        want = (i * 7) % 40
        exp = {"kind": "u64", "value": want}
        if pass_row:
            return exp, want
        return exp, want + 1 + (i % 5)
    if kind == "empty_seq":
        exp = {"kind": "empty_seq"}
        if pass_row:
            return exp, []
        return exp, [f"누름틀{(i % 17) + 1:02d}"]
    if kind == "present":
        exp = {"kind": "present"}
        if pass_row:
            return exp, "있음"
        return exp, None
    if kind == "str":
        catalog = {
            "outputFormat": ("hwp5", "hwpx"),
            "highestConfidence": ("high", "low"),
            "highestSeverity": ("high", "medium"),
            "status": ("OK", "OVER"),
            "format": ("hwp5", "hwpx"),
            "verdict": ("pass", "fail"),
        }
        # default pair
        good, bad = "pass", "fail"
        for key, pair in catalog.items():
            # caller does not pass field; keep generic but distinct
            good, bad = pair
            break
        # pick by i so strings vary
        pairs = list(catalog.values())
        good, bad = pairs[i % len(pairs)]
        exp = {"kind": "str_eq", "value": good}
        if pass_row:
            return exp, good
        return exp, bad
    exp = {"kind": "present"}
    return exp, "있음" if pass_row else None


def expected_for_field(field: str, kind: str, i: int, pass_row: bool) -> tuple[dict, object | None]:
    if kind == "str":
        catalog = {
            "outputFormat": ("hwp5", "hwpx"),
            "highestConfidence": ("high", "low"),
            "highestSeverity": ("high", "medium"),
            "status": ("OK", "OVER"),
            "format": ("hwp5", "hwpx"),
            "verdict": ("pass", "fail"),
        }
        good, bad = catalog.get(field, ("ok", "fail"))
        exp = {"kind": "str_eq", "value": good}
        return exp, (good if pass_row else bad)
    return expected_of(kind, i, pass_row)


def make_row(i: int) -> dict:
    agency = AGENCIES[i % len(AGENCIES)]
    doc = DOC_TYPES[(i // 5) % len(DOC_TYPES)]
    subj = SUBJECTS[(i * 7) % len(SUBJECTS)]
    year = hangul_year(i)
    dno = doc_no(i, agency, doc)
    ext = ".hwpx" if i % 2 == 0 else ".hwp"
    file_name = f"{agency}_{doc}_{dno}{ext}"
    fam_i = i % len(FAMILIES)
    command, fields = FAMILIES[fam_i]
    field, kind = fields[(i // len(FAMILIES)) % len(fields)]
    slot = i % 20
    crit = f"C-{command.replace(' ', '_')}-{field_slug(field)}-{i + 1:06d}"
    ko = FIELD_KO.get(field, field)
    task = (
        f"{agency} {year}년도 {doc}({dno})의 {subj} 과업을 `{command}` 로 검증할 때 "
        f"한 덩어리 점수 대신 기존 봉투 필드 `{field}`({ko}) 를 원자 기준으로 본다. "
        f"대상 파일은 {file_name} 이다."
    )

    row: dict = {
        "rowId": f"CD-{i + 1:06d}",
        "criterionId": crit,
        "envelopeField": field,
        "command": command,
        "file": file_name,
        "holisticOnly": False,
    }

    if slot <= 11:
        exp, obs = expected_for_field(field, kind, i, True)
        total = 3 + (i % 5)
        pass_n = total
        row.update(
            {
                "task": task,
                "atomPass": True,
                "holisticWouldHide": False,
                "bundlePassCount": pass_n,
                "bundleTotal": total,
                "expected": exp,
                "observed": obs,
            }
        )
        return row

    if slot <= 14:
        exp, obs = expected_for_field(field, kind, i, False)
        total = 5
        pass_n = 4
        fail_kind = "missing_field" if obs is None else "atom_mismatch"
        row.update(
            {
                "task": task + " 총점은 형제 원자 네 개가 통과해 이 실패를 가린다.",
                "atomPass": False,
                "holisticWouldHide": True,
                "bundlePassCount": pass_n,
                "bundleTotal": total,
                "expected": exp,
                "failKind": fail_kind,
            }
        )
        if obs is not None:
            row["observed"] = obs
        return row

    if slot <= 16:
        exp, obs = expected_for_field(field, kind, i, False)
        total = 5
        pass_n = 1
        fail_kind = "missing_field" if obs is None else "atom_mismatch"
        row.update(
            {
                "task": task + " 묶음 대부분이 실패해 총점도 실패를 숨기지 못한다.",
                "atomPass": False,
                "holisticWouldHide": False,
                "bundlePassCount": pass_n,
                "bundleTotal": total,
                "expected": exp,
                "failKind": fail_kind,
            }
        )
        if obs is not None:
            row["observed"] = obs
        return row

    if slot == 17:
        inv = INVENTED[i % len(INVENTED)]
        inv_ko = INVENTED_KO[inv]
        row.update(
            {
                "task": (
                    f"{agency} {year}년도 {doc}({dno})의 {subj} 검증에서 "
                    f"봉투에 없는 `{inv}`({inv_ko}) 를 기준으로 삼으려 한다. "
                    f"파일은 {file_name} 이며 이 키는 지식지도에 없다."
                ),
                "criterionId": f"C-invented-{inv}-{i + 1:06d}",
                "envelopeField": inv,
                "atomPass": False,
                "holisticWouldHide": False,
                "bundlePassCount": 0,
                "bundleTotal": 1,
                "expected": {"kind": "bool", "value": True},
                "observed": True,
                "failKind": "invented_field",
            }
        )
        return row

    if slot == 18:
        exp, _ = expected_for_field(field, kind, i, True)
        if exp["kind"] in {"absent", "empty_seq"}:
            exp = {"kind": "bool", "value": True}
        row.update(
            {
                "task": task + " 봉투에 해당 필드가 빠져 원자 기준을 읽을 수 없다.",
                "atomPass": False,
                "holisticWouldHide": True,
                "bundlePassCount": 3,
                "bundleTotal": 4,
                "expected": exp,
                "failKind": "missing_field",
            }
        )
        return row

    # slot 19 — empty task or holistic-only
    if (i // 20) % 2 == 0:
        row.update(
            {
                "task": "",
                "atomPass": False,
                "holisticWouldHide": False,
                "bundlePassCount": 0,
                "bundleTotal": 1,
                "expected": {"kind": "bool", "value": True},
                "observed": True,
                "failKind": "empty_task",
            }
        )
        return row

    row.update(
        {
            "task": (
                f"{agency} {year}년도 {doc}({dno})의 {subj} 을 "
                f"원자 없이 총점 0.{70 + (i % 29):02d} 한 줄로만 채점하려 한다. "
                f"파일은 {file_name} 이다."
            ),
            "criterionId": f"C-holistic_only-{i + 1:06d}",
            "envelopeField": "verdict",
            "atomPass": False,
            "holisticWouldHide": False,
            "bundlePassCount": 0,
            "bundleTotal": 1,
            "expected": {"kind": "str_eq", "value": "pass"},
            "observed": "pass",
            "failKind": "holistic_only",
            "holisticOnly": True,
        }
    )
    return row


def write_shards(out_dir: Path) -> dict:
    out_dir.mkdir(parents=True, exist_ok=True)
    for old in out_dir.glob("shard_*.ndjson"):
        old.unlink()

    shards = []
    pass_n = 0
    fail_n = 0
    hidden_n = 0
    shard_idx = 0
    buf: list[dict] = []
    shard_pass = 0
    shard_hidden = 0

    def flush() -> None:
        nonlocal shard_idx, buf, shard_pass, shard_hidden
        if not buf:
            return
        name = f"shard_{shard_idx:02d}.ndjson"
        path = out_dir / name
        with path.open("w", encoding="utf-8", newline="\n") as fh:
            for row in buf:
                fh.write(json.dumps(row, ensure_ascii=False, separators=(",", ":")))
                fh.write("\n")
        shards.append(
            {
                "path": name,
                "count": len(buf),
                "atomPassCount": shard_pass,
                "hiddenFailCount": shard_hidden,
            }
        )
        shard_idx += 1
        buf = []
        shard_pass = 0
        shard_hidden = 0

    seen_task: set[str] = set()
    seen_crit: set[str] = set()
    seen_tuple: set[tuple] = set()

    for i in range(ROWS):
        row = make_row(i)
        task = row["task"]
        if task:
            if task in seen_task:
                raise SystemExit(f"duplicate task at {i}")
            seen_task.add(task)
        if row["criterionId"] in seen_crit:
            raise SystemExit(f"duplicate criterionId at {i}")
        seen_crit.add(row["criterionId"])
        key = (
            row["task"],
            row["criterionId"],
            row["envelopeField"],
            row["atomPass"],
            row["holisticWouldHide"],
        )
        if key in seen_tuple:
            raise SystemExit(f"duplicate tuple at {i}")
        seen_tuple.add(key)

        if row["atomPass"]:
            pass_n += 1
            shard_pass += 1
        else:
            fail_n += 1
        if row["holisticWouldHide"]:
            hidden_n += 1
            shard_hidden += 1
        buf.append(row)
        if len(buf) >= SHARD_SIZE:
            flush()
    flush()

    manifest = {
        "schemaVersion": SCHEMA,
        "generatedBy": "tools/llm_verifier/criteria_decomp/scripts/gen_decomp_corpus.py",
        "axis": "criteria-decomp",
        "recordCount": ROWS,
        "shardCount": len(shards),
        "atomPassCount": pass_n,
        "atomFailCount": fail_n,
        "hiddenFailCount": hidden_n,
        "uniqueness": "rowId+criterionId+task+(task,criterionId,envelopeField,atomPass,holisticWouldHide)",
        "tupleFields": [
            "task",
            "criterionId",
            "envelopeField",
            "atomPass",
            "holisticWouldHide",
        ],
        "shards": shards,
    }
    (out_dir / "manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    return manifest


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    out = root / "fixtures" / "corpus"
    man = write_shards(out)
    print(
        f"wrote {man['recordCount']} rows in {man['shardCount']} shards "
        f"(pass={man['atomPassCount']} fail={man['atomFailCount']} "
        f"hidden={man['hiddenFailCount']}) -> {out}"
    )


if __name__ == "__main__":
    main()
