# Programming by Navigation

[Programming by Navigation](https://dl.acm.org/doi/10.1145/3729264) is an
interactive program synthesis problem in which a step _provider_ provides a list
of next steps that a step _decider_ can choose between.

To solve the Programming by Navigation Synthesis Problem, these steps are
required to satisfy properties called Strong Soundness and Strong Completeness,
which roughly say that all provided steps can lead to a valid solution and that
all possibly-reachable valid solutions are reachable among just the provided
steps (respectively).

This Rust crate provides a helpful interface for working with the concepts
defined in Section 3 of the
[Programming by Navigation paper](https://dl.acm.org/doi/10.1145/3729264).
