"""Put the reviewed set to a model outside the pipeline (C8 a171, a197).

Every check in `generate.py` is arithmetic: fill, gloss, parts, voids, containment, and
the geometry gate's four refusals. Not one of them can say whether an object *reads* as
the thing it is meant to be, whether it asserts a reading its own declaration forbids, or
whether five samples on a ladder look like **one object at five points** rather than five
different icons. That last question is the claim a180 and a194 make and nothing in this
repository tests it.

**What this tool learned the hard way, kept here rather than in a commit message.** The
first version asked a model four boolean questions per icon. It answered `reads_as_object:
true` for all six families with an empty `notes` field, and then answered `true` for pure
noise with the real declarations attached — while its own blind naming pass, asked the same
question in words, said *"unrecognisable: dots"* five times. A yes/no about a declared
object is a form the model fills in. Three things follow, and they shape everything below:

* **The question is a forced choice among the declared objects**, which the pipeline scores
  itself. A model that cannot refuse is caught by the control, and a model that guesses is
  caught by the options rotating per icon.
* **Nothing is quoted until the instrument is calibrated on the same run**: the corpus
  marks each icon was built against are put through the identical question, and their score
  is the **ceiling this run's verdicts are read against** — not a threshold invented here.
  The noise control must be refused, and a validator that calls noise an object is reported
  as uncalibrated rather than believed.
* **The matte is declared.** Transparency composites to black for the first model tried, so
  the corpus's black-on-transparent marks were invisible to it ("black screen", six times)
  and so were dark renders. Both sides are judged flattened onto the same declared matte.

It still fails closed and still promotes nothing: no endpoint or no model means no verdicts,
a missing validator is reported as missing rather than replaced by the local checks, and a
promotion stays a person's decision in the picker.

Configuration is environment-only:

    VALIDATOR_BASE_URL   default http://127.0.0.1:11434/v1 (a local endpoint)
    VALIDATOR_MODEL      required; nothing is sent without one
    VALIDATOR_API_KEY    optional, for a hosted endpoint
    VALIDATOR_CONTEXT    the served context in tokens, when the endpoint will not say

    python tools/icons/validate.py --reachable    # is there an endpoint at all?
    python tools/icons/validate.py --ladder       # every packet shape and its size
    python tools/icons/validate.py --dry-run      # build the packets, send nothing
    python tools/icons/validate.py --calibrate    # reference + noise only, no verdicts
    python tools/icons/validate.py                # calibrate, then read the reviewed set
    python tools/icons/validate.py --questions    # the boolean form, kept for a stronger
                                                  # model: it is not the default because it
                                                  # passed noise
"""

import argparse
import base64
import datetime
import hashlib
import io
import json
import os
import re
import sys
import urllib.error
import urllib.request

from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.abspath(os.path.join(HERE, "..", "..", "assets", "icons"))
MANIFEST = os.path.join(OUT, "manifest.json")
LEDGER = os.path.join(OUT, "validation.jsonl")

DEFAULT_BASE_URL = "http://127.0.0.1:11434/v1"

#: What the model composites transparency onto. `white` is the default because it is the
#: one the corpus reference marks are legible on: they are black on transparency, and on
#: transparency the first model tried reported "black screen" for every one of them.
MATTES = {"white": (255, 255, 255, 255), "black": (0, 0, 0, 255),
          "grey": (128, 128, 128, 255), "none": None}
DEFAULT_MATTE = "white"

#: The letters a forced choice is answered with, and the option rotation. The correct
#: answer's letter moves per icon, so a positional preference shows up as errors instead of
#: being argued away.
LETTERS = "ABCDEF"

#: The reviewed sizes as the record names them, and the size each one holds. The record
#: keys its files by *role* (`sharp_png`, `context_png`), which is a reading of what the
#: size is for rather than how big it is — so the mapping is stated here, once, instead of
#: being guessed from a filename.
REVIEWED_SIZES = {"sharp_png": 96, "recognition_png": 32, "context_png": 24}

#: The sizes a packet carries. The decision size is what the corpus anchors were measured
#: at; the smallest exists because "does it still read small" is one of the questions and no
#: number in this repository answers it. The middle size is left out on purpose.
PACKET_SIZES = (96, 24)

#: The contact sheet's layout, used only where a *sequence* is the subject. It must never
#: carry the reading question: measured, the same model scored the strip `3/6` and answered
#: "a ribbon of carriageway" for the bin, the ticket **and the noise** — five copies of one
#: object in a row are parallel bands, so the strip was being judged on its own layout.
SHEET_ROWS = (96, 24)
SHEET_GUTTER = 6

#: The questions the boolean form asks. Kept because a stronger model may answer them well,
#: and because they are the only way to ask about a *forbidden* reading, which a forced
#: choice cannot express. Not the default: this form passed noise.
QUESTIONS = (
    {"key": "reads_as_object",
     "ask": ("Does each render read as the object its family names? Answer yes or no, "
             "and say what it looks like if not.")},
    {"key": "asserts_forbidden",
     "ask": ("Does any render assert one of the icon's forbidden readings? A forbidden "
             "reading is a *meaning*, not a colour: state which one and where it is.")},
    {"key": "ladder_is_one_object",
     "ask": ("The five renders of one icon are declared to be ONE object at five points "
             "on a style ladder, not five different icons. Does the sequence read that "
             "way, and where does it break?")},
    {"key": "legible_at_smallest",
     "ask": ("At the smallest reviewed size, does the object still read? Name the "
             "smallest size at which it stops.")},
)

