"""The external decision layer (C8 a197/a204, Q207).

Laya is text-only and non-autoregressive: hand it a state and typed questions
and it returns typed answers with calibrated probabilities in one forward pass.
It never generates text, so there is nothing to parse and nothing to hallucinate.
That is why it is the right shape for the questions the pipeline asks — where the
vision validator answers *"does this read as its object"*, this answers *"is this
state admissible"*.

Its own card, though, states the limit that shapes every line here:

    "Base checkpoints are near chance on typed-decisions zero-shot — 0.362 here
     and 0.352 for multilingual, against a 0.318 random and 0.461 majority-class
     baseline."
    "Ships over-confident: Refitting one temperature per (question type, option
     count) moves mean ECE 0.466 -> 0.081 ... Do this on your own data before
     trusting the probabilities."

So a verdict from this file is **not evidence until it is calibrated in the same
run**, against decisions whose answers are records rather than opinions: the four
inputs the geometry gate's own self-test proves it refuses (`_smoke_families.py`,
`gate_self_test`) and the thirty candidates it actually judged
(`assets/icons/review.json`, `checks.geometry.verdict`). A run that does not beat
the majority class there reports `quotable: False`, and nothing downstream may
quote it.

The run also records what the library says about its own confidence: loading this
checkpoint prints a `RuntimeWarning` that some option-count buckets ship
temperatures outside `[0.5, 5]` and that "confidence from the affected buckets"
must be "treated as uncalibrated". That warning is captured rather than allowed
to scroll past, because it is the model's own statement about its numbers.

Usage:

    USE_TF=0 python tools/icons/decide.py            # calibrate, then decide
    USE_TF=0 python tools/icons/decide.py --record   # also write the ledger
"""

import argparse
import json
import os
import sys
import warnings

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
REVIEW = os.path.join(ROOT, "assets", "icons", "review.json")
LEDGER = os.path.join(ROOT, "assets", "icons", "decisions-laya.jsonl")

#: The endpoint's own load-time caution, filled in by `load()` from whatever the
#: library actually prints — read, not assumed.
LOAD_WARNINGS = []


def load(model="convaiinnovations/laya"):
    """Load the checkpoint, keeping what the library says about its own numbers.

    `USE_TF=0` is not decoration: the model card states that transformers probes
    for TensorFlow at import and that when TF is present "its abseil runtime can
    deadlock model construction".
    """
    os.environ.setdefault("USE_TF", "0")
    import laya

    global LOAD_WARNINGS
    with warnings.catch_warnings(record=True) as caught:
        warnings.simplefilter("always")
        agent = laya.load(model)
    LOAD_WARNINGS = [str(w.message) for w in caught]
    return agent


def questions():
    """The decision, in the model's own four primitives rather than a boolean.

    `noul` is a probability, not a flag: the earlier vision validator's failure
    was exactly a boolean form that a small model fills in, so the typed answer
    here is a number that can be wrong in a measurable way.
    """
    return {
        "admissible": {
            "type": "noul",
            "instructions": (
                "This is one candidate icon object in an icon pipeline. Promotion "
                "requires that the object arrives as the number of pieces its "
                "family declares, that it carries no enclosed void smaller than "
                "the declared floor (a drawing sliver), that its declared parts do "
                "not interpenetrate, and that its fill sits inside the family's "
                "envelope for its position. Is this candidate admissible?"
            ),
        },
    }


def state_text(geometry):
    """One candidate's measured state, in as few tokens as the evidence allows.

    The English checkpoint's budget is 512 tokens *total* with 192 reserved for
    option prompts, so the state has to be a handful of short lines rather than
    the whole record. Only the quantities the four judgements read are included.
    """
    pieces = geometry.get("measured_parts")
    declared = geometry.get("declared_parts")
    envelope = geometry.get("fill_envelope") or []
    shares = geometry.get("void_shares") or []
    interference = geometry.get("interference") or []
    return {
        "family": geometry.get("family"),
        "position": f"lambda {geometry.get('lambda')}",
        "declares_pieces": declared,
        "rendered_pieces": pieces,
        "fill": geometry.get("fill"),
        "envelope_here": [round(v, 4) for v in envelope],
        "void_floor": geometry.get("void_floor"),
        "enclosed_void_shares": [round(v, 5) for v in shares],
        "interpenetrating_pairs": len(interference),
    }


