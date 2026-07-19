---
kind: reference
status: active
canonical: mydocs/tech/hancom_model/README.md
last_verified: 2026-07-19
---

# Object/control placement semantics

This note is the fail-closed semantic model for object/control placement in the
HNC canonical model. It models only relations established within the model's
admitted judgments. Native names remain opaque, and a relation is not promoted
to ownership merely because it is pointer-valued or occurs near an owner
transition.

Related models within this directory:

- [Unified layout semantics](unified_layout_semantics.md)
- [Endpoint / line closure semantics](endpoint_line_closure_semantics.md)
- [Flow / pagination semantics](flow_pagination_semantics.md)
- [Model index](README.md)

## Provenance notation

Every judgment carries one or more tags:

- `[D:S]`: dynamically established under model stage `S` with a causal join;
- `[S:S]`: statically established under reviewed stage `S` structure;
- `[O:S]`: observed by `S`, but not semantically promoted;
- `[N:S]`: negative result or explicit non-typing rule from `S`;
- `[U]`: unknown and retained as a family parameter.

`S0 -> S1 -> S0r` denotes immutable baseline, one declared perturbation, and a
fresh byte-identical baseline reopen. A claim marked recurrent additionally
requires two fresh process-start repetitions.

## Domains

Let the following sets be disjoint:

```text
Pos             source-unit positions
State           source addressing states
Source          = Pos x State x ParagraphId
Span            = { [i,j) | i,j in Pos and i <= j }
ControlCode     native control codes
RuntimeObject   process-local concrete objects
RttiId          normalized concrete type identity plus base hierarchy
Call            per-thread call serials
TargetRva       module-relative native targets
Advance         signed native outward results
Class8          = {0,1,2,3,4,5,6,7}
Extent          numeric placement extents
Endpoint        = Pos x State
FrameKey        pointer-free normalized frame identities
PageKey         pointer-free normalized committed page identities
OwnerKey        pointer-free normalized destination-owner identities
AnchorKey       opaque anchor relation/identity keys
Relation30      opaque, non-null object secondary-relation values
```

The principal typed values are:

```text
SourceControl = (source : Source, code : ControlCode)
ObjectId      = (rtti : RttiId, dispatchIdentity, hierarchyIdentity)
DispatchJoin  = (source, controlCall : Call, resolvedCall : Call, object)
PlacementObs  = (placementCall : Call, class : Class8,
                 primaryExtent : Extent, secondaryExtent : Extent,
                 result : Extent)
OwnerCommit   = (flowSerial, candidateFrame : FrameKey,
                 rejectedFrames : list FrameKey,
                 destinationOwner : OwnerKey, committedPage : PageKey)
```

`RuntimeObject`, raw frame pointers, and raw owners are run-local witnesses and
must not appear in a cross-run semantic tuple. `[D:OP00,OP01,OP02,OP03a,XL02]`

## Partial native relations

The filtered resolver and the two virtual relations are partial functions:

```text
resolve    : SourceControl x Call ⇀ RuntimeObject option

outward    : RuntimeObject x Call ⇀ TargetRva x Advance

inward     : RuntimeObject x Call x Class8 ⇀
             (primaryExtent : Extent, secondaryExtent : Extent,
              result : Extent)
```

Here `outward` is the outward-dispatch relation carried by the numeric
resolver's placement provider, and `inward` is the interior extent-selection
relation reached only along the shared placement path.

`outward(o,c) = (t,r)` is admitted only when the resolver event is joined to
the outward call by parent/child call serial, the same source position/state,
and membership of `o` in that call's resolved-object set. `[D:OP00,OP01,OP02]`

The mere presence of a same-slot interior extent dispatch entry does **not**
establish `inward`. The relation is defined only when the shared caller actually
invokes it, the placement event is nested in the joined shared-extent frame, and
the runtime object matches the expected interior object register. `[N:OP01]`

## Dispatch sum type