ANSWER_SHAPE = (
    '{"per_icon": {"<icon id>": {"reads_as_object": true, "asserts_forbidden": [], '
    '"ladder_is_one_object": true, "legible_at_smallest": true, '
    '"smallest_legible_px": 24, "confidence": 0.0, "notes": ""}}, '
    '"disagreements": [""], "overall": ""}'
)

#: What a forced-choice answer looks like. `name_it` is the evidence rather than decoration:
#: it is the model's own words for the shape, written before any noun was offered to it, and
#: it is what makes a verdict checkable by a reader.
READ_SHAPE = (
    '{"choice": "<A|B|C|D|E|F|NONE>", "depicts": true, '
    '"name_it": "<what you see, <=10 words, describing the shape and its features>", '
    '"supporting_words": "<which of those words made you choose it>"}'
)

JUSTIFY_NOTE = (
    "You already looked at exactly these renders in an earlier request, before you were "
    "told what they were meant to be, and named them as: {names}\n"
    "Answer from your own naming. Every `true` must be supported by it: if "
    "`reads_as_object` is true, the name you gave must contain the object's own noun. Put "
    "the supporting words in `notes` — a bare `true` with no `notes` is not accepted."
)

NAMING_QUESTIONS = (
    "Name each reviewed sample as if you had not been told what it is: <=8 words per "
    "sample, naming its shape and its most distinctive feature (a hole, a rim, notches, "
    "lane markings, a handle). Do not guess a category; describe what is drawn. If a "
    "sample is not recognisable as anything, say \"unrecognisable: <what you see>\".",
    "Also say what the transparent background rendered as in your view (black, white, a "
    "chequerboard), because that affects how the object's colour reads.",
)
NAMING_SHAPE = ('{"names": ["<leftmost sample, <=8 words>", "...", "...", "...", '
                '"<rightmost sample>"], "background": "<black | white | other>"}')


def base_url() -> str:
    return os.environ.get("VALIDATOR_BASE_URL", DEFAULT_BASE_URL).rstrip("/")


def model_name() -> str:
    return os.environ.get("VALIDATOR_MODEL", "").strip()


def api_key() -> str:
    return os.environ.get("VALIDATOR_API_KEY", "").strip()


def declared_context():
    """`VALIDATOR_CONTEXT` as a number, or None. For endpoints that will not say."""
    raw = os.environ.get("VALIDATOR_CONTEXT", "").strip()
    return int(raw) if raw.isdigit() else None


def read_json(path):
    with open(path, "r", encoding="utf-8") as handle:
        return json.load(handle)


def digest(payload) -> str:
    """A stable digest of a packet, so a verdict names what it answered."""
    blob = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(blob).hexdigest()[:16]


def now() -> str:
    return datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def file_map(review: dict) -> dict:
    """`(icon id, generation id) -> {px: path}`, out of the picker's own record."""
    mapping = {}
    for icon in review.get("icons", []):
        for generation in icon.get("generations", []):
            files = generation.get("files") or {}
            mapping[(icon["id"], generation.get("id"))] = {
                REVIEWED_SIZES[role]: path
                for role, path in files.items() if role in REVIEWED_SIZES
            }
    return mapping


def evidence(manifest, files_by_candidate=None, lean=False) -> list:
    """The packet's text: what each icon claims and what the pipeline measured.

    Read back out of the manifest rather than recomputed, because a validator that judged
    numbers this program recalculated could disagree with the pipeline and be right about
    the disagreement while being wrong about the artefact.
    """
    icons = []
    for icon in manifest["icons"]:
        candidates = []
        for generation in icon.get("generations", []):
            silhouette = generation.get("silhouette") or {}
            gate = generation.get("geometry_gate") or {}
            sizes = generation.get("sizes") or []
            decision = next((item for item in sizes if item.get("px") == 96),
                            sizes[-1] if sizes else {})
            measurements = decision.get("measurements") or {}
            candidate = {
                "id": generation["generation"],
                "lambda": generation.get("lambda"),
                "measured": {key: measurements.get(key) for key in
                             ("coverage", "fill", "gloss", "p75_luminance",
                              "clip_fraction")},
                "parts": {"declared": gate.get("declared_parts"),
                          "measured": gate.get("measured_parts")},
                "voids": gate.get("void_shares"),
                "containment_of_declared_mark": silhouette.get("containment"),
                "gate_verdict": gate.get("verdict"),
                "gate_notes": gate.get("notes") or None,
                "files": (files_by_candidate or {}).get(
                    (icon["id"], generation["generation"]), {}),
            }
            if not lean:
                candidate.update({"label": generation.get("generation_label"),
                                  "why": generation.get("generation_why"),
                                  "frame_fit": generation.get("frame_fit"),
                                  "surface": generation.get("surface")})
            candidates.append(candidate)
        icons.append({
            "id": icon["id"], "meaning": icon.get("meaning"),
            "locates": icon.get("locates"), "identity": icon.get("identity"),
            "forbidden_readings": icon.get("forbidden_readings"),
            "declared_mark": (icon.get("silhouette") or {}).get("glyph"),
            "candidates": candidates,
        })
    return icons


