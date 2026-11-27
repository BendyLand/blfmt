#!/usr/bin/env bash
set -euo pipefail

# Move to repository root regardless of where the script is invoked from
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

BIN="./blfmt"
OUT_FILE="diffs_fmt.diff"
STYLE=""
DO_BUILD=0
WRITE_TESTS=0
RESTORE_ONLY=0
TARGETS=()

usage() {
	cat <<'EOF'
Usage: ./check_fmt.sh [options] [suite_or_file ...]

Runs blfmt against C/C++ files in tests/, compares the formatted output against
the original reference files in safe/ using unified diff, prints a status table,
and writes the overview + full diffs to a report file.

Targets:
  todo | easy | adv | long  Run only the specified test suite(s) (default: todo easy adv)
  path/to/file              Run on specific file(s), e.g. tests/todo/commands.c

Options:
  -o, --output <file>     Output report path (default: diffs_fmt.diff)
  -b, --build             Rebuild ./blfmt with cargo before running
  -s, --style <style>     Pass style to blfmt (stroustrup, allman, knr)
  -w, --write             Keep formatted output written in tests/ files
  -r, --restore           Restore tests/ C/C++ files from safe/ and exit
  -h, --help              Show this help message
EOF
}

is_c_cpp_file() {
	case "$1" in
		*.c|*.h|*.cpp|*.cc|*.C|*.hpp|*.hh|*.H) return 0 ;;
		*) return 1 ;;
	esac
}

restore_all_from_safe() {
	local count=0
	while IFS= read -r -d '' safe_file; do
		local rel="${safe_file#safe/}"
		if is_c_cpp_file "$rel"; then
			mkdir -p "$(dirname "tests/$rel")"
			cp "$safe_file" "tests/$rel"
			count=$((count + 1))
		fi
	done < <(find safe -type f -print0 | sort -z)
	echo "Restored $count C/C++ files in tests/ from safe/."
}

while [[ $# -gt 0 ]]; do
	case "$1" in
		-o|--output)
			[[ $# -ge 2 ]] || { echo "Error: $1 requires a file path." >&2; exit 1; }
			OUT_FILE="$2"
			shift 2
			;;
		-b|--build)
			DO_BUILD=1
			shift
			;;
		-s|--style)
			[[ $# -ge 2 ]] || { echo "Error: $1 requires a style name." >&2; exit 1; }
			STYLE="$2"
			shift 2
			;;
		-w|--write)
			WRITE_TESTS=1
			shift
			;;
		-r|--restore)
			RESTORE_ONLY=1
			shift
			;;
		-h|--help)
			usage
			exit 0
			;;
		-*)
			echo "Unknown option: $1" >&2
			usage >&2
			exit 1
			;;
		*)
			TARGETS+=("$1")
			shift
			;;
	esac
done

if [[ ! -d safe || ! -d tests ]]; then
	echo "Error: expected both 'safe/' and 'tests/' directories in $SCRIPT_DIR" >&2
	exit 1
fi

if [[ "$RESTORE_ONLY" -eq 1 ]]; then
	restore_all_from_safe
	exit 0
fi

if [[ "$DO_BUILD" -eq 1 || ! -x "$BIN" ]]; then
	echo "Building blfmt..."
	cargo build
	cp target/debug/blfmt "$BIN"
fi

if [[ "${#TARGETS[@]}" -eq 0 ]]; then
	TARGETS=("todo" "easy" "adv")
fi

# Collect relative paths (under safe/ and tests/) in suite order
REL_FILES=()
for target in "${TARGETS[@]}"; do
	cleaned="${target#./}"
	cleaned="${cleaned#tests/}"
	cleaned="${cleaned#safe/}"

	if [[ -d "safe/$cleaned" ]]; then
		while IFS= read -r -d '' f; do
			rel="${f#safe/}"
			if is_c_cpp_file "$rel"; then
				REL_FILES+=("$rel")
			fi
		done < <(find "safe/$cleaned" -maxdepth 2 -type f -print0 | sort -z)
	elif [[ -f "safe/$cleaned" ]]; then
		if is_c_cpp_file "$cleaned"; then
			REL_FILES+=("$cleaned")
		else
			echo "Skipping unsupported non-C/C++ file: $target" >&2
		fi
	else
		echo "Error: target '$target' not found under safe/" >&2
		exit 1
	fi
done

if [[ "${#REL_FILES[@]}" -eq 0 ]]; then
	echo "No C/C++ test files matched." >&2
	exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

SUMMARY_FILE="$TMP_DIR/summary.txt"
AST_WARN_FILE="$TMP_DIR/ast_warnings.txt"
DIFF_BODY_FILE="$TMP_DIR/diffs.patch"

: > "$SUMMARY_FILE"
: > "$AST_WARN_FILE"
: > "$DIFF_BODY_FILE"

TOTAL_FILES=0
MATCH_FILES=0
DIFF_FILES=0
AST_WARN_FILES=0
ERROR_FILES=0
TOTAL_AST_WARNS=0

BLFMT_EXTRA_ARGS=()
if [[ -n "$STYLE" ]]; then
	BLFMT_EXTRA_ARGS+=("-s" "$STYLE")
fi

printf -v TABLE_HEADER "%-30s %-10s %8s %8s %15s %9s" \
	"FILE" "STATUS" "ORIG_LN" "FMT_LN" "DIFF (+A/-R)" "AST_WARN"
TABLE_SEP="----------------------------------------------------------------------------------------"

{
	echo "$TABLE_HEADER"
	echo "$TABLE_SEP"
} >> "$SUMMARY_FILE"

CURRENT_SUITE=""

