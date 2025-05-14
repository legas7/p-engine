# p-engine


## Overview
Much of program correctness is provided by typesystem rather by explicit checks/guards.

`TransactionDTO` -> `Adjustment` (if deposit or withdraw) or `DisputeClaim` (if dispute). Dispute can be closed with `Resolution` (resolve or chargeback).


## Testing
Basic use cases are covered by rust (unit) tests. Bigger rust test suite was not implemented due to time constraints, but is a must in a production system.

File IO was tested on minimal sample to make sure csv parsing works.


## Improvements

### Error handling
For prototyping anyhow crate was used, but without much effort it should be changed for dedicated `enum EngineError` type. This would greatly improve edge-case readability and testability.

### Concurency
Currently program operates on two tokio tasks. One is responsible reading (streaming) transactions from file. Second is doing main processing loop inside `ProcessorImpl` struct. 

Program can be scaled horizontally by implementing e.g. `Balancer` component that would create pool of `ProcessorImpl` instances and forward user's transaction always to the same instance.

There is no limit on how many transaction streams or processing loops can be spawned.
