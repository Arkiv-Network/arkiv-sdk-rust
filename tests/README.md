  Running a Single Integration Test File

  Since each of our tests is in a separate file, you can run a specific test file with:

  cargo test --test test_create_and_read

  This runs all tests in tests/test_create_and_read.rs.

  Running by Test Function Name

  You can also run a specific test by its function name:

  cargo test test_create_and_read_entity

  This will find and run any test whose name matches the pattern.

  Running with Output

  To see println/eprintln output and more details:

  cargo test --test test_create_and_read -- --nocapture

  Or:

  cargo test test_create_and_read_entity -- --nocapture

  Examples for Our Tests

  # Run the create and read test
  cargo test --test test_create_and_read

  # Run the delete test
  cargo test --test test_delete_entity

  # Run the update test
  cargo test --test test_update_entity

  # Run the transfer ownership test
  cargo test --test test_transfer_ownership

  # Run the extend BTL test
  cargo test --test test_extend_btl

  # Run the multiple operations test
  cargo test --test test_multiple_operations

  # Run the query by owner test
  cargo test --test test_query_by_owner

  # Run the string attributes test
  cargo test --test test_string_attributes

  # Run the numeric attributes test
  cargo test --test test_numeric_attributes

  Listing Available Tests

  To see all available tests without running them:

  cargo test -- --list
