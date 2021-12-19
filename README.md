# Hithlain

[//]: # ([![Docs.rs]&#40;https://img.shields.io/badge/docs.rs-perpendicular-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K&#41;]&#40;https://docs.rs/perpendicular&#41;)
[//]: # ([![Crates.io]&#40;https://img.shields.io/crates/v/perpendicular?logo=rust&style=for-the-badge&#41;]&#40;https://crates.io/crates/perpendicular&#41;)
[![Github Workflows](https://img.shields.io/github/workflow/status/jonay2000/hithlain/label?logo=github&style=for-the-badge)](https://github.com/jonay2000/hithlain/actions/workflows/ci.yml)

Hithlain is a toy hardware descriptor language (HDL) created in Rust.
The syntax for it is currently not complete, but the syntax is usable.
An example program can be seen below.

```
circuit add: a b c_in -> o c_out {
    o = a xor b xor c_in;
    c_out = (a and b) or ((a xor b) and c_in);
}

test main {
    o, c_out = add(a, b, 0);

    at 0ns:
        a = 1;
        b = 1;

        assert o == 0;
        assert c_out == 1;

    after 5ns:
        a = 0;
        b = 0;

        assert o == 0;
        assert c_out == 0;

    after 5ns:
        a = 1;
        b = 0;

        assert o == 1;
        assert c_out == 0;
}
```

## Goals 

Hithlain can compile programs and simulate them (generating a VCD file). I'm intending
to also create a transpiler to both Verilog and VHDL.

## naming

Hithlain is the sindarin word for the material elvish ropes are made out of, which is given to Sam by Galadriel.
