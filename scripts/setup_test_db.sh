#!/bin/bash

# Create test database
psql -U postgres -c "DROP DATABASE IF EXISTS test_db;"
psql -U postgres -c "CREATE DATABASE test_db;"

# Apply migrations to test database
DATABASE_URL=postgres://postgres:postgres@localhost:5432/test_db sqlx migrate run 