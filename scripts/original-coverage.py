#!/usr/bin/env python3
"""How much of the original's game logic our sources cite, per function and per line.

The decompiled main script (DoAction of sprite 6765) is split into functions. Only lines holding a
literal value count (gameplay lines); braces, plain calls and bookkeeping do not. A gameplay line
is covered in one of two ways:

Cited: a Rust comment quotes part of it in backticks (an expression of the original; no longer written)
within a function the same file names (`sloty()`), or anywhere if the quote is distinctive (in at
most a few lines); or a comment cites its line (`DoAction 26277`). A cited range
(`DoAction 26277–26291`) only covers the lines in it whose literals the citing file uses, so a
range around unported branches does not claim them: one of the line's literals must be uncommon
(on at most 25 gameplay lines, and not one of the small numbers every program uses). In a
function a file names, a line also counts when every literal on it appears in that file, which
is generous for lines with only small numbers.

Found (no citation needed): every literal on the line appears in one Rust file's code (comments
removed) in `src/game` or `src/game_assets`, and at least one of them is rare in the original (on at
most a few gameplay lines: `0.965`, `0.004`, sound names). Gameplay values must be named
constants (AGENTS.md), so porting a line's values makes it count without remembering to cite it.
`--suggest` lists these found-but-uncited lines per Rust file, to cite them.

Either way it is evidence, not proof, of a port. The summary gives a strict count (quotes, cited
lines, rare values) and the generous upper bound (also every line of a named function whose
values a file uses); the per-function lists use the upper bound.

Usage:
  scripts/original-coverage.py [--top N]          summary, fully covered and least covered functions
  scripts/original-coverage.py --function NAME    the function with each gameplay line marked
  scripts/original-coverage.py --suggest          lines found in the code but not cited, per file
  scripts/original-coverage.py --svg FILE         a map of every gameplay line, grouped by function
                                                  (green: covered, amber: only by the upper bound, grey: not)
"""
import argparse
import functools
import pathlib
import re

SCRIPTS = pathlib.Path.home() / ".cache/opentboi/decompiled/scripts"
# The game's logic (cited as `DoAction N`) and the main timeline's script (cited as `Timeline N`: sounds
# and music, saving, the main loop). Both are read as one list of lines, the timeline's after the game's.
GAME_SCRIPT = SCRIPTS / "DefineSprite_6765/frame_1/DoAction.as"
TIMELINE_SCRIPT = SCRIPTS / "frame_1/DoAction.as"
TIMELINE_LABEL = "Timeline"
SOURCES = pathlib.Path(__file__).resolve().parent.parent / "src"
FUNCTION_START = re.compile(r"^function (\w+)\(")
QUOTE = re.compile(r"`([^`]+)`")
CITED_FUNCTION = re.compile(r"^(\w+)\(")
CITED_LINES = re.compile(r"(DoAction|Timeline)[^0-9]{0,12}(\d{1,5})(?:\s*[–-]\s*(\d{1,5}))?")
# A literal other than 0 or 1: a number, or a string (sound names, texts).
LITERAL = re.compile(r'(?<![\w.])(?:\d*\.\d+|[2-9]\d*|1\d+)(?![\w.])|"[^"]+"')
MINIMUM_QUOTE_LENGTH = 6
MAXIMUM_QUOTE_MATCHES = 4
# A literal on at most this many gameplay lines is rare enough to identify a line by anywhere, and
# one on at most the second many identifies it within a range a file cites.
MAXIMUM_RARE_LITERAL_LINES = 3
MAXIMUM_UNCOMMON_LITERAL_LINES = 25
# Literals too common in any code to identify anything, even where the original uses them rarely.
COMMON_LITERALS = {"2", "3", "4", "5", "10", "100", "0.5", "1.5", "2.5"}
# A string identifies a line only from this length on ("h" or "A" is in any code).
MINIMUM_TELLING_STRING_LENGTH = 7  # quotes included
# The game's rules and assets; the frontends only play what these say.
PORT_DIRECTORIES = ("game", "game_assets")


def place(number, timeline_offset):
    """How a combined line number is cited: `DoAction 7053` or `Timeline 548`."""
    if number > timeline_offset:
        return f"{TIMELINE_LABEL} {number - timeline_offset}"
    return f"DoAction {number}"


def squeeze(text):
    return re.sub(r"\s+", "", text)


def functions(lines):
    """(name, first line, last line) per function, 1-based."""
    starts = [(number, match.group(1)) for number, line in enumerate(lines, 1) if (match := FUNCTION_START.match(line))]
    return [(name, first, (starts[index + 1][0] - 1 if index + 1 < len(starts) else len(lines)))
            for index, (first, name) in enumerate(starts)]


