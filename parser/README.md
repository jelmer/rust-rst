`uo_rst_parser`
==============

This crate is a fork of the `rst_parser` crate, with fixes necessary
for ``upstream-ontologist``.

Part of the [`rst`][rst] crate family.
Offers the functions `parse` and `parse_only`,
which try to create a `document_tree::Document`.
`parse` simplifies this document and resolves references before returning it.

[rst]: https://github.com/flying-sheep/rust-rst/#readme