def render_paths(icon, sizes=PACKET_SIZES) -> dict:
    """`{px: [(candidate id, lambda, path), ...]}` in λ order — the reviewed renders."""
    collected = {size: [] for size in sizes}
    for generation in icon["candidates"]:
        for size, path in (generation.get("files") or {}).items():
            if size in collected:
                full = path if os.path.isabs(path) else os.path.join(OUT, path)
                collected[size].append((generation["id"], generation.get("lambda"), full))
    for size in collected:
        collected[size].sort(key=lambda item: (item[1] is None, item[1]))
    return collected


def sheet_data_url(sheet: "Image.Image") -> str:
    buffer = io.BytesIO()
    sheet.save(buffer, format="PNG")
    return "data:image/png;base64," + base64.b64encode(buffer.getvalue()).decode("ascii")


def flatten(url: str, matte: str) -> str:
    """The same image composited over a declared matte — no pixel of it resampled.

    Not cosmetic: on transparency this model reported a black background, which makes a
    black mark invisible (the corpus's own marks came back "black screen", six for six).
    Both sides of a comparison are flattened onto the same matte so the comparison is about
    the drawing.
    """
    if MATTES.get(matte) is None:
        return url
    with Image.open(io.BytesIO(base64.b64decode(url.split(",", 1)[1]))) as handle:
        image = handle.convert("RGBA")
    back = Image.new("RGBA", image.size, MATTES[matte])
    back.alpha_composite(image)
    return sheet_data_url(back)


def url_of(path: str) -> str:
    with open(path, "rb") as handle:
        return "data:image/png;base64," + base64.b64encode(handle.read()).decode("ascii")


def contact_sheet(paths_by_size: dict, rows=SHEET_ROWS, gutter=SHEET_GUTTER):
    """One image holding the whole ladder, `(data url, members, size)` or None."""
    rows = [size for size in rows if paths_by_size.get(size)]
    columns = max((len(paths_by_size[size]) for size in rows), default=0)
    if len(rows) * columns < 2:
        return None
    loaded = []
    members = []
    for size in rows:
        row = []
        for candidate, lam, path in paths_by_size[size]:
            with Image.open(path) as handle:
                row.append(handle.convert("RGBA").copy())
            members.append({"candidate": candidate, "lambda": lam, "px": size, "path": path})
        loaded.append((size, row))
    width = max(sum(image.width for image in row) + gutter * (len(row) - 1)
                for _, row in loaded)
    height = sum(max(image.height for image in row) for _, row in loaded) \
        + gutter * (len(loaded) - 1)
    sheet = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    y = 0
    for _, row in loaded:
        x = 0
        for image in row:
            sheet.alpha_composite(image, (x, y))
            x += image.width + gutter
        y += max(image.height for image in row) + gutter
    return sheet_data_url(sheet), members, (width, height)


def separate_images(members) -> list:
    return [{"candidate": item["candidate"], "lambda": item["lambda"], "px": item["px"],
             "url": url_of(item["path"])} for item in members]


def packet_shapes(icon) -> list:
    """Richest packet first, leanest last — the ladder the endpoint walks when it refuses.

    Shape 1 is the true artefact set, shape 2 the decision size alone, shape 3 one composite
    strip with the full record, shape 4 that strip with the record trimmed. A larger window
    answers with a richer shape; a small one answers with a leaner shape; a window too small
    for even the last refuses, and the refusal quotes the window it asked for.
    """
    paths = render_paths(icon)
    ladder = []
    every = [{"candidate": candidate, "lambda": lam, "px": size, "path": path}
             for size in sorted(paths, reverse=True) for candidate, lam, path in paths[size]]
    if every:
        ladder.append({"name": "renders", "kind": "separate", "lean": False,
                       "members": every, "images": separate_images(every)})
    decision_only = [item for item in every if item["px"] == max(paths)]
    if len(decision_only) > 1:
        ladder.append({"name": "renders-decision-size", "kind": "separate", "lean": False,
                       "members": decision_only, "images": separate_images(decision_only)})
    sheet = contact_sheet(paths)
    if sheet:
        url, members, size = sheet
        for name, lean in (("sheet", False), ("sheet-lean", True)):
            ladder.append({"name": name, "kind": "sheet", "lean": lean, "size": size,
                           "members": members,
                           "images": [{"url": url, "px": "sheet",
                                       "candidates": [item["candidate"]
                                                      for item in members]}]})
    return ladder


def layout_note(shape) -> str:
    """What the model is looking at, said in the packet rather than left to be inferred."""
    if shape["kind"] == "sheet":
        width, height = shape["size"]
        return (
            f"IMAGES: one contact sheet, {width}x{height} px. Columns are λ samples in "
            f"ascending order, left to right. Top row is each sample at the "
            f"{SHEET_ROWS[0]} px decision size, bottom row the same sample at the "
            f"{SHEET_ROWS[1]} px smallest reviewed size. Every render was pasted 1:1 with "
            f"its own alpha and nothing was scaled or tinted; the gutter between them is "
            f"background, not object."
        )
    listed = ", ".join(f"{item['candidate']} at {item['px']} px" for item in shape["images"])
    return (f"IMAGES: {len(shape['images'])} separate render(s), in this order — {listed}. "
            f"Each is the object on transparency, no frame.")


