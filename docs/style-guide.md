Rust Best Practice for ICU4X
============================

# Introduction

This document outlines the Rust style guide and best practice for code in ICU4X.

As Rust is a new programming language it is expected that the language guidelines will evolve over time, so care should be taken to keep this document up to date, and any new changes after the initially agreed upon version should be reflected in the change history at the bottom of this document.

Before reading this, it is recommended that you read most of [The Rust Programming Language](https://doc.rust-lang.org/book/title-page.html) and [Rust by Example](https://doc.rust-lang.org/rust-by-example/#rust-by-example). There's enough new concepts and syntax in Rust that it's not a language that's easy to pick up "as you go".

Items in this document are categorized as **required** or **suggested**, which essentially maps to the __MUST__ and __SHOULD__ language of Internet RFCs. Any use of the more lenient MAY is deliberately avoided here, since there's no need to encourage too much personal preference in this code base.

Many of the practices here are collected from existing sources (which should be cited) and where applicable there may be some additional justification for including that practice in this document and why it is categorized as it is. In general where a practice is sensible, expected to be achievable, and has no obvious downside, it will be marked as required.

Note however that none of this document is meant to trump common sense, if if you're in a situation where it would be better to violate a **required** practice, then it just means we should have a discussion about it. There will almost certainly be many cases where we need to update this doc and even go back to existing code to update it in the light of new information. However exceptions should always be commented clearly for the next maintainer.

If you're new to Rust, see the [Appendix](#appendix) for some, hopefully useful, links.

# Naming Conventions

### No Special Naming :: required

Follow the naming advice in [Naming - Rust API Guidelines](https://rust-lang.github.io/api-guidelines/naming.html) at all times, and use [Rust RFC 0430](https://github.com/rust-lang/rfcs/blob/master/text/0430-finalizing-naming-conventions.md) for any ambiguous issues (e.g. Crate naming).

* Infer Crate names from package names where feasible:
  * A package is the directory in which the Cargo.toml exists.
* Prefer single (lowercase) words for package names where feasible
* Use __lowercase-with-hyphen__ when multiple parts exist in a package name.
  * This seems to be the preferred Rust community approach.
* Hyphens in package names are converted to __underscore__ in Crate names.
  * To avoid ambiguity, do not use underscore in package names.

This should result in Crate names always being unambiguously mappable to/from package names (e.g. the Crate "foo_bar" relates to the package "foo-bar").

See also [this discussion on API guidelines](https://github.com/rust-lang/api-guidelines/issues/29#issuecomment-342745898).

## Module and Code Layout

### Don't have "mixed" Crates :: suggested

The current thinking is that you should either:

* Use a single `lib.rs` file for a Crate with everything in it
* Use multiple Rust sources and then only use `lib.rs` to export modules

Which of these options you chose is dependent on the size/complexity of the module(s).

There might be cases where an intermediate "mixed" Crate is worthwhile, but it should be clearly documented as to why it is necessary.

### Avoid over-exposing internal APIs :: suggested

When sharing an API between modules in a Crate, use `pub(<scope>)` to express visibility, and select the smallest scope which suits the intent of the API (i.e. prefer `pub(crate::specific_mod)` to `pub(crate)` where appropriate).

## Code Formatting and Linting

### Use "cargo fmt" prior to any code reviews :: required

Hopefully self explanatory. The less time we spend worrying about formatting and the fewer diffs during code reviews, the better.

### Run "cargo clippy" and accept its suggestions :: suggested

There are bound to be some cases where the linter makes suggestion we don't want to follow, but these should be rare. Any such false-positives should be [suppressed](https://github.com/rust-lang/rust-clippy#allowingdenying-lints) and commented for future maintainers.

**Open Question**: Should we prohibit un-suppressed warnings as a github check?

**Open Question**: Is this including all the pedantic warnings?

# Structs and Traits

For ICU4X we should be looking to keep code consistent and very unsurprising, especially for layout and behavior of data structures.

## Private vs Public

Rust offers a compelling model for encapsulating implementation details based on a hierarchical view of privacy; see [Visibility and Privacy - The Rust Reference](https://doc.rust-lang.org/reference/visibility-and-privacy.html). This should allow ICU4X to encapsulate all of its implementation details neatly from applications using the library and avoid too many cases of [Hyrum's Law](https://www.hyrumslaw.com/).

### Minimize user-visible structs :: suggested

While this sounds a bit obvious, it's important to stress that with Rust's privacy model, there is never any need to have an internal type be user-visible, and once a type is user-visible outside a Crate, it has a high cost associated with changing it (i.e. a semantic version change to the library). If there are parts of the API which must be released, but for which we cannot provide stability guarantees, they must be marked as such.

There is a known pattern of using [`doc(hidden)`](https://doc.rust-lang.org/rustdoc/the-doc-attribute.html#dochidden) and having module names prefixed with multiple underscores to indicate they must not be relied upon outside the project, but ideally APIs would be designed to minimize the need for this and the expected external usage would be clearly documented for future maintainers.

There are three areas which may warrant a technically public API that we don't consider stable:
* Where another crate may need access to non-public API
* Where we want to expose some non-public API for testing or benchmarking
* Where we want to expose some non-public API for helper macros

### No public fields :: suggested

Unless there is a compelling use case, it seems very sensible to never use public fields in structs. This allows for clean encapsulation with essentially no downside for ICU4X. This post sums things up nicely, which I am summarizing below:

For a **pub** field `bar` in a **pub** struct `Foo`:

* Construction: users can create a `Foo { bar: 42 }`.
  * This also means it’s a breaking change for you to add or remove any fields (and using a Default trait won't help you here since you still have to rely on every caller using it to avoid adding a new field being a breaking change).
* Mutation: if the user has some `mut var: Foo` or `&mut Foo`, they can change the value without notice.
  * While this won't cause any race conditions in Rust, it does prevent a whole class of useful optimizations for types around internal consistency (e.g. you cannot cache otherwise expensive hash-code values).
* Partial borrow: if a user binds `let var = &foo.bar;` they can still use other fields without any complaint from the borrow checker.

You can always supply public inlinable getters to access fields, and non-public fields are still directly accessible by the current module and any sub-modules.

One suggested situation in which public fields would be acceptable is for user-facing "bag of options" structs. These would have no inbuilt semantics and no consistency guarantees, so the visual "cleanliness" of bare fields might outweigh the issues above. See [this issue](https://github.com/unicode-org/rust-discuss/issues/15) for more.

## Derived Traits

### Debug Trait on Public Types :: required

This might be contentious, but I firmly believe we should allow all public types used by ICU4X to support (at least) the [Debug](https://doc.rust-lang.org/std/fmt/trait.Debug.html) trait. This adds a bit of additional code, and requires that all fields in the struct also have this trait (unless you add a non-derived custom implementation).

### Implement Clone and Copy only as needed :: suggested

Which types implement these traits should be carefully considered and documented.

* **Clone** is a general purpose duplication mechanism.
  * To derive this trait you must have all fields being Clone-able.
  * Types can also implement this trait manually and provide custom business logic to handle ownership etc.
  * While deriving or implementing Clone is normal for most data types, it may not be suitable for other types, especially those backed by system resources (e.g. data providers).
* **Copy** is for types which can always be safely duplicated (e.g by bitwise copying).
  * A type which implements **Copy always implicitly implements Clone**.
  * You cannot override and customize how Copy is implemented.
  * For example, note that `String` does not implement Copy.

There are pros and cons for using these derived traits, but once a (user) public struct uses a derived trait, it is **part of the published API and cannot be removed**.

Simple value types will benefit the most from implementing Clone and Copy traits, since it allows users to avoid having to deal with lifetimes and borrowing. However this can prohibit (or at least complicate) the act of adding a new field to the struct (e.g. adding a `String` to a Copy-able struct).

Adding a trait later is very unlikely to be a breaking change (technically it is, but only in cases where someone has extended the type with module specific traits).

My guess is that we should prefer implementing Clone on user visible types (for convenience) but avoid promising Copy in most situations. I believe that the Rust compiler is sensible enough to implement Clone exactly as if it were Copy if all fields are Copy-able, but by not advertising this we get the **freedom to add non-copyable fields later without breaking anyone**.

This could be slightly annoying for users who want to put ICU4X types into otherwise Copy-able structs, so we might face pressure to make some common data types Copy-able.

Note: A type which has **neither Copy or Clone** can still be passed by value or returned from a function, but only when that operation can be determined to always be a "move" of the data (i.e. where only one instance is needed). Copy and Clone are needed when duplicate instances need to be created.

### Implement Eq / PartialEq, Ord / PartialOrd only as needed :: suggested

Which types implement these traits should be carefully considered and documented.

* [**Eq**](https://doc.rust-lang.org/std/cmp/trait.Eq.html) and [**PartialEq**](https://doc.rust-lang.org/std/cmp/trait.PartialEq.html) are traits designed for comparing equality of types.
  * These traits define the behaviour of the `==`, `!=` operators.
* **PartialEq** is useful when a looser equality is required (e.g. comparing `NaN`s for floating point).
* [**Ord**](https://doc.rust-lang.org/std/cmp/trait.Ord.html) and [**PartialOrd**](https://doc.rust-lang.org/std/cmp/trait.PartialOrd.html) are traits designed for ordering types.
  * These traits define the behaviour of the comparison operators (`<`, `>`, `<=`, `>=`).
  * Their behavior must be consistent with **Eq** and **PartialEq**.

Implementing comparison traits can be problematic and should only be done for types where it is semantically meaningful. Note that Rust is very opinionated on the expected behaviour of these traits, so (for example) you must not implement a non-symmetric equality operator or a non-transitive partial ordering.

One drawback to deriving the Eq trait "too early" would be if you later add some other non-Eq type to the struct. For example, **floating point types implement PartialEq but not Eq**.

### No "Clever" Comparisons :: suggested

The above comparison traits can be implemented for "interesting" type combination, not just on a single type. This would allow for semantically complex comparisons between user types (e.g. a mutual ordering between region codes and language codes). However this sort of API design, with implicit relations, is often subtle and easy to misuse. As the documentation for PartialEq states:

> Implementations of PartialEq, PartialOrd, and Ord must agree with each other. **It's easy to accidentally make them disagree by deriving some of the traits and manually implementing others**.

Note that we might choose to have non-derived orderings for some types (e.g. script sub-tags), but these should always be carefully documented.

## Sized Types

**Open Question**: Decide what to recommend here (if anything).

Types in Rust can be **sized** or **unsized**. A sized type knows at compile time how much space is needed to hold it, and can thus have stack space allocated or be copied into a Vec or array. This improves memory locality and reduces the need for heap allocations.

Perhaps more importantly, **an unsized type can only be passed by reference to other functions** and cannot be Copy-able (though they can be Clone-able). In particular any trait is, by default, unsized which makes storing traits in other structs a bit interesting (there is a [ToOwned](https://doc.rust-lang.org/std/borrow/trait.ToOwned.html) trait to help with this though).

This adds weight to the idea that we should avoid traits and any other unsized types when specifying user visible types. Obviously we will accept things like `str` as parameters (these are unsized), but we aren't adding them directly to structures by reference.

For example, in general we should avoid returning an abstract trait to the user (intermediate traits like Iterator might be fine though since a user is expected to consume those fairly immediately).

# Idiomatic Code

A lot of this section just boils down to "[Read the Book](https://doc.rust-lang.org/book/)", but I'm highlighting a few things I personally found useful while learning.

## Pass by Reference vs Pass by Value

There is something a bit subtle about how Rust handles "pass by value" which means you should not just apply standard C++ best-practice. In particular, in C++ you would might expect a method like:

```
ReturnType ClassName::FunctionName(ParamType param) { … }
```

to copy the parameter and return types during the course of calling the method (e.g. as a memcpy to the stack). This leads C++ developers to favour passing objects by reference and passing pointers to hold return types, instead of taking the hit of copying the value.

Rust treats "pass by value" a little differently, and it can be thought more of an indication that "ownership is being transferred" rather than the API wants "a copy of the data". Thus the Rust equivalent code:

```
fn function_name(param: ParamType) -> ReturnType { … }
```

may still have parameters "passed by reference" when it can determine that the reference is no longer subsequently used by the calling code (this is a "move" in Rust parlance).

Furthermore, it will often [ellide the copy](https://en.wikipedia.org/wiki/Copy_elision) of the return value if it determines the returned object would otherwise go out of scope. It will allocate the space for the return value on the caller's stack or use the memory in a destination struct, to directly write the "returned" value in its final destination with no copying whatsoever. This is called Return Value Optimization (RVO) and while it is now available in C++ as well, it's a relatively new feature there.

It is still often better (for reasons of borrowing and ownership) to pass structs by non-mutable reference, but returning **newly created** results by value (even potentially large structures) is not expected to cause performance issues.

### Return new instance by value :: suggested

Use "return by value" for any functions which produce new data and let Rust handle optimizing away any copying, unless there's really measurable performance issue.

### Pass struct parameters by reference where possible :: suggested

When passing struct types to functions, it's always semantically safe to pass a reference (since there can be no race conditions around its use and it cannot be modified unexpectedly while being processed). The called function should be responsible for taking a copy if it needs to (e.g. by cloning or via the ToOwned trait).

### Pass Option<T> by value where possible :: suggested

[Option](https://doc.rust-lang.org/std/option/enum.Option.html) is a bit special, and Rust goes to great lengths to avoid needing to allocate an additional instance for this type. In particular, `Option<char>` is a zero-allocation representation and should be passed by value. This applies to some types where the extra [None](https://doc.rust-lang.org/std/option/enum.Option.html#variant.None) state can be encoded as an otherwise "invalid" bit pattern.

In the case of `Option<char>`, `None` is [encoded as `0x110000`](https://rust.godbolt.org/z/-ZFwKB) or other invalid bit patterns. This means types like `Vec<Option<char>>` or `[Option<char>;N]` should be preferred over `Vec<&Option<char>>` or `[&Option<char>;N]` which will often remove the need for additional lifetimes.

There are times where this may not be possible (e.g. if using non-`Copy` types you may need `&Option<T>` or `Option<&T>`), and there's a trade off between designing `Copy` types, exppected to be smaller value types, and other more complex types.

### Pass Box<T> by value where possible :: suggested

It's unlikely we will have many `Box<T>`s in public APIs. We should return boxes by-value, and accept things as `&T` as much as possible, instead of `&Box<T>`. Internally we might use `Box<T>` for efficiency, but it should be avoided in any public APIs.

### Pass and return fundamental type by value where possible :: required

Fundamental types are essentially free to pass around (at least never more expensive than passing a pointer) so unless you are using a mutable reference to the location of a fundamental type (e.g. in-place modification in a `Vec`) there's no reason not to just pass the value itself directly. Passing them by value, will often simplify cases where they are manipulated or compared with constants.

## Option

### Use Option exclusively to represent "missing" data :: required

[Option](https://doc.rust-lang.org/std/option/enum.Option.html) is Rust's only recommended way to represent the equivalent of a "null" value (i.e. missing data). Option has a host of useful traits and methods which make it easy to manipulate, propagate and transform.

```
// Get the value or a default.
let x = opt_value.unwrap_or(other_value);
```

```
// Get the value or return an Err
// (this can only be called in a function that returns a Result)
let y = opt_value.ok_or("Error message")?;
```

```
// Transform with lazy default value (note how "into()" can elide the
// type since it's already known to be OtherType).
let z = opt_value.map_or_else(OtherType::get_default, |v| v.into());
```

## Iteration

### Strongly prefer functional iteration :: suggested

Prefer using iterators rather than directly indexing data to avoid the need for any explicit bounds checks. The Rust compiler will make this at least as performant as a manual loop.

```
let tags: Vec<Tag> = ["en", "fr", "de"].iter().map(Tag::from).collect();
```

```
let mut tags: Vec<String> = Vec::new();
let lang = ["en", "fr", "de"];
for n in 0 .. lang.len() {
    tags.push(Tag::from(lang[n]));
}
```

## Enums

### Strongly prefer enums to define states :: suggested

Enums in Rust are cheap/free, and incredibly useful. They can be used (as in C++/Java) for providing named, bounded "choices", including avoiding bare boolean parameters, but they can also provide the basis for type-safe patterns such as [elegant finite state machines](https://bluejekyll.github.io/blog/fsm/rust/2015/08/13/rust-and-the-most-elegant-fsm.html) using generified enums with data.

It's probably worth noting here that the [Result](https://doc.rust-lang.org/std/result/) type itself is just a normal enum in Rust with two values (`Ok` and `Err`).

## Matching

### Prefer match statements for exhaustive conditional code :: suggested

Matching is Rust's idiomatic way to handle any non-trivial exhaustive conditional code, and works elegantly with enums, but handles any data for which a predicate can be formed (which is basically everything).

Even a simple if/else block can be expressed more idiomatically using a match statement and will produce essentially the same compiled code after optimization. This is especially true if you want all cases to be covered, since this is enforced by a match statement.

From [Rust By Example](https://doc.rust-lang.org/rust-by-example/flow_control/match.html):

```
match number {
    // Match a single value
    1 => println!("One!"),
    // Match several values
    2 | 3 | 5 | 7 | 11 => println!("This is a prime"),
    // Match an inclusive range
    13..=19 => println!("A teen"),
    // Handle the rest of cases
    _ => println!("Ain't special"),
}
```

Another nice property of match is that it returns a value, which tied to the fact that it must cover all cases means you get a readable idiom for ad-hoc matching and mapping:

```
let x = match number {
    0     => "Zero",
    1..=9 => "Some",
    _     => "Many",
}
```

Obviously where an if-statement is simply there to do optional work, and not cover every case, it may well be more suitable to just use that.

# Error Handling

See also the [Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html) chapter in the Rust Book.

The ICU4X library should be designed so as to never fail at runtime. Now obviously that's something you'd expect all library writers to say, but in Rust you can control the places where code can fail explicitly, so it's much easier to write self-documenting code where the assumptions around failure are obvious.

Most core rust APIs (traits) have two ways to access data, a version that can "panic" , and one which returns a [Result](https://doc.rust-lang.org/std/result/index.html) or an [Option](https://doc.rust-lang.org/std/option/enum.Option.html). Rust is a language which likes to avoid "unnecessary" overhead, and in some cases it is perfectly correct to use an API which can "panic" because you have already checked the arguments carefully (and performance will be better).

Note that in cases where the Rust compiler can statically determine that a check is sufficient to avoid panic, it will remove the internal check and panic related code, leaving just a provably safe data access.

### Where Result is needed, use IcuResult<T> :: required

While it's still an open question in the Rust community as to what the best way to handle error is, the current ICU4X concensus is that we should start simple and expect to revisit this topic again at some point. The simplest reasonable starting point would be to have a `IcuResult<T>`, which is type as `Result<T, IcuError>`, where:

```
// Nesting semantically interesting error information inside the generic error type.
enum IcuError {
    ParserError(parser::ParserError),
    RuntimeError(...)
}
```

A couple of crates by `@dtolnay` that are considered "new wave of good error APIs" and are complementary to each other:

* https://github.com/dtolnay/thiserror
* https://github.com/dtolnay/anyhow

Other links on error handling:

* https://blog.yoshuawuyts.com/error-handling-survey/
* http://sled.rs/errors
* https://boats.gitlab.io/blog/post/failure-to-fehler/
* https://boats.gitlab.io/blog/post/why-ok-wrapping/
* https://vorner.github.io/2020/04/09/wrapping-mental-models.html
* https://yaah.dev/try-blocks

## Panicking APIs

The most common example of an API which can panic is access to slices and elements of a slice. This includes accessing `array`, `str`, `Vec`, `HashMap` etc. via the `[`,`]` (square bracket) operator, implemented by the [Index](https://doc.rust-lang.org/std/ops/trait.Index.html) trait.

Thus statements like:

```
let x = self.data[n];
```

are always prone to "panic" if the index/key is incorrect. (⚠️ and this includes accessing maps ⚠️).

This is because these APIs return a value (or value reference) which cannot be "null", so there is no way for them to signal failure via the return type.

## Non-Panicking APIs

The alternative to using direct data accessors which can panic is to use a method which can return **Option** or **Result**. In the case of collections and strings, where a simple data item is being requested, this is most often provided by functions such as "get" (or "get_mut" for mutable references) which return `Option`.

If data access is expected to fail occasionally (e.g. looking up properties in a map) then the resulting [Option can be unwrapped](https://doc.rust-lang.org/std/option/enum.Option.html#method.unwrap_or) or propagated accordingly.

If missing data signals a "hard" error from which the function cannot recover (e.g. user supplies incorrect input) then any returned `Option` should be [propagated into a `Result` immediately](https://doc.rust-lang.org/std/option/enum.Option.html#method.ok_or), with an appropriate error value.

## Best Practice

### Don't Panic :: required

Call non-panicking data access APIs whenever data is not guaranteed to be safe.

This should not include the contract of code in a different Crate. I.e. if a function in a different Crate promises to return a valid map key, but it's not a compile time checked type (like an enum), then the calling code must allow for it to fail.

### Don't Handle Errors :: required

Functions which can error for any reason must return a `Result`, and APIs should be designed such that you should never need to recover from an [Err](https://doc.rust-lang.org/std/result/enum.Result.html#variant.Err) internally (which should always be immediately propagated up to the user by using the [`?` operator](https://doc.rust-lang.org/edition-guide/rust-2018/error-handling-and-panics/the-question-mark-operator-for-easier-error-handling.html)). I.e. Never write library code which recovers from its own "errors", since if it can be recovered from, then it wasn't an "error".

This approach should mean that error handling and the design of functions which can propagate errors is consistent everywhere. For non-error cases, where different types of result are possible, use a normal enum.

The only time you might need to handle an **Err** is if you call APIs outside the library which return Result rather than Option (e.g. allowing a retry for data loading).

Finally, and fairly obviously, **never turn an error into a panic by unconditionally unwrapping the result in the library**.

### Comment Use of Panicking Calls :: required

Use panicking methods only when the input has been explicitly checked to be correct.

```
// Attribute keys are checked for validity during data loading by ...
let x = self.attribute_map[char_attribute.key];
```

If this check does not occur immediately before the data access (i.e. shortly before in the same function), comment clearly where it does occur.

For example, if indices obtained from ICU data are to be trusted for indexed access, the data itself must have been validated at some earlier time (e.g. via a checking pass during data loading or use of a trusted hash).

However, you should **never add a check purely in order to call a method which could otherwise panic**; in that situation you should always prefer to call the non-panicking equivalent and handle the Option or Result idiomatically.

### Use Result over Option for errors :: suggested

When creating functions which can fail to return a value:
* Use **IcuResult** for all errors, or any cases where a user facing message is needed.
* Use **Option** for data accessors where "no data available" is a valid response (i.e. it's not an error per se).
  * Especially in cases where we expect the caller to have a reasonable response to getting [None](https://doc.rust-lang.org/std/option/enum.Option.html#variant.None).
* Use a different enum for non-error cases with multiple return types (which can't use `Option`).

Examples:
* Does file with this path exists? - Option.
* Is there an element with this key in the list? - Option
* Try to open a file - Result
* Try to parse a string into a valid Language Identifier - Result

# Advanced Features

## Operator Overloading

### No clever operators :: required

Other than the comparison operators defined by [**Eq**](https://doc.rust-lang.org/std/cmp/trait.Eq.html), [**Ord**](https://doc.rust-lang.org/std/cmp/trait.Ord.html) etc. there is no obvious benefit to overloading other operators (e.g. overloading `*` to do something clever via the [Mul](https://doc.rust-lang.org/std/ops/trait.Mul.html) trait). This could be relaxed if standard Rust idioms for using particular operator overloads exists, and is well understood in the Rust community.

See also [Operators and Symbols](https://doc.rust-lang.org/book/appendix-02-operators.html).

## Binding Traits to Inbuilt Types

Rust has neat ways to provide users with simple type conversion. Imagine if you have a function taking (reference to) a semantic type:

```
fn use_locale_id(id: &LocaleId) { … }
```

But the caller could just write:

```
use_locale_id(&"en_GB".into());
```

Rust's lets you bind traits to existing system type (e.g. `str`) for use within a module. And importantly, it lets you expose a series to trait bindings that other people can opt into if they want.
By implementing the generic [`TryFrom<&str>`](https://doc.rust-lang.org/std/convert/trait.TryFrom.html) trait on `LocaleId` to convert from a string to a locale ID, we also get the [`TryInto<Foo>`](https://doc.rust-lang.org/std/convert/trait.TryInto.html) trait implied on `&str` for free.

```
impl TryFrom<&str> for LocaleId {
  fn try_from(s: &str) -> IcuResult<LocaleId> {
    ...
  }
}
```

Note that there's also the [FromStr](https://doc.rust-lang.org/beta/std/str/trait.FromStr.html) trait for things which are explicitly parseable from strings.

However you can do more than just conversion types, and the [Unicode Segmentation](https://crates.io/crates/unicode-segmentation) crate binds a trait with many functions onto `str` to allow users to write things like:

```
use unicode_segmentation::UnicodeSegmentation;

// A vector of individual graphemes (true => extended).
let graphemes = "a\r\nb🇷🇺🇸🇹".graphemes(true).collect::<Vec<&str>>();

// Print each word (according to Unicode).
let s = "The quick (\"brown\") fox can't jump 32.3 feet, right?";
for w in s.unicode_words() {
  println!("{:?}", w);
}
```

Thus we could provide one or more ICU4X traits bound to things like `str` to provide a low friction way to access the libraries (obvious questions like naming notwithstanding).

# Appendix

## Sources

* [Learn Rust](https://doc.rust-lang.org/)
  * The canonical source for Rust information, but it doesn't offer advice on all aspects of code design.
* [Introduction - Learning Rust With Entirely Too Many Linked Lists](https://rust-unofficial.github.io/too-many-lists/)
  * This is brilliantly written and very educational about how you get into a Rust mindset.
* [Elegant Library APIs in Rust](https://deterministic.space/elegant-apis-in-rust.html)
  * This has a lot of good points and is well worth a read, but be warned that some of the details about implementation are somehwat out of date (2017). It has a video too.
* [Good Practices for Writing Rust Libraries](https://pascalhertleif.de/artikel/good-practices-for-writing-rust-libraries/)
  * Shorter and more focussed on the act of coding the libraries, and less about API design. Also potentially out of date in places.
* [Strategies for Returning References in Rust](http://bryce.fisher-fleig.org/blog/strategies-for-returning-references-in-rust/index.html)
  * Though I don't believe we should be doing this, it's still an interesting read.

## Other Useful Links

* Write and run Rust snippets: https://play.rust-lang.org
  * You can save snippets in permantent links and incluce them as working examples in docs.
* Write code and see what it compiles to: https://rust.godbolt.org
  * Note that you need to enable compiler optimizations via `-C opt-level=3` if you are looking to meaningfully compare two code snippets.
