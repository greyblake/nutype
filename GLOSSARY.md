
* _Sanitizer_ - an **infallible** operation on a piece of data that converts the data to some canonical form.
Example: trim trailing spaces and lowercase an email address.

* _Validator_ - a **fallible** operation, that checks if a piece of data matches some given rules.
Example: ensure that an email address contains `@` character. Validators come with associated error variants.

* _Guard_ - a generic term that covers both sanitizers and validators.

* _Inner Type_ - typically a simple type that is wrapped by a newtype.
Example: consider `Email` type defined as `Email(String)`. We have say that `Email` has inner type `String`.

* _Standard trait_ - a trait that can be simply derived without any extra libraries (e.g. `Debug`, `Clone`).
* _Irregular trait_ a trait that requires a custom implementation to be generated (e.g. `TryFrom`).