def objects_for(index: int) -> list:
    """The declared objects, rotated so the correct answer's letter moves.

    The closed set is `families.FAMILIES`' own `object` phrases — the project's words for
    what each family is, so a verdict about identification is a verdict about *this*
    declaration rather than about a vocabulary invented here.
    """
    import families
    names = list(families.FAMILIES)
    return [(name, families.FAMILIES[name]["object"]) for name in names[index:] + names[:index]]


def naming_body(shape, matte) -> dict:
    text = [
        "You are looking at renders of user-interface icons.",
        "You are deliberately NOT told what they are meant to be: naming them from their",
        "pixels is the whole question, so do not infer a category from context.",
        "", layout_note(shape), "",
        "WHAT TO ANSWER",
        "\n".join(f"- {item}" for item in NAMING_QUESTIONS),
        "", "Answer with JSON exactly in this shape, and nothing else:", NAMING_SHAPE,
        f"(The renders are shown over a {matte} background.)" if matte != "none" else "",
    ]
    return body(text, [{**image, "url": flatten(image["url"], matte)}
                       for image in shape["images"]])


def questions_body(icons, shape, matte, naming=None) -> dict:
    names = ", ".join(repr(name) for name in (naming or {}).get("names", [])) \
        or "(naming request did not answer)"
    text = [
        "You are validating icon candidates for a city-building game's HUD.",
        "Every claim below was measured by the pipeline that rendered them; do not",
        "re-derive it. Judge the artefacts and answer the questions.",
        "", "ICONS AND THEIR DECLARATIONS", json.dumps(icons, indent=2, sort_keys=True),
        "", layout_note(shape), "", JUSTIFY_NOTE.replace("{names}", names), "",
        "QUESTIONS",
        "\n".join(f"- ({item['key']}) {item['ask']}" for item in QUESTIONS), "",
        ("Key `per_icon` by the ICON id — the `id` of the top-level entry above (for "
         f"example `{icons[0]['id']}`), not by the id of any candidate. If your read of "
         "the five λ samples differs between them, say so in `notes` and set "
         "`ladder_is_one_object` to false: that difference is the finding."),
        "", "Answer with JSON exactly in this shape, and nothing else:", ANSWER_SHAPE,
    ]
    return body(text, [{**image, "url": flatten(image["url"], matte)}
                       for image in shape["images"]])


def read_body(url, options, matte, extra="") -> dict:
    """One forced choice: which of the declared objects does this render depict?

    One render per request, on purpose — the strip is not used for this question, because
    measured, the same model answered the strip `3/6` and called the bin, the ticket and the
    noise all "a ribbon of carriageway".
    """
    listed = "\n".join(f"  {LETTERS[i]}. {phrase}" for i, (_, phrase) in enumerate(options))
    text = [
        "You are shown ONE rendered icon from a city-building game's HUD.",
        "", "Declared objects from this icon set:", listed, "",
        "From the pixels alone, which single one does this render depict? Answer with its",
        "letter. If it depicts none of them, set `choice` to \"NONE\" — you are not required",
        "to choose, and a wrong choice is worse than NONE.",
        extra, "", "Answer with JSON exactly in this shape and nothing else:", READ_SHAPE,
        (f"(The render is shown over a {matte} background.)" if matte != "none" else ""),
    ]
    return body(text, [{"type": "image_url", "image_url": {"url": flatten(url, matte)}}])


def body(text_lines: list, content: list) -> dict:
    content = [{"type": "text", "text": "\n".join(item for item in text_lines if item)}] \
        + [item for item in content if item]
    return {
        "model": model_name() or "<VALIDATOR_MODEL unset>",
        "messages": [
            {"role": "system",
             "content": ("You are a careful design reviewer. You answer only from what the "
                         "image shows, you do not assume it is one of the options offered, "
                         "and you say NONE when it is none of them.")},
            {"role": "user", "content": content},
        ],
        "temperature": 0.0,
        "response_format": {"type": "json_object"},
    }


def endpoint(base: str) -> str:
    return base + "/chat/completions"


OVER_CONTEXT = re.compile(r"context size \((\d+)\s*tokens\)", re.I)
REQUEST_TOKENS = re.compile(r"request \((\d+)\s*tokens\)", re.I)


class ValidatorError(RuntimeError):
    """A refusal, carrying the body the server actually sent.

    A refusal that does not name its cause cannot be acted on, and a tool that reports
    "HTTP 400" for a packet that was twice the window is reporting the wrong thing.
    """

    def __init__(self, detail: str, status=None, body=""):
        super().__init__(detail)
        self.status = status
        self.body = body


def short_reason(error: ValidatorError) -> str:
    """The server's own sentence, dug out of however many times it was escaped."""
    text = error.body or str(error)
    for _ in range(4):
        start = text.find("{")
        if start < 0:
            break
        try:
            parsed = json.loads(text[start:])
        except ValueError:
            break
        message = (parsed.get("error") or {}).get("message") \
            if isinstance(parsed, dict) else None
        if isinstance(message, str) and "{" in message:
            text = message
            continue
        if isinstance(message, str):
            return message
        break
    return text[:300]


