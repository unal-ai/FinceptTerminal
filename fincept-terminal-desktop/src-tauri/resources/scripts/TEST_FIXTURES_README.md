# Test Fixtures

This directory contains test fixture files used for integration and E2E testing.

## Files

### `databento_raw_data.bin`
**Type**: Mock/stub test data  
**Format**: Plain text representation of Rust structs (not actual Databento binary format)  
**Purpose**: Provides sample TradeMsg data for testing Databento integration without requiring actual binary wire format data.  
**Note**: This file contains formatted text representations of TradeMsg structs, not the actual binary format that Databento uses. It serves as a lightweight mock for basic integration tests. For comprehensive Databento parsing tests, consider using actual Databento binary format data.

### `test_volatile_data.csv`
**Type**: CSV test data  
**Format**: Standard CSV with headers  
**Purpose**: Sample volatile data for testing CSV parsing and data processing functionality.

## Usage

These fixtures are referenced by integration tests and may be loaded via the `FINCEPT_SCRIPTS_PATH` environment variable or directly by test code.
