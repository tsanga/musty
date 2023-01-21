> musty is currently in very early development. the musty book is a work-in-progress.

# Introduction

**musty** is an asynchronous [object-document mapper](https://en.wikipedia.org/wiki/Object–relational_mapping) library for Rust. It turns your `struct`'s into queryable database models.

### Features

- Typed model filter/querying language via `filter!()` macro.
- Support for multiple different database backends.
- Automatically handles serializing, deserializing, id mapping, & more.
- Straight-forward integration, requiring little change to your data structs.
- Focus on extendability, underlying database driver is always available for advanced querying.
- Easily define indexes and dynamic `get_by` functions using the `#[musty()]` macro.

### Why use `musty`?

- Spend less time building an ODM and more time building your app.
- Leverage typed database-agnostic document queries using the `filter!()` macro.
- Ability to switch to a different database backend down the line with little to no code changes.

### Getting Started

**musty** is designed to integrate with little friction (i.e: not enforcing specific types to be used, etc), for how to get started using `musty`, check out the [quick start](./quickstart.md).