def citations(timeline_offset):
    """Per Rust file: the functions it names, the expressions it quotes, the line ranges it cites
    and its code without comments; and every function named anywhere."""
    files, names = [], set()
    for path in SOURCES.rglob("*.rs"):
        file_names, file_quotes, ranges = set(), set(), []
        text = path.read_text()
        for comment in re.findall(r"//.*", text):
            for quote in QUOTE.findall(comment):
                if match := CITED_FUNCTION.match(quote):
                    file_names.add(match.group(1))
                if len(squeeze(quote)) >= MINIMUM_QUOTE_LENGTH:
                    file_quotes.add(squeeze(quote))
            for label, first, last in CITED_LINES.findall(comment):
                shift = timeline_offset if label == TIMELINE_LABEL else 0
                ranges.append((int(first) + shift, int(last or first) + shift))
        names |= file_names
        code = re.sub(r"//.*", "", text)
        files.append((path, file_names, file_quotes, ranges, text, code))
    return files, names


@functools.lru_cache(maxsize=None)
def literals_of(line):
    """The literals on a line of the original (cached: every pass asks about every line)."""
    return tuple(LITERAL.findall(line))


@functools.lru_cache(maxsize=None)
def literal_in(literal, text, strings_quoted):
    """Whether `text` holds the literal (a sound name with any extension; with `strings_quoted`, only
    inside a string, not as part of a name). Cached: the same few texts are asked about all the time."""
    if literal.startswith('"'):
        stem = literal.strip('"').rsplit(".", 1)[0]
        return (('"' + stem) if strings_quoted else stem) in text
    return literal in numbers_in(text)


@functools.lru_cache(maxsize=None)
def numbers_in(text):
    """Every digit-and-dot string standing alone in `text` (not after a word character or a dot, not
    followed by a digit), prefixes of longer runs included: "2.5" holds "2" and "2.5"."""
    found = set()
    for run in re.finditer(r"(?<![\w.])[\d.]+", text):
        word = run.group()
        for end in range(1, len(word) + 1):
            if end == len(word) or not word[end].isdigit():
                found.add(word[:end])
    return found


def literals_found(line, text, strings_quoted=False):
    """Whether every literal of a gameplay line appears in `text`."""
    literals = literals_of(line)
    return bool(literals) and all(literal_in(literal, text, strings_quoted) for literal in literals)


def parameters_code():
    """`GameParameters`' code: gameplay values live there (AGENTS.md), so they count for every
    file. Only their code: the comments there quote many unrelated expressions."""
    return "".join(re.sub(r"//.*", "", path.read_text()) for path in (SOURCES / "game/parameters").rglob("*.rs"))


def literals_on_at_most(lines, maximum):
    """Literals on at most `maximum` gameplay lines of the original."""
    counts = {}
    for line in lines:
        for literal in set(literals_of(line)):
            counts[literal] = counts.get(literal, 0) + 1
    return {literal for literal, count in counts.items()
            if count <= maximum and literal not in COMMON_LITERALS
            and (not literal.startswith('"') or len(literal) >= MINIMUM_TELLING_STRING_LENGTH)}


def identified(line, text, telling):
    """Every literal of `line` is in `text` and one of them is in `telling`."""
    return literals_found(line, text) and any(literal in telling for literal in literals_of(line))


def found_lines(lines, files):
    """Per Rust file of the port, the gameplay lines whose literals its code all uses, one of them
    rare (values kept in `GameParameters` are found in its files)."""
    rare = literals_on_at_most(lines, MAXIMUM_RARE_LITERAL_LINES)
    candidates = [(number, line) for number, line in enumerate(lines, 1)
                  if any(literal in rare for literal in literals_of(line))]
    found = {}
    port = [SOURCES / directory for directory in PORT_DIRECTORIES]
    for path, _, _, _, _, code in files:
        if not any(directory in path.parents for directory in port):
            continue
        hits = {number for number, line in candidates if literals_found(line, code, strings_quoted=True)}
        if hits:
            found[path] = hits
    return found


