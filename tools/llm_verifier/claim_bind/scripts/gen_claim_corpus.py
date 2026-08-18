#!/usr/bin/env python3
"""Generate distinct Korean claim-bind fixture rows (axis 3).

Each NDJSON row is (claimText, coordsPresent, fieldSet, verdict) copied from
the existing search / extract-data envelope coordinate key set. Rows are
unique document claims, not comment padding.
"""

from __future__ import annotations

import json
from pathlib import Path

SCHEMA = "v-bind.1.0"
ROWS = 120_000
SHARD_SIZE = 3_000
REQUIRED = ("section", "paragraph", "page", "charOffset")

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
    "새만금개발청",
    "국가유산청",
    "한국지능정보사회진흥원",
    "한국인터넷진흥원",
    "한국정보화진흥원",
    "한국산업기술진흥원",
    "한국연구재단",
    "한국과학기술기획평가원",
    "한국교육학술정보원",
    "국민건강보험공단",
    "국민연금공단",
    "근로복지공단",
    "한국토지주택공사",
    "한국도로공사",
    "한국수자원공사",
    "한국전력공사",
    "한국가스공사",
    "한국공항공사",
    "인천국제공항공사",
    "서울대학교",
    "한국과학기술원",
    "포항공과대학교",
    "한국은행",
    "예금보험공사",
    "신용보증기금",
    "기술보증기금",
    "대한상공회의소",
    "중소기업중앙회",
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
    "시행령",
    "법률안",
    "예산요구서",
    "사업설명자료",
    "사업계획서",
    "연구계획서",
    "결과보고서",
    "중간보고서",
    "감사보고서",
    "감리보고서",
    "회의록",
    "이사회의사록",
    "주주총회의사록",
    "출장명령서",
    "출장복명서",
    "인사발령",
    "징계의결서",
    "민원회신",
    "질의회신",
    "유권해석",
    "판결문",
    "결정문",
    "고소장",
    "답변서",
    "준비서면",
    "내용증명",
    "내용증명회신",
    "견적서",
    "거래명세서",
    "세금계산서",
    "대금청구서",
    "준공계",
    "준공검사조서",
    "안전관리계획서",
    "환경영향평가서",
    "건축허가신청서",
    "착공신고서",
    "사용승인서",
    "학교생활기록부",
    "성적증명서",
    "진단서",
    "소견서",
    "처방전",
    "보험약관",
    "특별약관",
    "재무제표주석",
    "내부회계관리규정",
    "개인정보처리방침",
    "정보공개결정통지",
    "보안서약서",
    "비밀유지약정",
    "용역계약특수조건",
    "물품구매계약특수조건",
    "하도급계약서",
    "공동수급협정",
    "기술평가표",
    "적격심사세부기준",
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
    "소스코드 반출",
    "시험운영",
    "사용자 교육",
    "운영 이관",
    "하자보수",
    "유지관리",
    "성능시험",
    "부하시험",
    "취약점 진단",
    "모의침투",
    "개인정보 암호화",
    "고유식별정보 처리",
    "주민등록번호 수집",
    "위치정보 이용",
    "가명정보 결합",
    "공공데이터 개방",
    "공간정보 갱신",
    "주소정보 정비",
    "도로명주소 부여",
    "지적공부 정리",
    "토지보상",
    "수용재결",
    "도시계획시설",
    "개발행위허가",
    "건축선 지정",
    "에너지 사용량 보고",
    "온실가스 감축",
    "폐기물 처리",
    "대기배출시설",
    "수질오염물질",
    "화학물질 통계",
    "산업안전보건",
    "위험성평가",
    "중대재해 예방",
    "근로시간 단축",
    "임금체불 예방",
    "장애인 고용",
    "청년인턴",
    "경력단절여성",
    "보육지원",
    "기초생활보장",
    "긴급복지",
    "감염병 대응",
    "예방접종",
    "의약품 공급",
    "의료급여",
    "국민연금 가입",
    "고용보험 피보험",
    "산재보험 적용",
    "관세환급",
    "부가가치세 환급",
    "국고보조금 정산",
    "예산 전용",
    "예비비 사용",
    "계속비 이월",
    "명시이월",
    "사고이월",
    "계약금액 조정",
    "물가변동",
    "설계변경",
    "공기연장",
    "지체상금",
    "선금 지급",
    "기성금 지급",
    "준공금 지급",
    "하자담보",
    "계약보증",
    "선금보증",
    "하도급 대금",
    "공동수급 지분",
    "기술평가 배점",
    "가격평가 배점",
    "제안서 발표",
    "협상 적격자",
    "낙찰자 결정",
    "입찰무효",
    "담합 징후",
    "부정당업자",
    "과징금 부과",
    "시정명령",
    "영업정지",
    "과태료",
    "이행강제금",
    "행정심판",
    "행정소송",
    "집행정지",
    "손해배상",
    "부당이득 반환",
    "채무부존재",
    "소유권이전",
    "근저당권 설정",
    "전세권 설정",
    "가압류",
    "가처분",
    "강제집행",
    "화해권고",
    "조정 성립",
    "화해 권고조항",
]