def over_context(error: ValidatorError):
    """`(request tokens, window tokens)` when a refusal was about size, else None."""
    matched = OVER_CONTEXT.search(error.body or "")
    if not matched:
        return None
    asked = REQUEST_TOKENS.search(error.body or "")
    return (int(asked.group(1)) if asked else None, int(matched.group(1)))


def served_context(base: str, model: str, timeout: float = 8.0):
    """The window the endpoint is *serving* the model with, or None.

    Not the model's own maximum: the model this was first pointed at declares 128,000
    tokens in its card and is served at 4,096, and it is the served number that refuses a
    packet. Ollama reports it at `/api/ps`. A hosted gateway usually will not, and then
    `VALIDATOR_CONTEXT` is the only honest source — the tool does not guess a window.
    """
    root = base[:-3] if base.endswith("/v1") else base
    try:
        with urllib.request.urlopen(root + "/api/ps", timeout=timeout) as response:
            loaded = json.loads(response.read().decode("utf-8"))
    except (urllib.error.URLError, OSError, ValueError):
        return None
    for entry in loaded.get("models", []):
        if model in (entry.get("name"), entry.get("model")):
            return entry.get("context_length")
    return None


def reachable(base: str, timeout: float = 4.0) -> tuple:
    """`(ok, detail)` — whether there is an endpoint at all, without sending a verdict."""
    request = urllib.request.Request(base + "/models")
    if api_key():
        request.add_header("Authorization", f"Bearer {api_key()}")
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            return True, f"HTTP {response.status}"
    except urllib.error.HTTPError as error:
        return True, f"HTTP {error.code}"
    except (urllib.error.URLError, OSError, ValueError) as error:
        return False, f"{type(error).__name__}: {error}"


def send(payload: dict, timeout: float = 600.0) -> dict:
    request = urllib.request.Request(
        endpoint(base_url()), data=json.dumps(payload).encode("utf-8"),
        headers={"Content-Type": "application/json"}, method="POST")
    if api_key():
        request.add_header("Authorization", f"Bearer {api_key()}")
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            return json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as error:
        text = error.read().decode("utf-8", "replace")
        raise ValidatorError(f"HTTP {error.code}: {text[:400]}", status=error.code,
                             body=text) from error


def parse_answer(reply: dict) -> dict:
    """The model's JSON, or an explicit record that it did not give any."""
    choices = reply.get("choices") or []
    if not choices:
        return {"parsed": False, "why": "the reply carried no choices", "raw": reply}
    choice = choices[0]
    content = (choice.get("message") or {}).get("content") or ""
    result = {"finish_reason": choice.get("finish_reason")}
    try:
        result.update({"parsed": True, "answer": json.loads(content)})
    except (json.JSONDecodeError, TypeError):
        result.update({"parsed": False, "why": "the answer was not JSON", "raw": content})
    return result


def append_ledger(record: dict) -> None:
    os.makedirs(OUT, exist_ok=True)
    with open(LEDGER, "a", encoding="utf-8") as handle:
        handle.write(json.dumps(record, sort_keys=True) + "\n")


def exchange(record: dict, payload: dict) -> dict:
    """Send one payload, record what came back, and return the finished record.

    Every step the endpoint refuses is kept with the numbers it gave, so the shape a
    verdict was given about is never in doubt and a refusal names its cause. A window too
    small for any shape is recorded as exactly that rather than as a pass.
    """
    try:
        reply = send(payload)
    except ValidatorError as error:
        too_big = over_context(error)
        record.update({"answered": False, "why": short_reason(error),
                       "status": error.status,
                       "request_tokens": too_big[0] if too_big else None,
                       "window_tokens": too_big[1] if too_big else None})
        return record
    usage = reply.get("usage") or {}
    parsed = parse_answer(reply)
    window = record.get("window")
    record.update({
        "prompt_tokens": usage.get("prompt_tokens"),
        "answer_tokens": usage.get("completion_tokens"),
        "finish_reason": parsed.get("finish_reason"),
        "headroom_tokens": (window - usage["prompt_tokens"]
                            if window and usage.get("prompt_tokens") else None),
        "answered": parsed["parsed"],
        **({"answer": parsed["answer"]} if parsed["parsed"]
           else {"why": parsed["why"], "raw": parsed.get("raw")}),
    })
    return record


def base_record(kind: str, icon_id: str, payload: dict, matte: str, window) -> dict:
    return {"when": now(), "kind": kind, "icon": icon_id, "model": model_name(),
            "endpoint": base_url(), "matte": matte, "window": window,
            "asked": digest(payload)}


def ask_ladder(icon, window, kind, shapes, matte="none", naming=None) -> dict:
    """Offer packets richest-first and return the record of the one that was answered."""
    attempts = []
    for shape in shapes:
        payload = (naming_body(shape, matte) if kind == "naming"
                   else questions_body([icon], shape, matte, naming))
        record = base_record(kind, icon["id"], payload, matte, window)
        record.update({"packet": shape["name"], "lean": shape["lean"],
                       "ladder": shape["kind"], "images": len(shape["images"])})
        exchange(record, payload)
        if record["answered"]:
            record["attempts"] = attempts
            return read_record(record, silhouette_icon=icon)
        attempts.append({"shape": shape["name"], "why": record.get("why"),
                         "request_tokens": record.get("request_tokens"),
                         "window_tokens": record.get("window_tokens")})
        print(f"    {shape['name']}: refused ({record.get('why')})")
    return {"when": now(), "kind": kind, "icon": icon["id"], "model": model_name(),
            "endpoint": base_url(), "matte": matte, "window": window,
            "answered": False, "packet": None, "attempts": attempts,
            "why": ("no packet shape fitted the endpoint's window; the smallest one it "
                    "refused is quoted in `attempts`")}


