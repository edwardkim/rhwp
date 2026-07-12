"""규칙 공간 전수 탐색 v2 — 런별 자간(−6%/−3%) 상수 보정 반영."""
import sys
import numpy as np
from scipy.optimize import linprog

sys.stdout.reconfigure(encoding="utf-8")

W = 24016 - 2 * 141
P21 = "  8. 복합개발사업의 시행으로 용도가 폐지되는 기반시설의 조서ㆍ도면 및 그 기반시설에 대한 둘 이상의 감정평가법인등의 감정평가서와 새로 설치할 기반시설의 조서ㆍ도면 및 그 설치비용 계산서"
P27 = "  14. 현물출자시점 등을 포함한 구체적 사업일정(법 제14조제1항제6호에 따른 위탁관리 부동산투자회사가 사업시행자인 경우에 한한다)"
L21 = [20, 39, 58, 75, 93, 105]
L27 = [21, 41, 60, 75]
R21 = [22, 39, 58, 75, 95, 105]
R27 = [24, 43, 61, 75]
SP21 = [(74, 92, -84.0)]                       # 자간 보정(HU/자)
SP27 = [(6, 23, -84.0), (41, 58, -42.0)]

NV = 7  # h s d dot mid paren off
EPS = 1.0


def cvec(t):
    v = np.zeros(NV)
    for ch in t:
        if ch == " ":
            v[1] += 1
        elif ch.isdigit():
            v[2] += 1
        elif ch == ".":
            v[3] += 1
        elif ch == "ㆍ":
            v[4] += 1
        elif ch in "()":
            v[5] += 1
        else:
            v[0] += 1
    return v


def spadj(spans, a, b2):
    tot = 0.0
    for s0, s1, adj in spans:
        tot += adj * max(0, min(b2, s1) - max(a, s0))
    return tot


def trail(t):
    n = 0
    for ch in reversed(t):
        if ch == " ":
            n += 1
        else:
            break
    return n


def lines(text, marks, spans, hang, cf, co, first_kind, use_off, tag):
    A, b, names = [], [], []
    prev = 0
    for li, m in enumerate(marks):
        seg = text[prev:m]
        v = cvec(seg)
        nsp = v[1]
        ntr = min(hang, trail(seg))
        adj = spadj(spans, prev, m)
        offv = np.zeros(NV)
        base = W - adj
        if use_off:
            if li == 0:
                if first_kind == "same":
                    offv[6] = 1.0
                elif first_kind == "jut":
                    offv[6] = 1.0
                    base += 2440
            else:
                offv[6] = 1.0
        a = v.copy()
        a[1] -= ntr + cf * (nsp - ntr)
        a += offv
        A.append(a)
        b.append(base)
        names.append(f"{tag}L{li}fit")
        if li < len(marks) - 1:
            vn = cvec(text[m])
            adj2 = adj + spadj(spans, m, m + 1)
            a2 = v + vn
            a2[1] -= co * nsp
            a2 += offv
            A.append(-a2)
            b.append(-(W - adj2 + (2440 if (use_off and li == 0 and first_kind == "jut") else 0) + EPS))
            names.append(f"{tag}L{li}ovf")
        prev = m
    return A, b, names


def rows(hang):
    A, b, names = [], [], []

    def row(nh, ns, nd, ndot, nmid, ln, tr=0, nm=""):
        v = np.zeros(NV)
        v[0], v[1], v[2], v[3], v[4] = nh, ns, nd, ndot, nmid
        ntr = min(hang, tr)
        if ln == 1:
            a = v.copy()
            a[1] -= ntr
            A.append(a)
            b.append(W)
        else:
            a = -(v.copy())
            a[1] += ntr
            A.append(a)
            b.append(-(W + EPS))
        names.append(nm)

    row(16, 0, 0, 0, 0, 1, 0, "B16")
    row(17, 0, 0, 0, 0, 2, 0, "B17")
    row(0, 0, 32, 0, 0, 1, 0, "H1x32")
    row(3, 24, 0, 0, 0, 1, 0, "H4s24")
    row(3, 28, 0, 0, 0, 2, 0, "H4s28")
    row(16, 0, 0, 2, 0, 1, 0, "H7.2")
    row(16, 0, 0, 4, 0, 2, 0, "H7.4")
    row(8, 0, 0, 0, 8, 1, 0, "H6m8")
    row(8, 0, 0, 0, 9, 2, 0, "H6m9")
    row(16, 1, 0, 0, 0, 1, 1, "Es1")
    row(16, 2, 0, 0, 0, 1, 2, "Es2")
    row(16, 3, 0, 0, 0, 2, 3, "Es3")
    row(16, 0, 1, 0, 0, 1, 0, "H2")
    row(8, 0, 16, 0, 0, 1, 0, "Cx16")
    return A, b, names


feas = []
for hang in (0, 1, 2):
    for fk in ("full", "same", "jut"):
        for cname, cf, co in (("both", 0.25, 0.25), ("ovf", 0.0, 0.25),
                              ("none", 0.0, 0.0)):
            A, b, names = rows(hang)
            for t, mk in ((P21, L21), (P27, L27)):
                a2, b2, n2 = lines(t, mk, [], hang, 0, 0, "full", False, "lad")
                A += a2
                b += b2
                names += n2
            for t, mk, spn in ((P21, R21, SP21), (P27, R27, SP27)):
                a3, b3, n3 = lines(t, mk, spn, hang, cf, co, fk, True, "re")
                A += a3
                b += b3
                names += n3
            An, bn = np.array(A), np.array(b)
            n = len(bn)
            Ae = np.hstack([An, -np.eye(n)])
            c = np.concatenate([np.zeros(NV), np.ones(n)])
            bounds = [(1200, 1700), (300, 1000), (300, 1000), (150, 800),
                      (900, 1700), (300, 1000), (0, 3000)] + [(0, None)] * n
            res = linprog(c, A_ub=Ae, b_ub=bn, bounds=bounds, method="highs")
            viol = res.x[NV:]
            x = res.x[:NV]
            if viol.sum() < 0.5:
                feas.append((hang, fk, cname, tuple(x)))
                print(f"FEASIBLE hang={hang} first={fk} cnd={cname}: "
                      f"h={x[0]:.0f} s={x[1]:.0f} d={x[2]:.0f} dot={x[3]:.0f} "
                      f"mid={x[4]:.0f} paren={x[5]:.0f} off={x[6]:.0f}")
            elif viol.sum() < 400:
                bad = [(names[i], round(viol[i])) for i in range(n) if viol[i] > 0.5]
                print(f"near hang={hang} first={fk} cnd={cname} 위반합={viol.sum():.0f} {bad[:4]}")
print("총 해:", len(feas))
