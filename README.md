# p-engine

I tried to achieve program correctness as much as I could through rust typesystem.

`TransactionDTO` -> `Adjustment` (if deposit or withdraw) or `DisputeClaim` (if dispute). Dispute can be closed with `Resolution` (resolve or chargeback).

Basic use cases are covered by rust (unit) tests. Bigger rust test suite was not implemented due to time constraints, but is a must in a production system.

File IO was tested on minimal sample to make sure csv parsing works.