def read_record(record: dict, silhouette_icon=None) -> dict:
    """Attach the read form of an answer to its record.

    A naming answer is read **in order** — it is a list, one entry per sample left to
    right, with nothing to key it by, which is why the packet states the column order. A
    questions answer is read by icon id, falling back to candidate ids with the keys the
    model read differently kept visible: a model that reads the md1 end and the iOS 6 end
    differently has found a ladder that is not one object.
    """
    if not record.get("answered"):
        return record
    answer = record.get("answer") or {}
    if record["kind"] == "naming":
        names = answer.get("names")
        record["verdict"] = {
            "keyed_by": "order",
            "names": names if isinstance(names, list) else None,
            "background": answer.get("background"),
            "why": None if isinstance(names, list) and names
                   else "the naming answer carried no list of names",
        }
        return record
    if record["kind"] == "questions" and silhouette_icon is not None:
        per_icon = answer.get("per_icon") or {}
        direct = per_icon.get(silhouette_icon["id"])
        if isinstance(direct, dict):
            record["verdict"] = {"keyed_by": "icon", "values": direct}
            return record
        entries = {candidate["id"]: per_icon[candidate["id"]]
                   for candidate in silhouette_icon["candidates"]
                   if isinstance(per_icon.get(candidate["id"]), dict)}
        if entries:
            keys = sorted({key for entry in entries.values() for key in entry})
            differing = [key for key in keys
                         if len({json.dumps(entry.get(key), sort_keys=True)
                                 for entry in entries.values()}) > 1]
            values = dict(next(iter(entries.values())))
            if "ladder_is_one_object" in differing:
                values["ladder_is_one_object"] = None
            record["verdict"] = {"keyed_by": "candidate", "values": values,
                                 "candidates_answered": len(entries),
                                 "differing_between_candidates": differing}
        else:
            record["verdict"] = {"keyed_by": None, "values": None,
                                 "why": f"no entry for {silhouette_icon['id']} or its "
                                        f"candidates"}
    return record


def reference_url(icon_id: str) -> str:
    """The declared MD1 mark this icon is built against, at the decision size.

    This is the calibration's yardstick: the corpus object the family was read off. If a
    validator identifies these and not ours, the miss is the drawing's; if it misses both,
    it cannot be quoted for either.
    """
    import shapes
    glyph = shapes.SILHOUETTES[icon_id]["glyph"]
    with Image.open(shapes.md1_path(glyph)) as handle:
        return sheet_data_url(handle.convert("RGBA"))


def noise_url(size=(96, 96)) -> str:
    return sheet_data_url(Image.effect_noise(size, 60).convert("RGBA"))


def forced_choice(label: str, url: str, options, matte, window, extra="") -> dict:
    """One scored forced choice, recorded whichever way it goes."""
    payload = read_body(url, options, matte, extra)
    record = base_record("read", label, payload, matte, window)
    record.update({"options": [phrase for _, phrase in options], "position": None})
    exchange(record, payload)
    if not record["answered"]:
        return record
    chosen = (record["answer"].get("choice") or "").strip().upper()
    if chosen in LETTERS[:len(options)]:
        family, phrase = options[LETTERS.index(chosen)]
        record.update({"chosen": family, "chosen_phrase": phrase,
                       "chosen_letter": chosen,
                       "position": LETTERS.index(chosen) + 1})
    else:
        record.update({"chosen": None if chosen in ("NONE", "") else chosen,
                       "chosen_phrase": None, "chosen_letter": chosen, "position": None})
    return record


def calibrate(window, icons, matte) -> dict:
    """Put the corpus marks and a noise control through the identical question.

    The reference score is the **ceiling** our own scores are read against, measured in the
    same run with the same matte, rotation and model — so nothing here is a threshold
    invented to make a result look good. The noise control is a binary: a validator that
    calls noise one of the objects is reported as uncalibrated and its verdicts are not
    quoted.
    """
    print("CALIBRATION (corpus reference marks, identical question)")
    summary = {"reference": [], "noise": None, "ceiling": None, "calibrated": None}
    for index, icon in enumerate(icons):
        options = objects_for(index)
        record = forced_choice(f"reference/{icon['id']}", reference_url(icon["id"]),
                               options, matte, window,
                               extra=("This is a reference mark from the icon set the "
                                      "objects were read off."))
        record["expected"] = _expected_family(icon["id"])
        record["hit"] = record.get("chosen") == record["expected"]
        append_ledger(record)
        summary["reference"].append({"icon": icon["id"], "chosen": record.get("chosen"),
                                     "expected": record.get("expected"),
                                     "hit": record["hit"],
                                     "name_it": (record.get("answer") or {}).get("name_it")})
        print(f"    {icon['id']:16} {str(record.get('chosen')):8} "
              f"expected {str(record.get('expected')):8} "
              f"{(record.get('answer') or {}).get('name_it') or record.get('why') or ''}")
    hits = sum(1 for row in summary["reference"] if row["hit"])
    summary["ceiling"] = f"{hits}/{len(summary['reference'])}"

    options = objects_for(len(icons))
    record = forced_choice("control/noise", noise_url(), options, matte, window,
                           extra="There is no guarantee this image depicts any of them.")
    record["expected"] = None
    record["hit"] = record.get("chosen") is None
    append_ledger(record)
    summary["noise"] = {"chosen": record.get("chosen"), "refused": record["hit"],
                        "name_it": (record.get("answer") or {}).get("name_it"),
                        "why": record.get("why")}
    print(f"    {'control noise':16} {str(record.get('chosen')):8} "
          f"expected {'NONE':8} {(record.get('answer') or {}).get('name_it') or ''}")
    summary["calibrated"] = bool(record["hit"])
    print(f"  ceiling: the validator identifies {summary['ceiling']} of the corpus's own "
          f"marks; noise: {'refused, as it must' if summary['calibrated'] else 'NOT refused'}")
    return summary