for rel in "${REL_FILES[@]}"; do
	TOTAL_FILES=$((TOTAL_FILES + 1))
	safe_path="safe/$rel"
	test_path="tests/$rel"
	suite="${rel%%/*}"

	# Ensure tests/<rel> starts identical to safe/<rel> before invoking blfmt
	mkdir -p "$(dirname "$test_path")"
	cp "$safe_path" "$test_path"

	fmt_out="$TMP_DIR/fmt_out"
	err_out="$TMP_DIR/err_out"
	ast_hits_file="$TMP_DIR/ast_hits"
	file_diff="$TMP_DIR/file.diff"

	set +e
	"$BIN" "$test_path" "${BLFMT_EXTRA_ARGS[@]}" >"$fmt_out" 2>"$err_out"
	rc=$?
	set -e

	# Detect unhandled AST println! diagnostics from c_ast.rs / cpp_ast.rs
	grep -nE "^(You shouldn't be here[: ]|Unknown (c |cpp )?grammar name|Using declaration:|Gnu asm expression:)" "$fmt_out" >"$ast_hits_file" || true
	ast_warn_count="$(wc -l <"$ast_hits_file" | tr -d ' ')"
	TOTAL_AST_WARNS=$((TOTAL_AST_WARNS + ast_warn_count))

	orig_lines="$(wc -l <"$safe_path" | tr -d ' ')"
	fmt_lines="$(wc -l <"$fmt_out" | tr -d ' ')"

	set +e
	diff -u --label "safe/$rel" --label "formatted/$rel" "$safe_path" "$fmt_out" >"$file_diff"
	diff_rc=$?
	set -e

	added=0
	removed=0
	if [[ "$diff_rc" -eq 1 ]]; then
		added="$(grep -cE '^\+[^+]|^\+$' "$file_diff" || true)"
		removed="$(grep -cE '^-[^-]|^-$' "$file_diff" || true)"
	fi

	status="MATCH"
	if [[ "$rc" -ne 0 || -s "$err_out" ]]; then
		status="ERROR"
		ERROR_FILES=$((ERROR_FILES + 1))
	elif [[ "$ast_warn_count" -gt 0 ]]; then
		status="AST_WARN"
		AST_WARN_FILES=$((AST_WARN_FILES + 1))
	elif [[ "$diff_rc" -eq 1 ]]; then
		status="DIFF"
		DIFF_FILES=$((DIFF_FILES + 1))
	else
		MATCH_FILES=$((MATCH_FILES + 1))
	fi

	if [[ "$rc" -ne 0 || -s "$err_out" || "$ast_warn_count" -gt 0 ]]; then
		{
			echo "[$rel] (exit=$rc, ast_warnings=$ast_warn_count)"
			if [[ -s "$err_out" ]]; then
				sed 's/^/  stderr: /' "$err_out"
			fi
			if [[ "$ast_warn_count" -gt 0 ]]; then
				sed 's/^/  ast:    /' "$ast_hits_file"
			fi
			echo ""
		} >> "$AST_WARN_FILE"
	fi

	if [[ "$diff_rc" -eq 1 ]]; then
		if [[ "$suite" != "$CURRENT_SUITE" ]]; then
			{
				echo "################################################################################"
				echo "# SUITE: $suite"
				echo "################################################################################"
				echo ""
			} >> "$DIFF_BODY_FILE"
			CURRENT_SUITE="$suite"
		fi
		{
			echo "================================================================================"
			echo "FILE: $rel (+$added / -$removed lines)"
			echo "================================================================================"
			cat "$file_diff"
			echo ""
		} >> "$DIFF_BODY_FILE"
	fi

	if [[ "$WRITE_TESTS" -eq 1 ]]; then
		cp "$fmt_out" "$test_path"
	fi

	delta_str="+${added}/-${removed}"
	printf "%-30s %-10s %8d %8d %15s %9d\n" \
		"$rel" "$status" "$orig_lines" "$fmt_lines" "$delta_str" "$ast_warn_count" >> "$SUMMARY_FILE"
done

{
	echo "$TABLE_SEP"
	echo "Totals: $TOTAL_FILES files | $MATCH_FILES identical | $DIFF_FILES with diffs | $AST_WARN_FILES with AST warnings ($TOTAL_AST_WARNS total) | $ERROR_FILES runtime errors"
} >> "$SUMMARY_FILE"

# Assemble full report file
{
	echo "blfmt AST & Formatting Diff Report"
	echo "Generated: $(date -Iseconds)"
	echo "Style:     ${STYLE:-stroustrup (default)}"
	echo "Targets:   ${TARGETS[*]}"
	echo ""
	cat "$SUMMARY_FILE"
	echo ""
	echo "=== AST PARSING & RUNTIME DIAGNOSTICS ==="
	if [[ -s "$AST_WARN_FILE" ]]; then
		cat "$AST_WARN_FILE"
	else
		echo "None — 0 unhandled AST nodes or runtime errors across all $TOTAL_FILES tested files."
	fi
	echo ""
	echo "=== UNIFIED DIFFS (safe/ vs formatted) ==="
	if [[ -s "$DIFF_BODY_FILE" ]]; then
		cat "$DIFF_BODY_FILE"
	else
		echo "No formatting differences found."
	fi
} > "$OUT_FILE"

cat "$SUMMARY_FILE"
echo ""
if [[ "$WRITE_TESTS" -eq 1 ]]; then
	echo "Formatted outputs were also written into tests/ (run './check_fmt.sh --restore' to reset)."
else
	echo "tests/ files remain synced with safe/ (pass '-w' to write formatted outputs into tests/)."
fi
echo "Full overview and unified diffs written to: $OUT_FILE"

if [[ "$ERROR_FILES" -gt 0 || "$AST_WARN_FILES" -gt 0 ]]; then
	exit 1
fi