The five-way native dispatch result is:

```text
Dispatch =
    Mask(code)
  | Null(code, outwardResults)
  | Zero(object : ObjectId, target : TargetRva)
  | Specialized(object : ObjectId, target : TargetRva,
                outwardResults : set Advance)
  | Shared(object : ObjectId, target = sharedTarget,
           relation30 : Relation30,
           outwardResults : set Advance)
```

where `sharedTarget` is the model's single shared placement-provider target,
with these inference rules:

```text
maskAdmitted(code) = false    no control_advance    no resolved_control
----------------------------------------------------------------------- MASK
                    dispatch(code) = Mask(code)

maskAdmitted(code) = true    control_advance(c)    resolve(code,c) = None
------------------------------------------------------------------------ NULL
                dispatch(code) = Null(code, results(c))

resolve(code,c) = Some(o)    outward(o,c) = (t,0)    t != sharedTarget
----------------------------------------------------------------------- ZERO
                    dispatch(code) = Zero(id(o),t)

resolve(code,c) = Some(o)    outward(o,c) = (t,r)    r != 0
 t != sharedTarget
---------------------------------------------------------------- SPECIALIZED
          dispatch(code) = Specialized(id(o),t,results(c))

resolve(code,c) = Some(o)    outward(o,c) = (sharedTarget,r)
enteredSharedExtent(o,c,e)    relation30(o,e) != null
--------------------------------------------------------------------- SHARED
 dispatch(code) = Shared(id(o),sharedTarget,relation30(o,e),results(c))
```

The constructors are exclusive. Mixed null/non-null resolution, unstable
RTTI/hierarchy/outward-target identity, a shared target without a nested extent,
or a specialized constructor targeting `sharedTarget` is rejection. `[D:OP01]`

OP-02 and OP-03a use the following *partial* projection:

```text
outer3(Mask(_))          = BYPASS
outer3(Specialized(...)) = SPECIALIZED
outer3(Shared(...))      = SHARED
outer3(Null(...))        undefined
outer3(Zero(...))        undefined
```

Thus `BYPASS` means absence of outward and resolution events; it is not a
synonym for a null object or a zero outward result. `[D:OP03a]`

## Shared-path numeric placement

For a joined shared placement frame only, define:

```text
selectExtent : Class8 x Extent x Extent -> Extent
selectExtent(k,primary,secondary) = if k = 0 then primary else secondary
```

The accepted equation is:

```text
dispatch(code) = Shared(...)    inward(o,p,k) = (primary,secondary,r)
placementParent(p) = sharedExtentCall(e)    objectMatchesInterior(o,p)
------------------------------------------------------------------------
                         r = selectExtent(k,primary,secondary)
```

`k` is a numeric class only. No meaning such as horizontal, vertical, width,
height, inline, floating, owner, or anchor is assigned to any numeric value.
The selector equation is undefined for `Mask`, `Null`, `Zero`, and
`Specialized`. `[D:OP00,OP01,OP02,OP03a,XL02] [N:OP02]`

## TAC and non-TAC

Let `tac : SourceControl -> Bool` be the serialized/native treat-as-character
property. TAC is an independent fixture coordinate, not a dispatch constructor:

```text
Variant = OwnerKind x WritingMode x tac
OwnerKind  = {body, table, text_box}
WritingMode = {horizontal, vertical_laid, vertical_upright}
```

OP-02 proves coverage of all `3 x 3 x 2 = 18` variants while holding the
declared variant identity through every event. It does not prove either
`tac=true <=> Shared` or `tac=false <=> Specialized`. `[D:OP02]`

XL-02 therefore treats TAC/non-TAC as paired controls:

- `xl02a_tac`: boundary placement for TAC objects;
- `xl02b_float`: floating wrap/restriction/boundary controls, with paired TAC
  fixture identity;
- `xl02c_nested`: TAC/non-TAC outer containers crossed with inner content and
  boundary/edit coordinates.