def known_cases():
    """Decisions whose answers are on record, so calibration needs no opinion.

    The thirty promote cases are the candidates the gate judged and recorded in
    `review.json`. The four refusals are the inputs `_smoke_families.py`'s
    `gate_self_test` already proves the same gate refuses; their states are
    stated here as the gate saw them, and their labels come from that proof
    rather than from a second opinion about the same numbers.
    """
    with open(REVIEW, encoding="utf-8") as handle:
        review = json.load(handle)

    cases = []
    envelope_by_family = {}
    for icon in review["icons"]:
        for generation in icon["generations"]:
            geometry = generation["checks"]["geometry"]
            envelope_by_family.setdefault(
                (geometry["family"], geometry["lambda"]), geometry["fill_envelope"])
            cases.append({
                "id": f"{icon['id']}.{generation['id']}",
                "label": geometry["verdict"] == "pass",
                "source": "recorded by geometry_gate in review.json",
                "state": state_text(geometry),
            })

    gear_envelope = envelope_by_family.get(("gear", 0.0), [0.4, 0.6])
    refusals = (
        ("undeclared component", {
            "family": "gear", "lambda": 0.0, "declared_parts": 1,
            "measured_parts": 4, "fill": 0.479, "fill_envelope": gear_envelope,
            "void_floor": 0.035, "void_shares": [0.2], "interference": [],
        }),
        ("one-pixel sliver", {
            "family": "gear", "lambda": 0.0, "declared_parts": 1,
            "measured_parts": 1, "fill": 0.479, "fill_envelope": gear_envelope,
            "void_floor": 0.035, "void_shares": [0.0004], "interference": [],
        }),
        ("interpenetration", {
            "family": "gear", "lambda": 0.0, "declared_parts": 1,
            "measured_parts": 1, "fill": 0.479, "fill_envelope": gear_envelope,
            "void_floor": 0.035, "void_shares": [],
            "interference": [("body", "accent", 0.02)],
        }),
        ("fill outside the envelope", {
            "family": "gear", "lambda": 0.0, "declared_parts": 1,
            "measured_parts": 1, "fill": 0.99, "fill_envelope": gear_envelope,
            "void_floor": 0.035, "void_shares": [], "interference": [],
        }),
    )
    for label, geometry in refusals:
        cases.append({
            "id": f"known-refusal:{label}",
            "label": False,
            "source": "_smoke_families.py gate_self_test (the gate refuses this input)",
            "state": state_text(geometry),
        })
    return cases