def _expected_family(icon_id: str) -> str:
    import families
    return families.ICON_FAMILY[icon_id]


def instrument(icon, window, matte) -> dict:
    """Read one icon at every λ sample: the scored, checkable form of 'does it read'."""
    import families
    index = list(families.ICON_FAMILY).index(icon["id"])
    expected = families.ICON_FAMILY[icon["id"]]
    paths = render_paths(icon)[96]
    per_sample = []
    for candidate, lam, path in paths:
        options = objects_for(index)
        record = forced_choice(f"{icon['id']}@{lam}", url_of(path), options, matte, window,
                              extra="This is one of five style-ladder samples of one icon.")
        record["expected"] = expected
        record["lambda"] = lam
        record["hit"] = record.get("chosen") == expected
        append_ledger(record)
        per_sample.append({"candidate": candidate, "lambda": lam,
                           "chosen": record.get("chosen"), "hit": record["hit"],
                           "answered_none": record.get("chosen_letter") == "NONE",
                           "name_it": (record.get("answer") or {}).get("name_it")})
        print(f"    λ={lam:<5} {str(record.get('chosen')):8} "
              f"{'ok ' if record['hit'] else '   '}"
              f"{(record.get('answer') or {}).get('name_it') or record.get('why') or ''}")
    chosen = {row["chosen"] for row in per_sample}
    return {"icon": icon["id"], "expected": expected, "samples": per_sample,
            "reads_as_own_object": all(row["hit"] for row in per_sample),
            "ladder_is_one_object": len(chosen) == 1 and None not in chosen,
            "distinct_reads": sorted(item for item in chosen if item)}