Any claimed causal effect of TAC requires a paired fixture in which only the
declared TAC coordinate changes. `[D:XL02]`

## Verdict lattice

For each dimension `d` in `{sourceSpan, dispatch, endpointEffect}`, use:

```text
              ProvedBoth
              /        \
   ProvedStatic        ProvedDynamic
              \        /
                Observed
                   |
                Unknown
```

Formally, `Unknown <= Observed <= ProvedStatic <= ProvedBoth` and
`Unknown <= Observed <= ProvedDynamic <= ProvedBoth`; `ProvedStatic` and
`ProvedDynamic` are incomparable. A family verdict is the product lattice:

```text
FamilyVerdict = V_sourceSpan x V_dispatch x V_endpointEffect
```

A family is promotable only if every coordinate is at least one of
`ProvedStatic` or `ProvedDynamic`. Dispatch proof alone never implies a source
span or an endpoint effect. `[D:OP00,OP01]`

The endpoint-effect coordinate requires a pointer-free committed descriptor or
page tuple joined to the same source identity. Visual movement, fixture width,
RTTI, proximity, or a placement return is not endpoint proof.

## Ownership and anchor separation

Ownership is introduced only by a committed flow/page chain:

```text
source(s) = join(j) = commit(q) by
  (threadId, sourcePosition, addressState, paragraphIdentity)

flowSerial(j) is joined to a native flow-owner event
destinationOwner(j) = destinationOwner(q)
committedPage(j) = committedPage(q)
pointerFreeNormalized(q)
-------------------------------------------------------------------- OWNER
             owns(destinationOwner(q), committedPage(q), s)
```

The following are explicit non-rules:

```text
relation30(o) != null        -/-> owns(relation30(o), ...)
anchorRelation(a,s)          -/-> owns(a,s)
samePointer(x,y)             -/-> owns(x,y)
nearInTime(x,y)              -/-> owns(x,y)
```

The secondary relation is only an opaque relation required to enter the observed
shared path. OP-03a requires `requiredSecondaryRelationTypedAsOwner = false`.
`AnchorKey` remains separate from `OwnerKey`; OP-03a requires
`anchorSemanticOwnerTypeAssigned = false`, and XL-02 must join any anchor
relation onward to an independently evidenced flow-owner/page key.
`[N:OP03a] [D:XL02]`

## Proved lemmas

1. **Dispatch exclusivity.** Under stable source/call/object identity, exactly
   one `Dispatch` constructor is admissible. `[D:OP01]`
2. **Shared selector.** A joined shared interior return equals the primary
   extent iff class is zero and equals the secondary extent otherwise. No
   semantic class names follow.
   `[D:OP00,OP01,OP02,OP03a,XL02]`
3. **Causal outward join.** Resolver proximity is insufficient; source,
   call-serial parentage, resolved-object membership, and actual outward target
   are all necessary. `[D:OP00,OP01,OP02,OP03a]`
4. **Restoration.** For accepted OP-02, OP-03a, and XL-02 variants,
   `normalize(S0) = normalize(S0r)` and `normalize(S0) != normalize(S1)`, with
   fresh-run recurrence. `[D:OP02,OP03a,XL02]`
5. **Endpoint independence.** Equal dispatch observations do not establish
   equal endpoint effects; the committed pointer-free endpoint tuple is a
   separate verdict coordinate. `[D:OP00,OP01,OP02]`
6. **Owner non-aliasing.** Neither `Relation30` nor `AnchorKey` inhabits
   `OwnerKey`; ownership requires the committed flow/page rule above.
   `[N:OP03a] [D:XL02]`
7. **Acquisition is not semantics.** OP-03b qualification yields only a stable
   candidate family/fixture tuple. It proves no source span, dispatch, placement,
   endpoint, anchor, or owner judgment. `[N:OP03b]`

## Unknown family parameters

