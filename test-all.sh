assert() {
    expected="$1"
    input="$2"

    sh ./ironcc.sh "$2"
    ./a.out
    actual="$?"
    if [ "$actual" = "$expected" ]; then
        echo "Passed $2"
        #echo -n "."
        #echo "Got $actual as expected"
    else
        echo "Failed at $2"
        echo "$expected is expected, but got $actual"
        exit 1
    fi
}

assert 4 ./test/calc.c
assert 55 ./test/for.c
assert 30 ./test/if.c
echo OK