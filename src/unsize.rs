/*
A type implements Unsize<dyn Trait + 'a> if all of these conditions are met:
- The type implements Trait.
- Trait is object safe.
- The type is sized.
- The type outlives 'a.
*/

use std::marker::FnPtr;