def main() -> int:
    # The report names the samples by λ, and this project's shell is cp1252 on Windows: a
    # console that cannot encode its own finding would fail the run at the last step.
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except (AttributeError, ValueError):
            pass
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--manifest", default=MANIFEST)
    parser.add_argument("--only", nargs="*", default=None,
                        help="icon ids to validate; default is every reviewed icon")
    parser.add_argument("--matte", default=DEFAULT_MATTE, choices=sorted(MATTES),
                        help="what transparency is composited onto (default white)")
    parser.add_argument("--dry-run", action="store_true",
                        help="build and print a packet without sending it")
    parser.add_argument("--reachable", action="store_true",
                        help="report whether an endpoint answers, and exit")
    parser.add_argument("--ladder", action="store_true",
                        help="print every packet shape and its size, and exit")
    parser.add_argument("--calibrate", action="store_true",
                        help="corpus marks and a noise control only, no verdicts on ours")
    parser.add_argument("--questions", action="store_true",
                        help="also ask the boolean questions (they pass noise; see the doc)")
    parser.add_argument("--naming", action="store_true",
                        help="also ask for the blind naming of each icon")
    arguments = parser.parse_args()

    if arguments.reachable:
        ok, detail = reachable(base_url())
        window = served_context(base_url(), model_name()) if model_name() else None
        print(f"{base_url()}: {'reachable' if ok else 'NOT reachable'} ({detail})")
        print(f"model: {model_name() or '<VALIDATOR_MODEL unset>'}")
        print(f"served context: "
              f"{window or declared_context() or '<unknown; set VALIDATOR_CONTEXT>'}")
        return 0 if (ok and model_name()) else 3

    if not os.path.exists(arguments.manifest):
        print(f"no manifest at {arguments.manifest}: render the review set first")
        return 2
    manifest = read_json(arguments.manifest)
    review_path = os.path.join(os.path.dirname(arguments.manifest), "review.json")
    review = read_json(review_path) if os.path.exists(review_path) else {}
    icons = evidence(manifest, file_map(review), lean=False)
    if arguments.only:
        wanted = set(arguments.only)
        icons = [icon for icon in icons if icon["id"] in wanted]
        if not icons:
            print(f"none of {sorted(wanted)} are in the manifest")
            return 2

    if arguments.ladder:
        for icon in icons:
            print(f"=== {icon['id']}")
            for shape in packet_shapes(icon):
                size = shape.get("size")
                print(f"    {shape['name']:24} {len(shape['images'])} image(s)"
                      f"{f' {size[0]}x{size[1]} px' if size else ''}"
                      f"{'  (lean record)' if shape['lean'] else ''}")
        print("\nrichest first: the endpoint's refusals decide which one is answered.")
        return 0

    if arguments.dry_run:
        icon = icons[0]
        payload = read_body(reference_url(icon["id"]), objects_for(0), arguments.matte)
        print(f"=== forced choice, one image, {len(json.dumps(payload))} bytes of request")
        print(payload["messages"][1]["content"][0]["text"][:900])
        print(f"\nnothing sent. Endpoint would be {endpoint(base_url())}, model "
              f"{model_name() or '<VALIDATOR_MODEL unset>'}, matte {arguments.matte}.")
        return 0

    if not model_name():
        print("VALIDATOR_MODEL is unset, so there is nothing to ask. This exit is a refusal "
              "on purpose: the local checks answer different questions, and recording them "
              "as a validation would make the word mean nothing.")
        return 2
    ok, detail = reachable(base_url())
    if not ok:
        print(f"no validator endpoint at {base_url()} ({detail}). Set VALIDATOR_BASE_URL to "
              f"one, or run a local OpenAI-compatible server.")
        return 3
    window = served_context(base_url(), model_name()) or declared_context()
    print(f"validator: {model_name()} at {base_url()}, served context "
          f"{window or 'unknown'}, matte {arguments.matte}")

    calibration = calibrate(window, icons, arguments.matte)
    if not calibration["calibrated"]:
        summary = {"when": now(), "kind": "summary", "model": model_name(),
                   "endpoint": base_url(), "matte": arguments.matte,
                   "calibration": calibration, "uncalibrated": True,
                   "why": ("the validator chose one of the declared objects for an image "
                           "that is not one, so its verdicts on the real renders cannot be "
                           "quoted")}
        append_ledger(summary)
        print("\nUNCALIBRATED: no verdicts recorded for the reviewed set. This is a "
              "refusal, not a pass — a validator that calls noise an object cannot tell "
              "the set apart from noise.")
        return 5

    if arguments.calibrate:
        append_ledger({"when": now(), "kind": "summary", "model": model_name(),
                       "endpoint": base_url(), "matte": arguments.matte,
                       "calibration": calibration, "calibrate_only": True})
        print(f"\ncalibration recorded in {LEDGER}. No verdicts on the reviewed set were "
              f"asked for.")
        return 0

    readings = []
    for icon in icons:
        print(f"  {icon['id']} (declared {_expected_family(icon['id'])}, "
              f"reference {icon.get('declared_mark')}, ceiling "
              f"{calibration['ceiling']})")
        reading = instrument(icon, window, arguments.matte)
        readings.append(reading)
        append_ledger({"when": now(), "kind": "reading", "model": model_name(),
                       "endpoint": base_url(), "matte": arguments.matte,
                       "window": window, "calibration": calibration, **reading})
        print(f"    reads as its own object: {reading['reads_as_own_object']}; "
              f"one object across λ: {reading['ladder_is_one_object']}"
              f"{'' if reading['ladder_is_one_object'] else ' (' + ', '.join(reading['distinct_reads']) + ')'}")

    if arguments.naming:
        for icon in icons:
            record = ask_ladder(icon, window, "naming", packet_shapes(icon),
                               matte=arguments.matte)
            append_ledger(record)
            names = (record.get("verdict") or {}).get("names") or []
            print(f"  naming {icon['id']} (background "
                  f"{(record.get('verdict') or {}).get('background')}):")
            for name in names:
                print(f"    \"{name}\"")
    if arguments.questions:
        for icon in icons:
            for shape in packet_shapes(icon):
                payload = questions_body([icon], shape, arguments.matte)
                record = base_record("questions", icon["id"], payload, arguments.matte,
                                     window)
                exchange(record, payload)
                if record["answered"]:
                    break
            record = read_record(record, silhouette_icon=icon)
            append_ledger(record)
            values = (record.get("verdict") or {}).get("values") or {}
            print(f"  questions {icon['id']}: reads_as_object="
                  f"{values.get('reads_as_object')} ladder="
                  f"{values.get('ladder_is_one_object')} notes={values.get('notes')!r}")

    identified = sum(1 for reading in readings if reading["reads_as_own_object"])
    ladders = sum(1 for reading in readings if reading["ladder_is_one_object"])
    misread = [{"icon": reading["icon"], "expected": reading["expected"],
                "read_as": reading["distinct_reads"]}
               for reading in readings if not reading["reads_as_own_object"]]
    broken = [reading["icon"] for reading in readings
              if not reading["ladder_is_one_object"]]
    summary = {"when": now(), "kind": "summary", "model": model_name(),
               "endpoint": base_url(), "matte": arguments.matte, "window": window,
               "calibration": calibration,
               "ceiling": calibration["ceiling"],
               "identified_as_own_object": identified, "icons": len(readings),
               "ladders_that_are_one_object": ladders,
               "misread": misread, "broken_ladders": broken}
    append_ledger(summary)
    print(f"\nceiling (corpus marks): {calibration['ceiling']}; ours: "
          f"{identified}/{len(readings)} read as their own object at every λ sample; "
          f"{ladders}/{len(readings)} are one object across λ. Recorded in {LEDGER}. "
          f"Nothing was promoted: a promotion stays a person's decision in the picker.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
