#!/usr/bin/env elvish
# whtsky teaches me speaking elvish
# util function sets inspired by lodash

# define a function for tests.
# it only implement function when this script is
# called with and only with `--test` flag
# i.e. `./whtsky.elv --test`
fn IMPLEMENT_TEST_FUN [fun]{
    if (and (== (count $args) 1) (==s $args[0] '--test')) {
        put [@props]{ $fun $@props }
    } else {
        put [@_]{ nop }
    }
}

ASSERT_EQ=(IMPLEMENT_TEST_FUN [@values]{
    result=$false

    if (<= (count $values) 1) {
        err="ASSERT_EQ: need more than 1 values to compare"
        echo $err
        fail $err
    }

    try {
        result=(eq $@values)
    } except e {
        err="ASSERT_EQ: "$e
        echo $err
        pprint $@values
        put $err
    }

    if (not (eq $result $true)) {
        err="ASSERT_EQ: not equal"
        echo $err
        pprint $@values
        fail $err
    }
})

# test suite
SUITE=(IMPLEMENT_TEST_FUN [suiteMessage @rest]{
    if (== (count $rest) 1) {
        echo $suiteMessage
        $rest[0]
    } else {
        # pending suite
        echo (styled "☐ "$suiteMessage cyan)
    }
})
# pending test suite
XSUITE=(IMPLEMENT_TEST_FUN [suiteMessage @_]{
    echo $suiteMessage
})

# test case
IT=(IMPLEMENT_TEST_FUN [testMessage @rest]{
    if (== (count $rest) 1) {
        if ?($rest[0]) {
            echo (styled "\t✔ "$testMessage green)
        } else {
            echo (styled "\t✘ "$testMessage red)
        }
    } else {
        # pending test case
        echo (styled "\t☐ "$testMessage cyan)
    }
})
# pending test case
XIT=(IMPLEMENT_TEST_FUN [testMessage @_]{
    echo (styled "\t☐ "$testMessage cyan)
})

fn get []{}
$SUITE 'get'

fn set []{ nop }
$SUITE 'set'

fn findIndex []{}
$SUITE 'findIndex'

fn find []{}
$SUITE 'find'

fn filter [filterFun list]{
    put [(each [item]{
        if ($filterFun $item) {
            put $item
        }
    } $list)]
 }
$SUITE 'filter' {
    $IT '% 2' {
        $ASSERT_EQ (filter [item]{
            put (== (% $item 2) 0)
        } [1 2 3 4 5]) [2 4]
    }

    $IT '&value > 10' {
        $ASSERT_EQ (filter [item]{
            put (> $item[value] 10)
        } [
            [&value=0]
            [&value=11]
            [&value=3]
            [&value=100]
        ]) [
            [&value=11]
            [&value=100]
        ]
    }
}

fn map [mapFun list]{
    put [(
        each [item]{
            put ($mapFun $item)
        } $list
    )]
}
$SUITE 'map' {
    $IT '+ 1' {
        $ASSERT_EQ (map [item]{
            put (+ $item 1)
        } [1 2 3]) [2 3 4]
    }
}

# flatten nested list
fn flatten [list]{
    fn _flatten [_list]{
        put (each [item]{
            if (eq (kind-of $item) 'list') {
                each [i]{
                    put (_flatten $i)
                } $item
            } else {
                put $item
            }
        } $_list)
    }
    put [(_flatten $list)]
}
$SUITE 'flatten' {
    $IT '[1 [2 3] 4]' {
        $ASSERT_EQ (flatten [1 [2 3] 4]) [1 2 3 4]
    }
    $IT '[[[2]]]' {
        $ASSERT_EQ (flatten [[[2]]]) [2]
    }
    $IT '[[[2]] [3] [4 [[5]]]]' {
        $ASSERT_EQ (flatten [[[2]] [3] [4 [[5]]]]) [2 3 4 5]
    }
}

# expand a dot notation or list of dot notation
# into plain list of path
fn expandPath [@paths]{
    put [(each [pathNotation]{

    } $paths)]
}
$SUITE 'expandPath' {
    $XIT 'a.b' {
        $ASSERT_EQ (expandPath 'a.b') [a b]
    }
}

# check if a (nested)map have a specific key
# path is presented as dot notation or
# list of dot notation
fn hasKey [mapLike path]{
}
$SUITE 'hasKey'