For every runtime family `F`, retain:

```text
theta_F = (
  admittedMaskCodes,
  resolverDomain,
  concreteRttiHierarchy,
  outwardTarget,
  specializedResultMeaning,
  tacSensitivity,
  observedClassSubset : set Class8,
  numericClassMeanings,
  primaryExtentMeaning,
  secondaryExtentMeaning,
  relation30Meaning,
  endpointTransform,
  ownerTransferRule
)
```

Only observed values may be instantiated. In particular,
`numericClassMeanings`, both extent meanings, `relation30Meaning`, and any owner
transfer rule remain `[U]` until separately discriminated.

## Structural placement model appendix

The facts in this appendix describe the model's static placement structure —
the type hierarchy, provider mapping, and interior selectors — independent of
any single execution. A provider mapping is a structural implementation
mapping, not proof that a native fixture resolves that class or invokes the
slot. `[S:STATIC-PLACEMENT]`

Type identity in the model follows an MSVC-style RTTI structure: a type
descriptor links to a complete-object locator, which links to a class-hierarchy
descriptor, which links to the primary-dispatch identity. The model tracks these
object families by normalized type identity and base hierarchy; the concrete
per-family identity and base-descriptor count are family parameters.

The placement-relevant object families are:

| Family | Base hierarchy | Placement role |
|---|---|---|
| `EquationEditObject` | `ShapeObject -> TextCtrl -> Ctrl` plus its interface | shared placement provider |
| `GenericFormObject` | `ShapeObject -> TextCtrl -> Ctrl` plus its interface | shared placement provider |
| `FormObject` | `Ctrl` plus its interfaces and command-target base | non-shared specialized provider |
| `HyperlinkField` | `FieldCtrl -> TextCtrl -> Ctrl` plus its field interface | non-shared provider whose body returns zero |
| `DutmalControl` | `TextCtrl -> Ctrl` | non-shared specialized provider |
| `UnknownObject` | `DrawingObject -> ShapeComponent -> Ctrl` | static candidate mapping only; no genuine runtime fixture |
| `UnknownField` | `FieldCtrl -> TextCtrl -> Ctrl` plus its field interface | separate field family; not `UnknownObject` evidence |

### Family-to-provider (outward) mappings

Each object family maps to a placement provider reached through the outward
relation. The mappings are:

| Concrete family | Outward provider role | Static consequence |
|---|---|---|
| `EquationEditObject` | shared placement provider (`sharedTarget`) | shared implementation target |
| `GenericFormObject` | shared placement provider (`sharedTarget`) | shared implementation target |
| `FormObject` | non-shared specialized provider | non-shared specialized target; result remains input-dependent |
| `HyperlinkField` | field provider whose body returns zero | non-shared target whose body returns zero |
| `DutmalControl` | non-shared specialized provider | non-shared specialized target |
| `UnknownObject` | candidate provider (no genuine fixture) | static candidate mapping only; no genuine runtime fixture |
| `UnknownField` | field provider whose body returns zero | separate field family; not `UnknownObject` evidence |

The zero-returning field provider body returns `0` on every local exit.
Therefore, if a future joined native resolver/outward call proves either field
class at this slot, the five-way dynamic constructor is `Zero`, not
`Specialized`; the static provider mapping alone does not emit either dynamic
verdict. `[S:STATIC-PLACEMENT]`

The shared placement provider has the following exact return selector. Let `q`
denote the opaque value derived from argument 4 (directly as `arg4.derivedField` when
argument 4 is non-null, otherwise by the recovered helper chain),
`k = numericClass(secondaryRelation(this), q)` where `numericClass` is the
numeric-class resolver, and `p = call [interiorExtentSlot(this)](this,0)` where
`interiorExtentSlot` is the structural interior-extent slot. Then, writing
`primary` and `secondary` for the primary and secondary extent fields of `p`:

