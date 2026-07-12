"""대표 파라미터로 마크 생성 비교 — 규칙 변형(hang, off 적용 위치) 진단."""
import sys

sys.stdout.reconfigure(encoding="utf-8")
INNER = 24016 - 2 * 141

P21 = "  8. 복합개발사업의 시행으로 용도가 폐지되는 기반시설의 조서ㆍ도면 및 그 기반시설에 대한 둘 이상의 감정평가법인등의 감정평가서와 새로 설치할 기반시설의 조서ㆍ도면 및 그 설치비용 계산서"
M21 = [22, 39, 58, 75, 95, 105]
P27 = "  14. 현물출자시점 등을 포함한 구체적 사업일정(법 제14조제1항제6호에 따른 위탁관리 부동산투자회사가 사업시행자인 경우에 한한다)"
M27 = [24, 43, 61, 75]


def cw(ch, h=1400, d=816, s=700, dot=378, mid=1400, paren=560):
    if ch == " ":
        return s
    if ch.isdigit():
        return d
    if ch == ".":
        return dot
    if ch == "ㆍ":
        return mid
    if ch in "()":
        return paren
    return h


def wrap(text, w_first, w_cont, hang):
    marks, cur, limit, hung, start = [], 0.0, w_first, 0, 0
    for i, ch in enumerate(text):
        w = cw(ch)
        if ch == " " and cur + w > limit and start < i and hung < hang:
            cur += w
            hung += 1
            continue
        if cur + w > limit and start < i:
            marks.append(i)
            start, cur, hung, limit = i, w, 0, w_cont
        else:
            cur += w
    marks.append(len(text))
    return marks


for off_f, off_c, hang, tag in [
    (0, 2440, 1, "first=full cont=-2440 hang1"),
    (0, 2440, 99, "first=full cont=-2440 hang∞"),
    (0, 0, 1, "off없음 hang1"),
    (0, 0, 99, "off없음 hang∞"),
    (2440, 0, 1, "first=-2440 cont=full hang1"),
    (0, 1220, 1, "cont=-1220 hang1"),
]:
    m1 = wrap(P21, INNER - off_f, INNER - off_c, hang)
    m2 = wrap(P27, INNER - off_f, INNER - off_c, hang)
    ok1 = "✓" if m1 == M21 else "✗"
    ok2 = "✓" if m2 == M27 else "✗"
    print(f"[{tag}] p21{ok1} {m1}  p27{ok2} {m2}")
print("오라클     p21", M21, " p27", M27)
