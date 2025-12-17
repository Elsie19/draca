# TODO

Make evaluation print at the end of the last parens instead of newline, so:

```
\> (+ 1
    (*
        6
        7
    )
) => 43
\>
```

Instead of:

```
\> (+ 1
    (*
        6
        7
    )
)
43
\>
```

`define/in-namespace` should detect `define` and append the function name to the namespace.