```text
(u8(this.flags4c) & 1) = 0 or secondaryRelation(this) = 0  =>  sharedResult = 0
otherwise k != 0                                           =>  sharedResult = secondary
otherwise                                                  =>  sharedResult = primary
```

The argument-2 reference cleanup after selection does not change the return
value. This shows that the shared outward provider actually calls the structural
interior-extent slot for EqEdit and GenFormObject; it does not assign a semantic
name to either selected extent field.

The `FormObject` specialized provider copies a fixed-size record from
argument 1, reads `s = arg1.leadingField`, an internal field of `this`, and `that record.internalField`,
calls an internal record-validation routine with the copied record and literal
mode arguments, and, when that result is not `-1`, calls an internal record-index
routine `(s, index, &record)`. It then derives two record differences and returns
the indirect result through an internal object reference, at a fixed indirect
dispatch slot:

```text
form_specialized(this,a) = call(obj, indirectSlot,
                                transform(copy_record(a), a.derivedField))
```

where `transform` and the indirect callee semantics remain `[U]`.

The `DutmalControl` specialized provider derives the same opaque `q` shape,
calls `k = numericClass(secondaryRelation(this), q)`, computes the UTF-16 length
of an internal string field, prepares a reference-counted temporary from
arguments 1 and 2, and calls an internal processing routine with `k`, that
length, argument 5, an internal field pointer, literal `1`, and the temporary
pair. Its exact numeric return is that routine's return unchanged; cleanup does
not alter the returned value. This is specialized class-dependent processing,
not a proved extent, ownership, or placement result.

The `UnknownObject` candidate provider is also closed statically. Let `P` mean
that arguments 1 and 2 are non-null and both style-object creations return
non-null. On the local return graph:

```text
not P                                      =>  0 (after the exception helper returns)
P and *(this.textFlag) = 0                 =>  1 (after text-object create/attach)
P and *(this.textFlag) != 0 and
    call convert(arg1,arg2,0) = 0          =>  0
otherwise                                  =>  1
```

The intervening body exports drawing style, fill, shadow, padding/margin,
wrapping, and text alignment state. Its result is therefore a Boolean
conversion-success status, not an extent. Because this class still lacks a
genuine fixture and joined outward invocation, it remains a static candidate
and emits no dynamic constructor.

### Structural interior-extent (`inward`) mappings

Only families whose primary dispatch extends through the interior-extent slot are
listed. These are structural slot mappings. They establish `inward` only if the
actual shared caller invokes the slot with the required nesting/object/call
evidence from the main model.

| Family | Proved partial equation | Runtime status |
|---|---|---|
| `EquationEditObject` | with `p=*(this.tableRef)`: `arg=0 or p=0 => this.base`; otherwise `*(*(p+4)+u16(p.stride)*(arg-1))` | invoked by shared caller; runtime nesting still required |
| `GenericFormObject` | `interiorSlot(this,arg) = this.base` | invoked by shared caller; runtime nesting still required |
| `FormObject` | indirect transformed call through an internal object reference, at a fixed indirect slot | non-shared outer target; do not type as placement |
| `UnknownObject` | returns `(*(this.textFlag) != 0)` | candidate only; no genuine fixture/caller proof |

The `EquationEditObject` interior-extent equation is a table lookup: given a nonzero
selector `arg`, the return is `*(base + stride*(arg-1))`, where `base` and
`stride` are read from the family's interior table structure. This table-lookup
relation is kept as a model relation only; no semantic meaning is assigned to
the indexed entries.

The primary dispatchs for `HyperlinkField`, `DutmalControl`, and
`UnknownField` end before the interior-extent slot; no such slot is claimed
for them.

### Numeric class resolver

The numeric-class resolver takes an entry object `S` and an
argument `A`, and produces a value in `Class8`. Let `W = the control-word field of A` be the
control-word field of `A`, and let `H(S)` denote the resolver's helper-derived
object for `S`.

