#!/usr/bin/env elvish
# whtsky teaches me speaking elvish
# util function sets inspired by lodash

use str

# define a function for tests.
# it only implement function when this script is
# called with and only with `--test` flag
# i.e. `./whtsky.elv --test`
fn IMPLEMENT_TEST_FN [testFn~]{
    if (and (== (count $args) 1) (==s $args[0] '--test')) {
        put [@props]{ testFn $@props }
    } else {
        put [@_]{ nop }
    }
}

var ASSERT_EQ~=(IMPLEMENT_TEST_FN [@values]{
    result=$false

    if (<= (count $values) 1) {
        err = "ASSERT_EQ: need more than 1 values to compare"
        echo $err
        fail $err
    }

    try {
        result = (eq $@values)
    } except e {
        err = "ASSERT_EQ: "$e
        echo $err
        pprint $@values
        put $err
    }

    if (not (eq $result $true)) {
        err = "ASSERT_EQ: not equal"
        echo $err
        pprint $@values
        fail $err
    }
})

# test suite
var SUITE~=(IMPLEMENT_TEST_FN [suiteMessage @rest]{
    if (== (count $rest) 1) {
        echo $suiteMessage
        $rest[0]
    } else {
        # pending suite
        echo (styled "☐ "$suiteMessage cyan)
    }
})
# pending test suite
var XSUITE~=(IMPLEMENT_TEST_FN [suiteMessage @_]{
    echo $suiteMessage
})