OBLIGATIONS = [
    "필수기능으로 정한다",
    "우선 적용한다",
    "예외 없이 이행한다",
    "착수 전 승인을 받는다",
    "완료 후 검사를 받는다",
    "매월 실적을 보고한다",
    "분기별 점검을 실시한다",
    "변경 시 사전 협의한다",
    "원본을 훼손하지 않는다",
    "개인정보를 목적 외 이용하지 않는다",
    "하도급을 원칙적으로 금지한다",
    "공동수급 지분을 준수한다",
    "보안서약을 제출한다",
    "산출물을 발주기관에 귀속한다",
    "오픈소스 목록을 첨부한다",
    "시험 시나리오를 제출한다",
    "운영 매뉴얼을 이관한다",
    "장애 대응 체계를 갖춘다",
    "백업을 일 1회 수행한다",
    "접속 기록을 1년 이상 보관한다",
    "암호화 키를 분리 보관한다",
    "원격 접속을 통제한다",
    "반출 자료를 암호화한다",
    "검수 기준을 충족한다",
    "품질 목표를 달성한다",
    "일정 지연 시 만회계획을 제출한다",
    "이해충돌을 신고한다",
    "청렴계약을 준수한다",
    "안전보건 조치를 이행한다",
    "환경 법령을 준수한다",
]

PURPOSES = [
    "정보화사업 구축비",
    "시스템 고도화비",
    "클라우드 이용료",
    "소프트웨어 라이선스",
    "하드웨어 임차료",
    "감리비",
    "보안컨설팅비",
    "교육훈련비",
    "시험운영비",
    "유지관리비",
    "예비비",
    "부가가치세",
    "손해배상 예정액",
    "지체상금 한도",
    "계약보증금",
    "선금",
    "1차 기성금",
    "2차 기성금",
    "준공금",
    "하자보수보증금",
    "토지보상비",
    "이주대책비",
    "설계비",
    "공사비",
    "감리비 정산",
    "안전관리비",
    "보험료",
    "법정복리후생비",
    "일반관리비",
    "이윤",
]

COURTS = [
    "서울중앙지방법원",
    "서울동부지방법원",
    "서울서부지방법원",
    "서울남부지방법원",
    "서울북부지방법원",
    "수원지방법원",
    "인천지방법원",
    "대전지방법원",
    "대구지방법원",
    "부산지방법원",
    "광주지방법원",
    "춘천지방법원",
    "청주지방법원",
    "전주지방법원",
    "창원지방법원",
    "제주지방법원",
    "특허법원",
    "서울고등법원",
    "대전고등법원",
    "대구고등법원",
    "부산고등법원",
    "광주고등법원",
    "수원고등법원",
]