def calibrate(agent, cases, threshold=0.5):
    """Score the checkpoint against decisions whose answers are already on record.

    Returns accuracy beside the majority-class baseline, the two error rates
    separately — because a gate that refuses everything has perfect sensitivity
    and is useless — and the expected calibration error, which is the quantity
    the model card says must be brought down "on your own data before trusting
    the probabilities".
    """
    import numpy as np

    confidences, correct, rows = [], [], []
    for case in cases:
        result = agent.predict(case["state"], questions())
        probability = float(result["answers"]["admissible"]["noul"])
        called = probability >= threshold
        confidences.append(probability if called else 1.0 - probability)
        correct.append(1.0 if called == case["label"] else 0.0)
        rows.append({
            "id": case["id"],
            "label": case["label"],
            "promote_probability": round(probability, 4),
            "called": called,
            "correct": called == case["label"],
        })

    positives = [c for c in cases if c["label"]]
    negatives = [c for c in cases if not c["label"]]
    caught = sum(1 for r in rows if not r["label"] and not r["called"])
    spared = sum(1 for r in rows if r["label"] and r["called"])
    majority = max(len(positives), len(negatives)) / len(cases)

    try:
        import laya
        ece = float(laya.ece_score(np.array(confidences), np.array(correct), bins=5))
    except Exception as reason:  # noqa: BLE001 - reported, not swallowed
        ece = None
        rows.append({"id": "ece", "unavailable": str(reason)})

    accuracy = sum(1 for r in rows if r.get("correct")) / len(cases)

    # A model can be wrong about which way its own answer points without being
    # unable to separate the classes. Those are different faults with different
    # remedies — phrasing versus fine-tuning — so they are measured separately
    # rather than left for a reader to infer from a list of misses.
    def mean_for(label):
        values = [r["promote_probability"] for r in rows
                  if "promote_probability" in r and r.get("label") is label]
        return round(sum(values) / len(values), 4) if values else None

    inverted = sum(1 for r in rows
                   if "promote_probability" in r
                   and (r["promote_probability"] < threshold) == r["label"])
    return {
        "mean_promote_probability_when_admissible": mean_for(True),
        "mean_promote_probability_when_refused": mean_for(False),
        "accuracy_if_read_inverted": round(inverted / len(cases), 4),
        "cases": len(cases),
        "accuracy": round(accuracy, 4),
        "majority_class": round(majority, 4),
        "beats_majority": accuracy > majority,
        "sensitivity": round(caught / len(negatives), 4) if negatives else None,
        "specificity": round(spared / len(positives), 4) if positives else None,
        "ece_bins5": None if ece is None else round(ece, 4),
        "quotable": accuracy > majority,
        "load_warnings": LOAD_WARNINGS,
        "rows": rows,
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--record", action="store_true",
                        help="append the calibration to decisions-laya.jsonl")
    arguments = parser.parse_args(argv)

    cases = known_cases()
    agent = load()
    report = calibrate(agent, cases)

    print(f"checkpoint: convaiinnovations/laya (english, ModernBERT-large 421M)")
    print(f"budget: 512 tokens total, 192 of them option prompts")
    for note in report["load_warnings"]:
        print(f"the library's own caution: {note}")
    print()
    print(f"decisions with answers on record: {report['cases']}")
    print(f"  accuracy           {report['accuracy']:.4f}")
    print(f"  majority class     {report['majority_class']:.4f}"
          f"   ({'beats' if report['beats_majority'] else 'does NOT beat'} it)")
    print(f"  catches refusals   {report['sensitivity']:.4f}"
          f"   (sensitivity: of the {sum(1 for c in cases if not c['label'])}"
          f" known refusals)")
    print(f"  spares passes      {report['specificity']:.4f}"
          f"   (specificity: of the {sum(1 for c in cases if c['label'])}"
          f" recorded passes)")
    print(f"  ECE (5 bins)       {report['ece_bins5']}")
    print(f"  mean p when admissible {report['mean_promote_probability_when_admissible']}"
          f"   when refused {report['mean_promote_probability_when_refused']}")
    print(f"  accuracy if the answer were read inverted: "
          f"{report['accuracy_if_read_inverted']}")
    print()
    wrong = [r for r in report["rows"] if r.get("correct") is False]
    print(f"wrong on {len(wrong)} of {report['cases']}:")
    for row in wrong[:10]:
        print(f"  {row['id']:24} said promote p={row['promote_probability']:.4f}"
              f"  (truth {'admissible' if row['label'] else 'refuse'})")
    print()
    if report["quotable"]:
        print("VERDICT: quotable — this checkpoint beats the majority class on record.")
    else:
        print("VERDICT: NOT quotable — it does not beat the majority class on "
              "decisions whose answers are on record, so nothing downstream may "
              "quote it. The model card's own remedy is a temperature refit on "
              "your own data, or fine-tuning the typed-decisions checkpoint.")

    if arguments.record:
        with open(LEDGER, "a", encoding="utf-8") as handle:
            handle.write(json.dumps({
                "kind": "laya-calibration",
                "checkpoint": "convaiinnovations/laya",
                "english_context_tokens": 512,
                **{k: v for k, v in report.items() if k != "rows"},
            }, sort_keys=True) + "\n")
        print(f"appended to {os.path.relpath(LEDGER, ROOT)}")

    return 0 if report["quotable"] else 1


if __name__ == "__main__":
    sys.exit(main())
