# Hithlain

Hithlain is a toy hardware descriptor language (HDL) created in Rust.
The syntax for it is currently not complete, but the syntax is usable.
An example program can be seen below.

```
circuit add: a b c-in -> o c-out {
    o = a xor b xor c-in;
    c-out = (a and b) or ((a xor b) and c-in);
}

test main {
    o, c-out = add(a, b, 0);

    at 0ns:
        a = 1;
        b = 1;

        assert o == 0;
        assert c-out == 1;

    after 5ns:
        a = 0;
        b = 0;

        assert o == 0;
        assert c-out == 0;

    after 5ns:
        a = 1;
        b = 0;

        assert o == 1;
        assert c-out == 0;
}
```

## Goals 

Hithlain can compile programs and simulate them (generating a VCD file). I'm intending
to also create a transpiler to both Verilog and VHDL.
