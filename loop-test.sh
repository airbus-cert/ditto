TARGET="0.00001 0.0001 0.001 0.01 0.1 0.2 0.3 0.4 0.5 0.6 0.7 0.8 0.9 10.1 100.1 1000.1"

cargo build --release --package ditto-cli --bin ditto-cli || exit 1

count=0
while true; do
  count=$((count + 1))
  ./target/release/ditto-cli -p test.js -i Metagross -d info -t > out.js \
    && output=$(node out.js) \
    && echo "$output" \
    && grep -qvxF "$TARGET" <<< "$output" \
    && break
done

count=$((count - 1))

echo "Did $count successful tests"