def covered_lines(lines, spans, files, generous=True):
    """A quote counts where it occurs inside a function its file names; elsewhere only if it is
    distinctive (found in at most a few lines). A cited range counts its lines that are quoted or
    whose literals the file uses."""
    squeezed = [squeeze(line) for line in lines]
    span_of = {}
    for name, first, last in spans:
        span_of.setdefault(name, []).append((first, last))
    covered = set()
    matches_of = {}
    parameters = parameters_code()
    uncommon = literals_on_at_most(lines, MAXIMUM_UNCOMMON_LITERAL_LINES)
    for _, file_names, file_quotes, ranges, text, _ in files:
        if not (file_names or file_quotes or ranges):
            continue
        context = [span for name in file_names for span in span_of.get(name, [])]
        searched = text + parameters
        for first, last in ranges:
            if first == last:
                covered.add(first)
                continue
            covered.update(number for number in range(first, last + 1)
                           if identified(lines[number - 1], searched, uncommon)
                           or any(quote in squeezed[number - 1] for quote in file_quotes))
        for first, last in context:
            covered.update(number for number in range(first, last + 1)
                           if (literals_found if generous else lambda line, text: identified(line, text, uncommon))(
                               lines[number - 1], searched))
        for quote in file_quotes:
            if quote not in matches_of:
                matches_of[quote] = [number for number, line in enumerate(squeezed, 1) if quote in line]
            matches = matches_of[quote]
            in_context = [number for number in matches if any(first <= number <= last for first, last in context)]
            if in_context:
                covered.update(in_context)
                continue
            if len(matches) <= MAXIMUM_QUOTE_MATCHES:
                covered.update(matches)
    return covered


def report(spans, lines, covered, names, top, found_count, strict_count, offset):
    rows = []
    for name, first, last in spans:
        gameplay = [number for number in range(first, last + 1) if LITERAL.search(lines[number - 1])]
        if gameplay:
            hit = sum(number in covered for number in gameplay)
            rows.append((name, first, len(gameplay), hit, name in names))
    total = sum(row[2] for row in rows)
    hit = sum(row[3] for row in rows)
    touched = [row for row in rows if row[3] or row[4]]
    print(f"Gameplay lines covered: at least {strict_count} ({100 * strict_count / total:.1f} %), at most {hit} "
          f"({100 * hit / total:.1f} %) of {total}; {found_count} of them found in the code without a citation; "
          f"functions touched: {len(touched)} of {len(rows)}")
    print("(At least: quotes, cited lines and values rare in the original. At most: also every line of a named "
          "function whose values a file uses. The lists below use the upper bound.)")
    complete = sorted((row for row in rows if row[3] == row[2]), key=lambda row: row[0])
    print(f"\nFully covered ({len(complete)}): " + ", ".join(f"{row[0]}()" for row in complete))
    partial = sorted((row for row in touched if row[3] < row[2]), key=lambda row: row[3] / row[2])
    print(f"\nTouched but with blind spots (least covered first, top {top}):")
    for name, first, count, hit_count, _ in partial[:top]:
        print(f"  {name}() {place(first, offset)}: {hit_count}/{count} gameplay lines ({100 * hit_count / count:.0f} %)")
    untouched = sorted((row for row in rows if not (row[3] or row[4])), key=lambda row: -row[2])
    print(f"\nNot touched, largest first (top {top}):")
    for name, first, count, _, _ in untouched[:top]:
        print(f"  {name}() {place(first, offset)}: {count} gameplay lines")


def show_function(spans, lines, covered, wanted, offset):
    matches = [(first, last) for name, first, last in spans if name == wanted]
    if not matches:
        raise SystemExit(f"no function {wanted}()")
    for first, last in matches:
        for number in range(first, last + 1):
            line = lines[number - 1]
            mark = " " if not LITERAL.search(line) else ("+" if number in covered else "-")
            print(f"{mark} {place(number, offset):>14} {line}")
    print("\n+ covered gameplay line, - blind spot")


def suggest(lines, spans, found, cited, offset):
    """Found-but-uncited lines per Rust file, with the original function each is in."""
    function_of = {number: name for name, first, last in spans for number in range(first, last + 1)}
    for path, numbers in sorted(found.items()):
        uncited = sorted(number for number in numbers - cited if LITERAL.search(lines[number - 1]))
        if not uncited:
            continue
        print(f"{path.relative_to(SOURCES.parent)}:")
        for number in uncited:
            print(f"  {place(number, offset)} in {function_of.get(number, '(top level)')}(): {lines[number - 1].strip()}")


SVG_WIDTH = 1000
SVG_SQUARE = 4.0
SVG_GAP = 0.6
SVG_BLOCK_GAP = 8
SVG_LABEL_HEIGHT = 14
SVG_LABEL = "#c9d1d9"
SVG_TIMELINE_LABEL = "#58a6ff"
SVG_COLORS = {"strict": "#2ea043", "bound": "#d29922", "none": "#6e7681"}


def block_columns(count):
    """Columns for a function's block: roughly twice as wide as tall."""
    return max(1, min(count, round((count * 2) ** 0.5)))


