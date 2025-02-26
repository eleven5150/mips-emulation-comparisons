#! /usr/bin/env bash

texts_failed=0

for test_file in test_files/success/*; do
    echo -n "Checking \"$test_file\"... "

    if echo "$test_file" | grep -q 'broken'; then
        echo 'Skipping broken file'
        continue
    fi

    rush_out=$( yes 3 2>/dev/null | ./target/debug/rush   "$test_file" 2>&1 | sed -E "/Loaded: .+/d")
    spim_out=$( yes 3 2>/dev/null |               spim -f "$test_file" 2>&1 | sed -E "/Loaded: .+/d")

    if diff <(echo "$rush_out") <(echo "$spim_out") >/dev/null; then
        echo "PASSED"
    else
        echo "FAILED"
        echo "    rush_output: $rush_out"
        echo ""
        echo "    spim_output:  $spim_out"
        echo ""
        texts_failed=$((texts_failed + 1))
    fi
done

echo
echo
echo

RUSH_OUT=$(mktemp -d)
trap 'rm -rf "$RUSH_OUT"' EXIT
shopt -s globstar
shopt -s nullglob

for test_file in test_files/instructions/**/*.s; do
    EXPECTED_FILE="${test_file%.s}.out"
    OBSERVED_FILE="$RUSH_OUT/$(basename "$EXPECTED_FILE")"

    echo -n "Checking \"$test_file\"... "

    ./target/debug/rush "$test_file" > "$OBSERVED_FILE"

    if diff "$OBSERVED_FILE" "$EXPECTED_FILE" >/dev/null; then
        echo "PASSED"
    else
        echo "FAILED"

        echo "----- < Observed Output - Expected Output > -----"
        diff -s "$OBSERVED_FILE" "$EXPECTED_FILE" --label "'Observed Output'" --label "'Expected Output'"
        echo "-------------------------------------------------"

        texts_failed=$((texts_failed + 1))
    fi
done

exit $((texts_failed == 0 ? 0 : 1))