CASE_KINDS = [
    "가합",
    "가단",
    "가소",
    "나",
    "다",
    "르",
    "카합",
    "카단",
    "카기",
    "즈합",
    "즈단",
    "구합",
    "구단",
    "행합",
    "행단",
    "아",
    "카확",
    "타기",
    "초기",
    "머",
]

EXT = (".hwp", ".hwpx")


def hangul_year(i: int) -> int:
    return 2018 + (i % 9)


def doc_no(i: int, agency: str, doc: str) -> str:
    code = f"{len(agency):02d}{len(doc):02d}"
    return f"{code}-{hangul_year(i)}-{i:06d}"


def money(i: int) -> tuple[int, str]:
    won = 1_000_000 + (i * 137_531) % 98_765_432_000
    if won % 17 == 0:
        raw = f"{won // 1_000_000:,}백만원"
    else:
        raw = f"{won:,}원"
    return won, raw


def iso_date(i: int) -> str:
    y = hangul_year(i)
    m = 1 + (i * 3) % 12
    d = 1 + (i * 7) % 28
    return f"{y:04d}-{m:02d}-{d:02d}"


def korean_date(i: int) -> str:
    y = hangul_year(i)
    m = 1 + (i * 3) % 12
    d = 1 + (i * 7) % 28
    return f"{y}년 {m}월 {d}일"


def article(i: int) -> tuple[int, int]:
    return 1 + (i * 5) % 87, 1 + (i * 2) % 8


def coords_of(i: int) -> dict[str, int]:
    return {
        "section": i % 5,
        "paragraph": (i * 3) % 420,
        "page": (i * 5) % 96,
        "charOffset": (i * 11) % 1800,
        "length": 12 + (i * 13) % 80,
    }


def field_set(keys: list[str]) -> list[str]:
    order = ["cell", "charOffset", "length", "page", "paragraph", "section", "textbox"]
    present = set(keys)
    return [k for k in order if k in present]