def write_svg(path, spans, lines, covered, strict, offset):
    """One square per gameplay line, a block per function (largest first, packed in rows), labeled with
    the function's name and coverage. Only names and counts: none of the original's code. The main
    timeline's functions are labeled in blue and say so in their tooltips."""
    blocks = []
    for name, first, last in spans:
        numbers = [number for number in range(first, last + 1) if LITERAL.search(lines[number - 1])]
        if numbers:
            blocks.append((name, numbers, first > offset))
    blocks.sort(key=lambda block: -len(block[1]))
    pitch = SVG_SQUARE + SVG_GAP
    total = sum(len(numbers) for _, numbers, _ in blocks)
    exact = sum(number in strict for _, numbers, _ in blocks for number in numbers)
    bound = sum(number in covered for _, numbers, _ in blocks for number in numbers)
    body, x, y, row_height = [], 1.0, 34.0, 0.0
    for name, numbers, in_timeline in blocks:
        columns = block_columns(len(numbers))
        rows = -(-len(numbers) // columns)
        width = max(columns * pitch, len(name) * 6.0 + 4)
        if x + width > SVG_WIDTH:
            x, y, row_height = 1.0, y + row_height + SVG_BLOCK_GAP, 0.0
        hit = sum(number in covered for number in numbers)
        where = " (main timeline)" if in_timeline else ""
        body.append(f'<g><title>{name}(){where}: {hit}/{len(numbers)} ({100 * hit // len(numbers)} %)</title>')
        if in_timeline or len(numbers) >= columns * 3 or width > columns * pitch:
            color = SVG_TIMELINE_LABEL if in_timeline else SVG_LABEL
            body.append(f'<text x="{x:.1f}" y="{y + 9:.1f}" font-size="10" fill="{color}">{name}</text>')
        for index, number in enumerate(numbers):
            state = "strict" if number in strict else "bound" if number in covered else "none"
            body.append(f'<rect x="{x + (index % columns) * pitch:.2f}" '
                        f'y="{y + SVG_LABEL_HEIGHT + (index // columns) * pitch:.2f}" '
                        f'width="{SVG_SQUARE}" height="{SVG_SQUARE}" fill="{SVG_COLORS[state]}"/>')
        body.append("</g>")
        row_height = max(row_height, SVG_LABEL_HEIGHT + rows * pitch)
        x += width + SVG_BLOCK_GAP
    height = y + row_height + 4
    title = (f"Original gameplay lines: at least {100 * exact / total:.1f} % ({exact}), "
             f"at most {100 * bound / total:.1f} % ({bound}) of {total}")
    path.write_text(
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {SVG_WIDTH} {height:.0f}" font-family="sans-serif">'
        f'<rect width="100%" height="100%" fill="#0d1117"/>'
        f'<text x="4" y="21" font-size="16" fill="#ffffff">{title}</text>'
        f'<text x="{SVG_WIDTH - 4}" y="21" font-size="10" fill="{SVG_TIMELINE_LABEL}" text-anchor="end">'
        f'blue names: the main timeline script (sounds, music, saving, the main loop)</text>' + "".join(body) + "</svg>\n")
    print(f"Wrote {path}: {title}")


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--top", type=int, default=20)
    parser.add_argument("--function", help="show one function with its lines marked")
    parser.add_argument("--svg", type=pathlib.Path, help="write a map of every gameplay line to this file")
    parser.add_argument("--suggest", action="store_true", help="list lines found in the code but not cited")
    arguments = parser.parse_args()
    lines = GAME_SCRIPT.read_text(errors="replace").splitlines()
    offset = len(lines)
    lines += TIMELINE_SCRIPT.read_text(errors="replace").splitlines()
    files, names = citations(offset)
    spans = functions(lines)
    game_names = {name for name, first, _ in spans if first <= offset}
    # A timeline function sharing a name with a game function is cited as `timeline_NAME()`.
    spans = [(f"timeline_{name}" if first > offset and name in game_names else name, first, last)
             for name, first, last in spans]
    cited = covered_lines(lines, spans, files)
    found = found_lines(lines, files)
    found_anywhere = set().union(*found.values()) if found else set()
    covered = cited | found_anywhere
    gameplay = {number for number, line in enumerate(lines, 1) if LITERAL.search(line)}
    if arguments.svg:
        strict = (covered_lines(lines, spans, files, generous=False) | found_anywhere) & gameplay
        write_svg(arguments.svg, spans, lines, covered, strict, offset)
    elif arguments.suggest:
        suggest(lines, spans, found, cited, offset)
    elif arguments.function:
        show_function(spans, lines, covered, arguments.function, offset)
    else:
        found_count = len((found_anywhere - cited) & gameplay)
        strict = (covered_lines(lines, spans, files, generous=False) | found_anywhere) & gameplay
        report(spans, lines, covered, names, arguments.top, found_count, len(strict), offset)


if __name__ == "__main__":
    main()
