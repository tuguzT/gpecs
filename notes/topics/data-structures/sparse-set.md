# Sparse set

Components of the same type are stored in a data structure called slot map
\[[1](/notes/bibliography/Data%20Structures%20for%20Game%20Developers%20The%20Slot%20Map.md), [2](/notes/bibliography/slot_map%20Container%20in%20C++.md)],
which is actually built on top of [sparse set](/notes/bibliography/Using%20Uninitialized%20Memory%20for%20Fun%20and%20Profit.md).

## Implementation

Slot map consists of two arrays:

- **sparse** array stores indices to the dense array,
- **dense** array stores *values* paired with indices to the sparse array, or *keys*.

![Sparse set structure](/notes/attachments/sparse-set-structure.png)

This round-trip is needed for time-efficient *O(1)* lookup by key
**with** time-efficient *O(n)* iteration by values while preserving their insertion order!
*O(1)* lookup by key means that addition & removal by key is *O(1)* too.
This also allows to trivially clean sparse set just by cleaning its dense array.

But this clearly is **not** memory-efficient: new array must be allocated for this data structure
and an index should be stored for each value.

## Why "sparse set" then?

That is because many developers, including myself, are usually using these terms interchangeably.

Original sparse set can only store integer keys, but slot map additionally stores values of these keys.
In other words, core properties of slot map are built on sparse set.

And so, sparse set could be thought as a specialization of slot map which stores empty values.
For example, in Rust, unit type could be used.