# test case
var IT~=(IMPLEMENT_TEST_FN [testMessage @rest]{
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
var XIT~=(IMPLEMENT_TEST_FN [testMessage @_]{
    echo (styled "\t☐ "$testMessage cyan)
})

#         .__     __          __           
# __  _  _|  |___/  |_  _____|  | _____.__.
# \ \/ \/ /  |  \   __\/  ___/  |/ <   |  |
#  \     /|   Y  \  |  \___ \|    < \___  |
#   \/\_/ |___|  /__| /____  >__|_ \/ ____|
#              \/          \/     \/\/     

fn filter [filterFn~ list]{
    put [(each [item]{
        if (filterFn $item) {
            put $item
        }
    } $list)]
 }
SUITE 'filter' {
    IT '% 2' {
        ASSERT_EQ (filter [item]{
            put (== (% $item 2) 0)
        } [1 2 3 4 5]) [2 4]
    }

    IT '&value > 10' {
        ASSERT_EQ (filter [item]{
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

    IT 'empty result should be []' {
        ASSERT_EQ (filter [@_]{ put $false } [1 2 3]) []
    }
}

fn map [mapFn~ list]{
    put [(
        each [item]{
            put (mapFn $item)
        } $list
    )]
}
SUITE 'map' {
    IT '+ 1' {
        ASSERT_EQ (map [item]{
            put (+ $item 1)
        } [1 2 3]) [(num 2) (num 3) (num 4)]
    }
}

# flatten nested list
fn flatten [list]{
    put [(each [item]{
        if (==s (kind-of $item) 'list') {
            put $@item
        } else {
            put $item
        }
    } $list)]
}
SUITE 'flatten' {
    IT '[1 [2 3] 4]' {
        ASSERT_EQ (flatten [1 [2 3] 4]) [1 2 3 4]
    }
    IT '[[[2]]]' {
        ASSERT_EQ (flatten [[[2]]]) [[2]]
    }
}

# flatten nested list, recursively
fn flattenDeep [list]{
    fn _flatten [_list]{
        put (each [item]{
            if (==s (kind-of $item) 'list') {
                put (_flatten $item)
            } else {
                put $item
            }
        } $_list)
    }
    put [(_flatten $list)]
}
SUITE 'flattenDeep' {
    IT '[1 [2 3] 4]' {
        ASSERT_EQ (flattenDeep [1 [2 3] 4]) [1 2 3 4]
    }
    IT '[[[2]]]' {
        ASSERT_EQ (flattenDeep [[[2]]]) [2]
    }
    IT '[[[2]] [3] [4 [[5]]]]' {
        ASSERT_EQ (flattenDeep [[[2]] [3] [4 [[5]]]]) [2 3 4 5]
    }
    IT "['a' ['b' 'c.d'] 'e.f' [['g']]]" {
        ASSERT_EQ (flattenDeep ['a' ['b' 'c.d'] 'e.f' [['g']]]) [a b c.d e.f g]
    }
}

# expand a dot notation or list of dot notation
# into plain list of path
fn expandPath [@paths]{
    put [(each [pathNotation]{
        str:split '.' $pathNotation
    } (flattenDeep $paths))]
}
SUITE 'expandPath' {
    IT 'a.b' {
        ASSERT_EQ (expandPath 'a.b') [a b]
    }
    IT 'a.b c d.e' {
        ASSERT_EQ (expandPath 'a.b' 'c' 'd.e') [a b c d e]
    }
    IT 'a b' {
        ASSERT_EQ (expandPath a b) [a b]
    }
    IT 'a [b c.d] e.f [[g]]' {
        ASSERT_EQ (expandPath 'a' ['b' 'c.d'] 'e.f' [['g']]) [a b c d e f g]
    }
}

fn findIndex [list targetOrTestFn]{
    testFn~ = $nop~

    if (==s (kind-of $targetOrTestFn) 'fn') {
        testFn~ = $targetOrTestFn
    } else {
        testFn~ = [value]{ eq $value $targetOrTestFn }
    }

    result = (num -1)
    for index [(range (count $list))] {
        if (testFn $list[$index]) {
            result = $index
            break
        }
    }
    put $result
}
SUITE 'findIndex' {
    IT '[1 2 3] 1 -> 0' {
        ASSERT_EQ (findIndex [1 2 3] 1) (num 0)
    }
    IT '[1 2 3] 9 -> -1' {
        ASSERT_EQ (findIndex [1 2 3] 9) (num -1)
    }
    IT '[] 9 -> -1' {
        ASSERT_EQ (findIndex [] 9) (num -1)
    }
    IT 'custom testFn' {
        ASSERT_EQ (findIndex [1 3 6 8 9] [v]{ == 0 (% $v 2) } ) (num 2)
    }
    IT 'custom testFn, index not found' {
        ASSERT_EQ (findIndex [1 3 6 8 9] [v]{ == 0 (% $v 10) } ) (num -1)
    }
}

fn find [list targetOrTestFn]{
    testFn~ = $nop~

    if (==s (kind-of $targetOrTestFn) 'fn') {
        testFn~ = $targetOrTestFn
    } else {
        testFn~ = [value]{ eq $value $targetOrTestFn }
    }

    for item $list {
        if (testFn $item) {
            put $item
            break
        }
    }
}
SUITE 'find' {
    IT '[1 2 3] 3 -> 3' {
        ASSERT_EQ (find [1 2 3] 3) 3
    }
    IT '[1 2 3] 0 -> nothing' {
        ASSERT_EQ [(find [1 2 3] 0)] []
    }
    IT 'customTestFn' {
        ASSERT_EQ (find [1 3 5 7 9] [v]{ == (% $v 5) 0 }) 5
    }
    IT 'customTestFn, no result' {
        ASSERT_EQ [(find [1 3 5 7 9] [v]{ == (% $v 9999) 0 })] []
    }
}

fn hasKey [map searhKey]{
    != (findIndex [(keys $map)] $searhKey) -1
}
SUITE 'hasKey' {
    IT '[&a &b] has b' {
        ASSERT_EQ (hasKey [&a=1 &b=3] b) $true
    }
    IT '[&a &b] does not have z' {
        ASSERT_EQ (hasKey [&a=1 &b=3] z) $false
    }
}

fn get [mapLike path]{
    fn _get [_map _pathList]{
        key @restPathList = $@_pathList
        if (hasKey $_map $key) {
            if (== (count $restPathList) 0) {
                # last key of pathList is found
                put $_map[$key]
            } elif (==s (kind-of $_map[$key]) 'map') {
                put (_get $_map[$key] $restPathList)
            }
        }
    }

    pathList = (expandPath $path)
    put (_get $mapLike $pathList)
}
SUITE 'get' {
    IT 'a' {
        ASSERT_EQ (get [&a=3 &b=8] 'a') 3
    }
    IT 'a.b.c' {
        ASSERT_EQ (get [&a=[&b=[&c=23] &b2=9] &y=8] 'a.b.c') 23
    }
    IT 'illegal path should return nothing' {
        ASSERT_EQ [(get [&a=[&b=[&c=23] &b2=9] &y=8] 'd.e.f')] []
    }
}