The first partial branch draws the result directly from `S`'s own dispatch:

```text
v_probe = call [probeSlot(S)](S)
v_probe != 0 and call [resultSlot(S)](S) = R != 0
-------------------------------------------------
numericClass(S,A) = R
```

`R` is returned directly and is not locally constrained to `Class8`. Thus a
requested value in `{1,3,4,5,6,7}` may come directly from the result slot, but
the static structure does not identify which concrete dispatch supplies it.

When that path does not return nonzero, the result is the bit-extraction of the
control word:

```text
C(W) = (W >> 1) & 7
```

It requires `A != null` and `(the control-word low byte of A & 0x0e) != 0`, plus either:

```text
I.  (call [flagSlot(S)](S) & 1) != 0

II. the same bit is zero, H(S) != null, and Q is accepted, where
    E = H(S), V = *E,
    Q = u16(call V.controlWordSlot(E))             if V.dispatchSlot = the known dispatch entry
      = u16(call V.dispatchSlot(E))            otherwise

    accepted(Q,A) =
         Q = 17
      or Q = 16 and (the control-word low byte of A & 0x10) != 0
      or Q = 2 and call [tagSlot(H(S))](H(S)) = 0x73656364
```

All other local paths return the current zero accumulator. On an admitted
bit-extraction path, the requested numeric values correspond exactly to:

| Numeric result | Required `W & 0x000e` |
|---:|---:|
| `1` | `0x0002` |
| `3` | `0x0006` |
| `4` | `0x0008` |
| `5` | `0x000a` |
| `6` | `0x000c` |
| `7` | `0x000e` |

For the `Q=16` branch, `W&0x0010` is additionally required. Bits outside the
tested masks are locally irrelevant. There are many direct call sites to the
numeric-class resolver, but no call-site family or semantic class name is
assigned here.

The meanings of `S`, `A`, `W`, `Q`, the magic values, the direct result `R`, and
every numeric result remain `[U]`. This appendix proves code relations, not
runtime family resolution, placement entry, source span, endpoint effect, or
ownership.

## Stage proof obligations

| Stage | May establish | Must not claim | Remaining obligation |
|---|---|---|---|
| OP-00 | known-positive shared calibration; joined outward; shared selector equation; three verdict coordinates | general family semantics | retain calibration-only scope and pointer-free endpoint recurrence |
| OP-01 | five-way dispatch for a real novel family; shared interior only after shared discovery | placement from same-slot interior extent; promotion from dispatch alone | close source span and endpoint effect independently for each family |
| OP-02 | 18-case owner/mode/TAC matrix; numeric class subsets; shared-only selector | semantic class names; TAC/dispatch equivalence | discriminate class behavior with one-variable paired fixtures |
| OP-03a | source -> object -> outward -> optional shared placement -> committed owner/page chain | secondary relation or anchor as owner | preserve separate relation/anchor/owner types and exact causal joins |
| OP-03b | acquisition-qualified unknown family candidates | any runtime semantics | obtain authorized native fixtures, then restart at OP-01 discovery |
| XL-01 | carry/refinalize/replay endpoint classification | object placement or ownership from endpoint alone | maintain exact endpoint/state commit and unequal-width discriminator |
| XL-02 | TAC/floating/nested flow-page integration and committed ownership | anchor identity as ownership; unjoined placement | complete declared matrices, paired deltas, owner stacks, restoration |
| XL-03 | endpoint crossing versus reservation/owner-geometry effects | anchor pointer as owner | keep endpoint/body maps and selected page controls independently joined |
| XL-04 | line-local adjusted advances and committed endpoint closure | stored endpoint partitions or object-owner semantics | retain pointer-free arrays, width counterexamples, and recurrence |

No later stage may weaken an earlier proof obligation. An undefined partial
function, an unjoined event, a pointer-bearing normalized tuple, an unknown
dispatch projection, or a missing provenance tag yields rejection rather than
semantic completion.
