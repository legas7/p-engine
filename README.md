# p-engine
Toy transaction processing engine.

Engine processes transactions. Transaction can introduce debit or credit to account. Transaction can be disputed, which opens a dispute. Dispute can be resolved with chargeback or be dropped. Common (banking) sense applies for things not specified.

## Overview
Much of program correctness is provided by typesystem rather by explicit checks/guards.

`TransactionDTO` -> `Adjustment` (if deposit or withdraw) or `DisputeClaim` (if dispute). Dispute can be closed with `Resolution` (resolve or chargeback).


## Testing
Basic use cases are covered by rust (unit) tests.

File IO was tested on minimal sample to make sure csv parsing works.