def claim_and_quote(i: int) -> tuple[str, str, str, str, str | None]:
    """Return (claim, quote, file, envelope_kind, data_kind)."""
    agency = AGENCIES[i % len(AGENCIES)]
    doc = DOC_TYPES[(i // 3) % len(DOC_TYPES)]
    subj = SUBJECTS[(i * 7) % len(SUBJECTS)]
    obl = OBLIGATIONS[(i * 11) % len(OBLIGATIONS)]
    purpose = PURPOSES[(i * 13) % len(PURPOSES)]
    art, hang = article(i)
    dno = doc_no(i, agency, doc)
    won, raw_amt = money(i)
    date_iso = iso_date(i)
    date_kr = korean_date(i)
    ext = EXT[i % 2]
    file_name = f"{agency}_{doc}_{dno}{ext}"
    court = COURTS[i % len(COURTS)]
    ck = CASE_KINDS[(i * 17) % len(CASE_KINDS)]
    case_no = f"{hangul_year(i)}{ck}{10000 + (i % 90000)}"
    family = i % 12

    if family == 0:
        claim = (
            f"{agency} {doc}({dno}) 제{art}조 제{hang}항은 {subj}을(를) {obl}. "
            f"본 조항은 과업 범위의 일부이며 발주기관의 승낙 없이 제외할 수 없다."
        )
        quote = f"제{art}조 제{hang}항은 {subj}을(를) {obl}"
        return claim, quote, file_name, "search", None
    if family == 1:
        claim = (
            f"{agency} {doc}({dno}) {purpose}는 {raw_amt}을 편성한다. "
            f"정규화된 금액은 {won}원이며 전용·이월은 별도 승인을 요한다."
        )
        quote = raw_amt
        return claim, quote, file_name, "extract-data", "amount"
    if family == 2:
        claim = (
            f"{agency} {doc}({dno})는 {subj} 이행 기한을 {date_kr}({date_iso})로 정한다. "
            f"기한 도과 시 지체상금 조항이 적용된다."
        )
        quote = date_iso
        return claim, quote, file_name, "extract-data", "date"
    if family == 3:
        claim = (
            f"{agency} {doc}({dno}) 제안서평가기준은 {subj} 배점을 "
            f"{60 + (i % 35)}점, 가격평가 배점을 {5 + (i % 20)}점으로 둔다."
        )
        quote = f"{subj} 배점을 {60 + (i % 35)}점"
        return claim, quote, file_name, "search", None
    if family == 4:
        claim = (
            f"{court} {case_no} 판결문은 {subj}에 관하여 원고 청구 중 "
            f"{raw_amt}을 인용하고 나머지 청구를 기각한다."
        )
        quote = f"원고 청구 중 {raw_amt}을 인용"
        return claim, quote, file_name, "search", None
    if family == 5:
        claim = (
            f"{agency} {doc}({dno}) 특수조건은 {subj} 관련 산출물의 지식재산권을 "
            f"발주기관에 귀속시키고 수급인은 이용허락만 받는다고 적는다."
        )
        quote = "산출물의 지식재산권을 발주기관에 귀속"
        return claim, quote, file_name, "search", None
    if family == 6:
        claim = (
            f"{agency} {doc}({dno}) 보안 조항은 {subj} 처리 시 고유식별정보를 "
            f"암호화하고 접속기록을 {1 + (i % 5)}년 보존하도록 한다."
        )
        quote = f"접속기록을 {1 + (i % 5)}년 보존"
        return claim, quote, file_name, "search", None
    if family == 7:
        claim = (
            f"{agency} {doc}({dno}) 계약금액 조정 조항은 {purpose}에 대해 "
            f"물가변동률 {(i % 19) + 1}.{(i % 9)}%를 넘는 경우에만 조정을 허용한다."
        )
        quote = f"물가변동률 {(i % 19) + 1}.{(i % 9)}%"
        return claim, quote, file_name, "search", None
    if family == 8:
        claim = (
            f"{agency} {doc}({dno}) 회의 결과는 {date_kr} {subj} 안건을 의결하고 "
            f"후속 조치는 {obl}."
        )
        quote = f"{subj} 안건을 의결"
        return claim, quote, file_name, "search", None
    if family == 9:
        claim = (
            f"{agency} {doc}({dno}) 민원 회신은 {subj} 신청을 "
            f"{'인용' if i % 2 == 0 else '일부 인용'}하고 처리기한을 {date_iso}로 통지한다."
        )
        quote = f"처리기한을 {date_iso}로 통지"
        return claim, quote, file_name, "extract-data", "date"
    if family == 10:
        claim = (
            f"{agency} {doc}({dno}) 준공검사조서는 {subj} 구간의 기성률을 "
            f"{70 + (i % 30)}%로 인정하고 {purpose} {raw_amt}을 지급 대상으로 한다."
        )
        quote = raw_amt
        return claim, quote, file_name, "extract-data", "amount"
    claim = (
        f"{agency} {doc}({dno}) 붙임 서식은 {subj} 점검표를 두며 "
        f"점검 항목 {3 + (i % 12)}번은 {obl}."
    )
    quote = f"점검 항목 {3 + (i % 12)}번은 {obl}"
    return claim, quote, file_name, "search", None


def make_row(i: int) -> dict:
    claim, quote, file_name, env_kind, data_kind = claim_and_quote(i)
    c = coords_of(i)
    slot = i % 20
    extra_keys: list[str] = ["length"]
    invented: list[str] = []
    fail_kind = None
    verdict = "pass"
    drop: set[str] = set()

    if i % 31 == 0:
        extra_keys.append("cell")
    if i % 37 == 0:
        extra_keys.append("textbox")

    if slot <= 14:
        pass
    elif slot == 15:
        drop.add("page")
        fail_kind = "incomplete_coords"
        verdict = "fail"
    elif slot == 16:
        drop.add("charOffset")
        fail_kind = "incomplete_coords"
        verdict = "fail"
    elif slot == 17:
        drop.add("paragraph")
        fail_kind = "incomplete_coords"
        verdict = "fail"
    elif slot == 18:
        drop.add("section")
        fail_kind = "incomplete_coords"
        verdict = "fail"
    else:
        sub = (i // 20) % 3
        if sub == 0:
            drop.update(REQUIRED)
            drop.update(extra_keys)
            fail_kind = "unbound"
            verdict = "fail"
        elif sub == 1:
            invented = [["pdfPage", "humanPage", "line"][(i // 60) % 3]]
            fail_kind = "invented_key"
            verdict = "fail"
        else:
            claim = ""
            fail_kind = "empty_claim"
            verdict = "fail"

    keys = [k for k in list(REQUIRED) + extra_keys if k not in drop]
    coords_present = all(k in keys for k in REQUIRED)
    row: dict = {
        "rowId": f"CB-{i + 1:06d}",
        "claimText": claim,
        "coordsPresent": coords_present and verdict == "pass" or (
            coords_present and fail_kind in {"invented_key", "empty_claim"}
        ),
        "fieldSet": field_set(keys),
        "envelopeKind": env_kind,
        "verdict": verdict,
        "file": file_name,
        "quote": quote,
    }
    # coordsPresent is purely about the four keys, independent of fail kind.
    row["coordsPresent"] = all(k in keys for k in REQUIRED)
    if fail_kind:
        row["failKind"] = fail_kind
    if data_kind:
        row["dataKind"] = data_kind
    if invented:
        row["inventedKeys"] = invented
    for k in keys:
        if k == "cell":
            row["cell"] = {"row": (i * 2) % 20, "col": i % 8}
        elif k == "textbox":
            row["textbox"] = {"index": i % 6}
        else:
            row[k] = c[k]
    return row


def main() -> None:
    out_dir = Path(__file__).resolve().parents[1] / "fixtures" / "corpus"
    out_dir.mkdir(parents=True, exist_ok=True)
    for old in out_dir.glob("shard_*.ndjson"):
        old.unlink()

    shards = []
    pass_count = 0
    fail_count = 0
    shard_idx = 0
    buf: list[str] = []
    shard_pass = 0
    shard_fail = 0

    def flush() -> None:
        nonlocal shard_idx, buf, shard_pass, shard_fail
        if not buf:
            return
        name = f"shard_{shard_idx:02d}.ndjson"
        (out_dir / name).write_text("\n".join(buf) + "\n", encoding="utf-8", newline="\n")
        shards.append(
            {
                "path": name,
                "count": len(buf),
                "passCount": shard_pass,
                "failCount": shard_fail,
            }
        )
        shard_idx += 1
        buf = []
        shard_pass = 0
        shard_fail = 0

    for i in range(ROWS):
        row = make_row(i)
        if row["verdict"] == "pass":
            pass_count += 1
            shard_pass += 1
        else:
            fail_count += 1
            shard_fail += 1
        buf.append(json.dumps(row, ensure_ascii=False, separators=(",", ":")))
        if len(buf) >= SHARD_SIZE:
            flush()
    flush()

    manifest = {
        "schemaVersion": SCHEMA,
        "generatedBy": "tools/llm_verifier/claim_bind/scripts/gen_claim_corpus.py",
        "axis": "claim-coords",
        "recordCount": ROWS,
        "shardCount": len(shards),
        "passCount": pass_count,
        "failCount": fail_count,
        "uniqueness": "rowId+claimText",
        "requiredFields": list(REQUIRED),
        "shards": shards,
    }
    (out_dir / "manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
        newline="\n",
    )
    print(f"wrote {ROWS} rows in {len(shards)} shards to {out_dir}")
    print(f"pass={pass_count} fail={fail_count}")


if __name__ == "__main__":
    main()
