# uefi-things

## Description

uefi-things was created in order to simplify rust development of software written
using the [uefi](https://crates.io/crates/uefi) crate.

The goal is to remove boilerplate code as well as adding functionality for common functions
to allow for creating complex uefi software easily.

This has been tested with uefi 0.12.0

## Testing

Testing is done within the test-runner crate which has a simple library designed to run integration tests.

The test-runner library may be extended to further simplify the process